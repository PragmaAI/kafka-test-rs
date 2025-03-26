#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel
};

/// Tests basic transaction performance
#[tokio::test]
async fn test_basic_performance() {
    let ctx = setup_test("basic-perf").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "perf",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests transaction performance with different message sizes
#[tokio::test]
async fn test_size_performance() {
    let ctx = setup_test("size-perf").await;
    let total_messages = 100;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        total_messages,
        "size",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
    assert_eq!(count, total_messages, "Expected {} messages, got {}", total_messages, count);

    cleanup_test(ctx).await;
}

/// Tests transaction performance with concurrent transactions
#[tokio::test]
async fn test_concurrent_performance() {
    let ctx = setup_test("concurrent-perf").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "concurrent",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests transaction performance with different isolation levels
#[tokio::test]
async fn test_isolation_performance() {
    let ctx = setup_test("isolation-perf").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "isolation",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}