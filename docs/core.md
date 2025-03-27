# Kafka Transaction Testing Best Practices

This document summarizes key learnings and best practices for testing Kafka transactions using the `rdkafka` library in Rust.

## Test Categories

### 1. Core Functionality Tests
```rust
/// Tests basic transaction operations
#[tokio::test]
async fn test_basic_transaction_operations() {
    let timer = OperationTimer::new("Basic transaction operations");
    let ctx = setup_test("basic-transaction").await;
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
    timer.print_duration();
}
```

Key test scenarios:
- Basic transaction operations
- Message ordering
- Multi-topic transactions
- Transaction recovery
- Transaction isolation

### 2. Error Handling Tests
```rust
/// Tests transaction isolation with retry logic
#[tokio::test]
async fn test_transaction_isolation() {
    let timer = OperationTimer::new("Transaction isolation");
    let ctx = setup_test("isolation-test").await;
    
    // Create a separate consumer with read uncommitted isolation
    let uncommitted_consumer = create_consumer(IsolationLevel::ReadUncommitted).await;
    uncommitted_consumer.subscribe(&[&ctx.topic_name])?;

    let result = send_messages_in_transaction(...).await;
    
    if let Err(e) = &result {
        if e.to_string().contains("fenced") {
            println!("Producer was fenced, creating new producer and retrying");
            let new_ctx = setup_test("isolation-retry").await;
            let retry_result = send_messages_in_transaction(...).await;
            assert!(retry_result.is_ok(), "Failed to send messages after retry");
            cleanup_test(new_ctx).await;
            return;
        }
    }
    // ... rest of the test
    timer.print_duration();
}
```

Key error scenarios:
- Producer fencing
- Transaction timeouts
- Network failures
- Invalid states
- Edge cases

### 3. Performance Tests
```rust
/// Tests concurrent transaction performance
#[tokio::test]
async fn test_concurrent_performance() -> Result<(), String> {
    let timer = OperationTimer::new("Concurrent performance");
    let ctx = setup_test("concurrent-perf").await;
    let message_count = 5;
    let concurrent_transactions = 3;

    // Send messages concurrently with separate producers
    let mut handles = Vec::new();
    for i in 0..concurrent_transactions {
        let topic = ctx.topic_name.clone();
        let handle = tokio::spawn(async move {
            let producer = create_producer(&format!("concurrent-{}", i)).await;
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
            result
        });
        handles.push(handle);
    }

    // Wait for all transactions to complete
    let results = futures::future::join_all(handles).await;
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Transaction {} failed: {:?}", i, result);
    }

    cleanup_test(ctx).await;
    timer.print_duration();
    Ok(())
}
```

Performance scenarios:
- Basic performance
- Concurrent transactions
- Isolation level impact
- Message size impact

### 4. Monitoring Tests
```rust
/// Tests basic monitoring with timing metrics
#[tokio::test]
async fn test_basic_monitoring() -> Result<(), String> {
    let timer = OperationTimer::new("Basic monitoring");
    let ctx = setup_test("basic-monitoring").await;
    let message_count = 5;

    // Add basic metrics collection
    let start_time = Instant::now();
    let initial_memory = get_memory_usage().await?;
    
    let result = send_messages_in_transaction(
        &ctx.producer,
        &ctx.topic_name,
        message_count,
        "key",
    ).await;
    assert!(result.is_ok());

    let duration = start_time.elapsed();
    let final_memory = get_memory_usage().await?;
    
    println!("Transaction completed in {:?}", duration);
    println!("Memory usage delta: {} KB", final_memory - initial_memory);

    // Verify message delivery
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
    assert_eq!(count, message_count);

    cleanup_test(ctx).await;
    timer.print_duration();
    Ok(())
}
```

Key monitoring scenarios:

1. **Basic Monitoring**
   - Transaction timing measurements
   - Success/failure tracking
   - Message count verification
   - Example metrics: `Transaction completed in 933ms`

2. **Size-based Monitoring**
   ```rust
   let message_sizes = vec![100, 1000, 10000]; // bytes
   for size in message_sizes {
       println!("Testing with message size: {} bytes", size);
       let start_time = Instant::now();
       
       // Send messages and measure performance
       let result = send_messages_in_transaction(...).await;
       
       let duration = start_time.elapsed();
       println!("Transaction completed in {:?} for size {}", duration, size);
   }
   ```
   - Tests different message sizes (100B, 1KB, 10KB)
   - Measures size impact on performance
   - Typical results:
     - 100B: ~1.5s for 100 messages
     - 1KB: ~650ms for 100 messages
     - 10KB: ~640ms for 100 messages

3. **Success/Failure Monitoring**
   ```rust
   // Test successful transaction
   let success_result = send_messages_in_transaction(
       &ctx.producer,
       &ctx.topic_name,
       message_count,
       "success",
   ).await;
   assert!(success_result.is_ok());

   // Test failed transaction
   let failure_result = send_messages_in_transaction(
       &ctx.producer,
       "invalid/topic/with/slashes",
       message_count,
       "failure",
   ).await;
   assert!(failure_result.is_err());
   ```
   - Tests both successful and failed scenarios
   - Verifies error handling
   - Monitors error types and frequencies
   - Example error: `KafkaError (Message production error: InvalidTopic)`

4. **Health Monitoring**
   ```rust
   // Check producer health
   assert!(ctx.producer.client().fatal_error().is_none());

   // Send messages with health checks
   let result = send_messages_in_transaction(...).await;
   assert!(result.is_ok());

   // Verify consumer health through message count
   let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(message_count)).await;
   assert_eq!(count, message_count);
   ```
   - Producer health verification
   - Consumer health checks
   - Fatal error detection
   - Message delivery confirmation

5. **Metrics Collection**
   ```rust
   // Collect pre-transaction metrics
   let start_time = Instant::now();
   let start_memory = get_process_memory();

   // Perform transaction
   let result = send_messages_in_transaction(...).await;

   // Collect post-transaction metrics
   let duration = start_time.elapsed();
   let end_memory = get_process_memory();

   println!("Transaction duration: {:?}", duration);
   println!("Memory usage: {} -> {} KB", start_memory, end_memory);
   ```
   - Transaction duration tracking
   - Memory usage monitoring
   - Resource utilization metrics
   - Performance statistics

## Test Setup Best Practices

### 1. Test Context Management
```rust
pub struct TestContext {
    pub topic_name: String,
    pub producer: FutureProducer,
    pub consumer: StreamConsumer,
}

pub async fn setup_test(test_name: &str) -> TestContext {
    let topic_name = format!("test-topic-{}-{}", test_name, Uuid::new_v4());
    // ... setup code
}
```

Key points:
- Use unique topic names
- Clean up resources after tests
- Handle producer/consumer lifecycle
- Manage transaction state

### 2. Producer Configuration
```rust
pub async fn create_producer(client_id: &str) -> FutureProducer {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", "localhost:9092")
        .set("client.id", client_id)
        .set("transactional.id", format!("transactional-{}", client_id))
        .set("enable.idempotence", "true")
        .set("transaction.timeout.ms", "10000")
        .set("message.timeout.ms", "5000")
        .set("request.timeout.ms", "5000")
        .set("debug", "all");
    // ... producer setup
}
```

### 3. Consumer Configuration
```rust
pub async fn create_consumer(isolation_level: IsolationLevel) -> StreamConsumer {
    let group_id = format!("test-group-{}", Uuid::new_v4());
    let isolation_level_str = match isolation_level {
        IsolationLevel::ReadCommitted => "read_committed",
        IsolationLevel::ReadUncommitted => "read_uncommitted",
    };
    // ... consumer setup
}
```

## Common Test Patterns

### 1. Transaction Retry Pattern
```rust
if let Err(e) = &result {
    if e.to_string().contains("fenced") {
        println!("Producer was fenced, retrying with new producer");
        // Retry logic
    }
}
```

### 2. Message Verification Pattern
```rust
let count = count_records(&consumer, IsolationLevel::ReadCommitted, Some(expected_count)).await;
assert_eq!(count, expected_count, "Expected {} messages, got {}", expected_count, count);
```

### 3. Resource Cleanup Pattern
```rust
pub async fn cleanup_test(ctx: TestContext) {
    // Delete topic
    // Unsubscribe consumer
    // Wait for cleanup completion
}
```

## Test Debugging Tips

1. **Enable Verbose Logging**
   ```rust
   println!("Starting operation: {}", operation_name);
   println!("Operation result: {:?}", result);
   ```

2. **Handle Timeouts**
   ```rust
   tokio::time::sleep(Duration::from_secs(1)).await;  // Add delays when needed
   ```

3. **Transaction State Verification**
   ```rust
   // Verify before commit
   let uncommitted_count = count_records(&uncommitted_consumer, ...).await;
   // Verify after commit
   let committed_count = count_records(&committed_consumer, ...).await;
   ```

## Future Improvements

1. **Enhanced Metrics**
   - Transaction latency tracking
   - Message size impact analysis
   - Concurrent transaction performance

2. **Advanced Error Scenarios**
   - Network partition simulation
   - Broker failure handling
   - Producer fencing scenarios

3. **Performance Benchmarks**
   - Throughput measurements
   - Latency profiling
   - Resource usage tracking

4. **Monitoring Integration**
   - Prometheus metrics
   - Health check endpoints
   - Alert configuration

Remember to:
- Keep tests focused and independent
- Clean up resources properly
- Handle errors gracefully
- Add appropriate logging
- Measure performance impacts 

## Best Practices for Monitoring Tests

1. **Timing Measurements**
   - Use `Instant::now()` for precise timing
   - Measure both overall and per-operation durations
   - Log timing data for analysis

2. **Resource Monitoring**
   - Track memory usage before and after operations
   - Monitor system resource utilization
   - Watch for resource leaks

3. **Error Handling**
   - Test both success and failure paths
   - Verify error types and messages
   - Ensure proper error recovery

4. **Health Checks**
   - Regular producer/consumer health verification
   - Fatal error detection
   - Connection status monitoring

5. **Performance Metrics**
   - Message size impact analysis
   - Throughput measurements
   - Latency tracking
   - Resource usage patterns

## Monitoring Implementation Tips

1. **Metric Collection**
   ```rust
   // Basic timing metric
   let start = Instant::now();
   // ... operation ...
   let duration = start.elapsed();

   // Memory usage
   let memory = std::process::Command::new("ps")
       .arg("-o")
       .arg("rss=")
       .arg("-p")
       .arg(std::process::id().to_string())
       .output()
       .unwrap();
   ```

2. **Error Monitoring**
   ```rust
   // Check for specific error types
   match result {
       Ok(_) => println!("Operation successful"),
       Err(e) => println!("Error type: {:?}", e),
   }
   ```

3. **Health Verification**
   ```rust
   // Producer health
   assert!(producer.client().fatal_error().is_none());

   // Consumer health via message count
   let count = count_records(&consumer, isolation_level, expected_count).await;
   assert_eq!(count, expected_count);
   ```

## Future Monitoring Improvements

1. **Enhanced Metrics**
   - Detailed transaction state tracking
   - Per-message timing statistics
   - Network usage monitoring
   - Partition distribution analysis

2. **Advanced Health Checks**
   - Connection pool monitoring
   - Broker health verification
   - Consumer group lag tracking
   - Resource limit monitoring

3. **Performance Analytics**
   - Historical performance tracking
   - Trend analysis
   - Anomaly detection
   - Performance regression testing

4. **Integration Monitoring**
   - External system health checks
   - End-to-end transaction tracking
   - Cross-service metrics
   - System-wide health status

Remember to:
- Keep monitoring non-intrusive
- Log relevant metrics
- Track resource usage
- Monitor both success and failure cases
- Implement health checks
- Collect performance data 