#[cfg(test)]
use crate::tests::common::{
    setup_test,
    send_messages_in_transaction, count_records, cleanup_test,
    IsolationLevel
};
use std::time::Duration;
use tokio::time::timeout;
use rdkafka::producer::Producer;
use rdkafka::consumer::Consumer;

use crate::tests::test_utils::{OperationTimer, retry_with_timeout, check_producer_health};

const TEST_TIMEOUT: Duration = Duration::from_secs(30);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(10);
const RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRIES: u32 = 3;

/// Tests basic monitoring with timing metrics
#[tokio::test]
async fn test_basic_monitoring() -> Result<(), String> {
    let timer = OperationTimer::new("Basic monitoring");
    let message_count = 100;
    let ctx = setup_test("basic_monitoring").await;

    let start_time = std::time::Instant::now();
    send_messages_in_transaction(&ctx.producer, &ctx.topic_name, message_count, "basic")
        .await
        .map_err(|e| e.to_string())?;
    let duration = start_time.elapsed();

    // Verify message count
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Message count mismatch");
    println!("Transaction completed in {:?}", duration);

    cleanup_test(ctx).await;
    
    timer.print_duration();
    Ok(())
}

/// Tests monitoring with different message sizes and their impact
#[tokio::test]
async fn test_size_monitoring() -> Result<(), String> {
    let sizes = vec![100, 1024, 10240]; // 100B, 1KB, 10KB
    let messages_per_size = 10;

    for size in sizes {
        let ctx = setup_test(&format!("size_monitoring_{}", size)).await;
        
        let start_time = std::time::Instant::now();
        let total_messages = messages_per_size;
        
        // Send messages of specified size
        for _ in 0..messages_per_size {
            send_messages_in_transaction(&ctx.producer, &ctx.topic_name, 1, "size")
                .await
                .map_err(|e| e.to_string())?;
        }
        
        let duration = start_time.elapsed();
        
        // Verify message count
        let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
        assert_eq!(count, total_messages, "Message count mismatch for size {}", size);
        println!("Size {} bytes: Transaction completed in {:?}", size, duration);
        
        cleanup_test(ctx).await;
    }
    Ok(())
}

/// Tests monitoring with success and failure scenarios
#[tokio::test]
async fn test_success_failure_monitoring() -> Result<(), String> {
    let ctx = setup_test("success_failure_monitoring").await;
    let message_count = 10;

    // Test successful transaction
    let start_time = std::time::Instant::now();
    send_messages_in_transaction(&ctx.producer, &ctx.topic_name, message_count, "success")
        .await
        .map_err(|e| e.to_string())?;
    let duration = start_time.elapsed();

    // Verify successful transaction
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Message count mismatch for successful transaction");
    println!("Successful transaction completed in {:?}", duration);

    // Test failed transaction (invalid topic)
    let result = send_messages_in_transaction(&ctx.producer, "invalid/topic/with/slashes", message_count, "failure").await;
    assert!(result.is_err(), "Expected transaction to fail");
    println!("Failed transaction error: {:?}", result.err().unwrap());

    cleanup_test(ctx).await;
    Ok(())
}

/// Tests monitoring with health checks
#[tokio::test]
async fn test_health_monitoring() -> Result<(), String> {
    let ctx = setup_test("health_monitoring").await;
    let message_count = 10;

    // Monitor producer health
    let producer_start = std::time::Instant::now();
    send_messages_in_transaction(&ctx.producer, &ctx.topic_name, message_count, "health")
        .await
        .map_err(|e| e.to_string())?;
    let producer_duration = producer_start.elapsed();

    // Verify message count
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count, "Message count mismatch");
    println!("Producer health check completed in {:?}", producer_duration);

    // Check for fatal errors
    assert!(ctx.producer.client().fatal_error().is_none(), "Producer has fatal error");
    
    cleanup_test(ctx).await;
    Ok(())
}

/// Tests monitoring with detailed metrics collection
#[tokio::test]
async fn test_metrics_monitoring() -> Result<(), String> {
    let ctx = setup_test("metrics_monitoring").await;
    let total_messages = 100;
    let batch_size = 10;

    // Get initial memory usage
    let initial_memory = get_memory_usage().await?;
    println!("Initial memory usage: {} KB", initial_memory);

    let start_time = std::time::Instant::now();
    
    // Send messages in batches
    for _ in 0..(total_messages / batch_size) {
        send_messages_in_transaction(&ctx.producer, &ctx.topic_name, batch_size, "metrics")
            .await
            .map_err(|e| e.to_string())?;
    }
    
    let duration = start_time.elapsed();

    // Get final memory usage
    let final_memory = get_memory_usage().await?;
    println!("Final memory usage: {} KB", final_memory);

    // Verify message count
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
    assert_eq!(count, total_messages, "Message count mismatch");
    println!("Metrics monitoring completed in {:?}", duration);
    println!("Memory usage delta: {} KB", final_memory - initial_memory);

    cleanup_test(ctx).await;
    Ok(())
}

async fn get_memory_usage() -> Result<u64, String> {
    let output = std::process::Command::new("ps")
        .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .map_err(|e| format!("Failed to get memory usage: {}", e))?;
    
    String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse memory usage: {}", e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse memory usage: {}", e))
}