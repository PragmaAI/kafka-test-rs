#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel, create_producer
};
use std::time::Duration;
use tokio::task::JoinError;

use crate::tests::test_utils::{OperationTimer, retry_with_timeout};

/// Tests basic transaction performance
#[tokio::test]
async fn test_basic_performance() {
    let timer = OperationTimer::new("Basic performance");
    let ctx = setup_test("basic-perf").await;
    let message_count = 5;

    // Measure transaction performance
    let start = std::time::Instant::now();
    let result = retry_with_timeout(
        || async {
            send_messages_in_transaction(
                &ctx.producer,
                &ctx.topic_name,
                message_count,
                "perf",
            ).await.map_err(|e| e.to_string())
        },
        "Transaction send"
    ).await;

    assert!(result.is_ok(), "Transaction failed: {:?}", result);
    let transaction_duration = start.elapsed();
    println!("Transaction completed in {:?}", transaction_duration);

    // Verify messages with timeout
    let count = retry_with_timeout(
        || async {
            let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
            Ok(count)
        },
        "Message verification"
    ).await.expect("Failed to verify messages");

    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
    
    timer.print_duration();
}

/// Tests concurrent transaction performance
#[tokio::test]
async fn test_concurrent_performance() -> Result<(), String> {
    let timer = OperationTimer::new("Concurrent performance");
    let ctx = setup_test("concurrent-perf").await;
    let message_count = 5;
    let concurrent_transactions = 3;
    let total_messages = message_count * concurrent_transactions;

    // Send messages concurrently
    let mut handles = Vec::new();
    for i in 0..concurrent_transactions {
        let topic = ctx.topic_name.clone();
        let handle = tokio::spawn(async move {
            let producer = create_producer(&format!("concurrent-{}", i)).await;
            let start = std::time::Instant::now();
            let result = retry_with_timeout(
                || async {
                    send_messages_in_transaction(
                        &producer,
                        &topic,
                        message_count,
                        &format!("concurrent-{}", i),
                    ).await.map_err(|e| e.to_string())
                },
                &format!("Transaction {}", i)
            ).await;
            let duration = start.elapsed();
            (result, duration)
        });
        handles.push(handle);
    }

    // Wait for all transactions to complete
    let results = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, JoinError>>()
        .map_err(|e| e.to_string())?;

    // Verify all transactions succeeded
    for (i, (result, duration)) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Transaction {} failed: {:?}", i, result);
        println!("Transaction {} completed in {:?}", i, duration);
    }

    // Verify total message count with retries
    let mut retries = 3;
    let mut total_count = 0;
    while retries > 0 {
        let count = retry_with_timeout(
            || async {
                let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
                Ok(count)
            },
            "Message verification"
        ).await.expect("Failed to verify messages");

        total_count = count;
        if total_count == total_messages {
            break;
        }
        retries -= 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    assert_eq!(total_count, total_messages, "Expected {} messages, got {}", total_messages, total_count);

    cleanup_test(ctx).await;
    
    timer.print_duration();
    Ok(())
}

/// Tests performance with different isolation levels
#[tokio::test]
async fn test_isolation_level_performance() {
    let timer = OperationTimer::new("Isolation level performance");
    let ctx = setup_test("isolation-perf").await;
    let message_count = 5;

    for isolation_level in &[IsolationLevel::ReadCommitted, IsolationLevel::ReadUncommitted] {
        let start = std::time::Instant::now();
        
        // Send messages
        let result = retry_with_timeout(
            || async {
                send_messages_in_transaction(
                    &ctx.producer,
                    &ctx.topic_name,
                    message_count,
                    &format!("isolation-{:?}", isolation_level),
                ).await.map_err(|e| e.to_string())
            },
            "Transaction send"
        ).await;

        assert!(result.is_ok(), "Transaction failed: {:?}", result);
        let send_duration = start.elapsed();
        println!("Send completed in {:?} for {:?}", send_duration, isolation_level);

        // Verify messages
        let count = retry_with_timeout(
            || async {
                let count = count_records(&ctx.consumer, *isolation_level, Some(message_count)).await;
                Ok(count)
            },
            "Message verification"
        ).await.expect("Failed to verify messages");

        assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);
    }

    cleanup_test(ctx).await;
    
    timer.print_duration();
}