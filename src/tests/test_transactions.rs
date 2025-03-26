#![allow(dead_code)]

use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel
};

/// Tests basic transaction functionality
#[tokio::test]
async fn test_basic_transaction() {
    let ctx = setup_test("basic").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "basic",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

/// Tests transaction rollback functionality
#[tokio::test]
async fn test_transaction_rollback() {
    let ctx = setup_test("rollback").await;
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
    let ctx = setup_test("isolation").await;
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

#[tokio::test]
async fn test_transaction_commit() {
    let ctx = setup_test("test-transaction-commit").await;
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

#[tokio::test]
async fn test_transaction_abort() {
    let ctx = setup_test("test-transaction-abort").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "abort",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

#[tokio::test]
async fn test_transaction_multiple() {
    let ctx = setup_test("test-transaction-multiple").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "multiple",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

#[tokio::test]
async fn test_transaction_commit_success() {
    let ctx = setup_test("test-transaction-commit-success").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "commit-success",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
}

#[tokio::test]
async fn test_transaction_abort_success() {
    let ctx = setup_test("test-transaction-abort-success").await;
    let message_count = 5;

    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "abort-success",
    ).await;
    assert!(result.is_ok(), "Failed to send messages: {:?}", result);

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Expected {} messages, got {}", message_count, count);

    cleanup_test(ctx).await;
} 