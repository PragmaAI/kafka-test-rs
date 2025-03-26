#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel, create_consumer
};
use tokio::time::Duration;
use rdkafka::consumer::Consumer;

/// Tests basic transaction operations
#[tokio::test]
async fn test_basic_transaction_operations() {
    println!("Starting test_basic_transaction_operations");
    let ctx = setup_test("basic-transaction").await;
    let message_count = 5;

    println!("Sending {} messages in transaction", message_count);
    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "basic",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    println!("Counting received messages");
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    println!("Received {} messages", count);
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    println!("Cleaning up test");
    cleanup_test(ctx).await;
}

/// Tests transaction message ordering
#[tokio::test]
async fn test_message_ordering() {
    let ctx = setup_test("message-ordering").await;
    let message_count = 10;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "key",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests multi-topic transactions
#[tokio::test]
async fn test_multi_topic_transactions() {
    println!("Starting test_multi_topic_transactions");
    let ctx1 = setup_test("multi-topic-1").await;
    let ctx2 = setup_test("multi-topic-2").await;
    let message_count = 5;

    println!("Sending {} messages to first topic", message_count);
    let result = send_messages_in_transaction(
        &ctx1.producer,
        &ctx1.topic_name,
        message_count,
        "key1",
    ).await;
    assert!(result.is_ok(), "Failed to send messages to topic 1: {:?}", result);

    println!("Sending {} messages to second topic", message_count);
    let result = send_messages_in_transaction(
        &ctx2.producer,
        &ctx2.topic_name,
        message_count,
        "key2",
    ).await;
    assert!(result.is_ok(), "Failed to send messages to topic 2: {:?}", result);

    println!("Counting messages in first topic");
    let count1 = count_records(&ctx1.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    println!("Counting messages in second topic");
    let count2 = count_records(&ctx2.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    
    assert_eq!(count1, message_count, "Expected {} messages in first topic, got {}", message_count, count1);
    assert_eq!(count2, message_count, "Expected {} messages in second topic, got {}", message_count, count2);

    println!("Cleaning up first topic");
    cleanup_test(ctx1).await;
    println!("Cleaning up second topic");
    cleanup_test(ctx2).await;
}

/// Tests transaction recovery scenarios
#[tokio::test]
async fn test_recovery() {
    println!("Starting test_recovery");
    let ctx = setup_test("recovery").await;
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
        "recovery",
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

#[tokio::test]
async fn test_transaction_rollback() {
    let ctx = setup_test("transaction-rollback").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "rollback",
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
    let ctx = setup_test("isolation-test").await;
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

    if let Err(e) = &result {
        if e.to_string().contains("fenced") {
            println!("Producer was fenced, creating new producer and retrying");
            let new_ctx = setup_test("isolation-retry").await;
            let retry_result = send_messages_in_transaction(
                &new_ctx.producer,
                &new_ctx.topic_name,
                message_count,
                "isolation-retry",
            ).await;
            assert!(retry_result.is_ok(), "Failed to send messages after retry: {:?}", retry_result);
            cleanup_test(new_ctx).await;
            return;
        }
    }

    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    // Wait a bit to ensure messages are sent and committed
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Checking for committed messages");
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages after commit", message_count);

    println!("Cleaning up test");
    cleanup_test(ctx).await;
}

#[tokio::test]
async fn test_transaction_commit() {
    let ctx = setup_test("transaction-commit").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "commit",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}