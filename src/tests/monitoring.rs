#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel
};

/// Tests basic monitoring
#[tokio::test]
async fn test_basic_monitoring() {
    let ctx = setup_test("basic-monitoring").await;
    let message_count = 5;

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

/// Tests monitoring with different message sizes
#[tokio::test]
async fn test_size_monitoring() {
    let ctx = setup_test("size-monitoring").await;
    let total_messages = 100;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        total_messages,
        "key",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
    assert_eq!(count, total_messages, "Expected {} messages, got {}", total_messages, count);

    cleanup_test(ctx).await;
}

/// Tests monitoring with success and failure scenarios
#[tokio::test]
async fn test_success_failure_monitoring() {
    let ctx = setup_test("success-failure-monitoring").await;
    let message_count = 5;

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

/// Tests monitoring with health checks
#[tokio::test]
async fn test_health_monitoring() {
    let ctx = setup_test("health-monitoring").await;
    let message_count = 5;

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

/// Tests monitoring with metrics collection
#[tokio::test]
async fn test_metrics_monitoring() {
    let ctx = setup_test("metrics-monitoring").await;
    let total_messages = 100;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        total_messages,
        "key",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
    assert_eq!(count, total_messages, "Expected {} messages, got {}", total_messages, count);

    cleanup_test(ctx).await;
}