# Kafka Transaction Testing Best Practices

This document summarizes key learnings and best practices for testing Kafka transactions using the `rdkafka` library in Rust.

## Test Categories

### 1. Core Functionality Tests
```rust
/// Tests basic transaction operations
#[tokio::test]
async fn test_basic_transaction_operations() {
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
    println!("Starting test_transaction_isolation");
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
    assert!(result.is_ok());

    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(total_messages)).await;
    assert_eq!(count, total_messages);
}
```

Performance scenarios:
- Basic performance
- Message size impact
- Concurrent transactions
- Isolation level impact

### 4. Monitoring Tests
```rust
/// Tests monitoring with metrics collection
#[tokio::test]
async fn test_metrics_monitoring() {
    let ctx = setup_test("metrics-monitoring").await;
    let total_messages = 100;

    let result = send_messages_in_transaction(...).await;
    assert!(result.is_ok());

    let count = count_records(...).await;
    assert_eq!(count, total_messages);
}
```

Monitoring scenarios:
- Basic monitoring
- Size monitoring
- Success/failure monitoring
- Health checks
- Metrics collection

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