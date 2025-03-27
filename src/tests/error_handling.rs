#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel, create_consumer
};
use std::time::Duration;
use rdkafka::consumer::Consumer;

use crate::tests::test_utils::{OperationTimer, retry_with_timeout, handle_transaction_error};

/// Tests basic transaction error scenarios
#[tokio::test]
async fn test_basic_error_handling() {
    let timer = OperationTimer::new("Basic error handling");
    let ctx = setup_test("basic-error").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "error",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
    
    timer.print_duration();
}

/// Tests error recovery scenarios
#[tokio::test]
async fn test_error_recovery() {
    let ctx = setup_test("error-recovery").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "recovery",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests edge cases and invalid states
#[tokio::test]
async fn test_edge_cases() {
    let ctx = setup_test("edge-cases").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "edge",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests transaction isolation levels
#[tokio::test]
async fn test_transaction_isolation() {
    println!("Starting test_transaction_isolation");
    let ctx = setup_test("transaction-isolation").await;
    let message_count = 5;

    // Create a separate consumer with read uncommitted isolation
    println!("Creating uncommitted consumer");
    let uncommitted_consumer = create_consumer(IsolationLevel::ReadUncommitted).await;
    uncommitted_consumer.subscribe(&[&ctx.topic_name])
        .expect("Failed to subscribe to topic");

    // Wait a bit to ensure consumer has joined the group
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Sending messages in transaction");
    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "isolation",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    // Wait a bit to ensure messages are sent and committed
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Checking for committed messages");
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages after commit", message_count);

    println!("Cleaning up test");
    cleanup_test(ctx).await;
}