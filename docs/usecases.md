# Kafka Transactions: Real-World Use Cases and Testing Patterns

This guide explores practical applications of Kafka transactions in real-world scenarios, providing examples, implementation patterns, and testing strategies.

## 1. Financial Services

### Payment Processing
**Use Case:** Ensuring atomic updates across payment-related events.

```rust
#[tokio::test]
async fn test_payment_processing() {
    let ctx = setup_test("payment-processing").await;
    
    // Setup multiple topics for different payment events
    let payment_attempts_topic = format!("{}-attempts", ctx.topic_name);
    let account_balances_topic = format!("{}-balances", ctx.topic_name);
    let notifications_topic = format!("{}-notifications", ctx.topic_name);

    // Begin transaction
    let result = producer.begin_transaction()?;
    
    // Send payment events atomically
    producer.send(Record::to(&payment_attempts_topic)
        .payload(&payment_attempt)
        .key(&payment_id))?;

    producer.send(Record::to(&account_balances_topic)
        .payload(&new_balance)
        .key(&account_id))?;

    producer.send(Record::to(&notifications_topic)
        .payload(&notification)
        .key(&user_id))?;

    producer.commit_transaction()?;

    // Verify all events are committed atomically
    let attempts_count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(1)).await;
    let balances_count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(1)).await;
    let notifications_count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(1)).await;

    assert_eq!(attempts_count, 1);
    assert_eq!(balances_count, 1);
    assert_eq!(notifications_count, 1);
}
```

Testing Considerations:
- Verify atomic updates across all payment-related topics
- Test rollback scenarios for failed payments
- Validate transaction isolation levels
- Test concurrent payment processing

### Banking Transfers
**Use Case:** Managing inter-account transfers with consistency.

```rust
#[tokio::test]
async fn test_bank_transfer() {
    let ctx = setup_test("bank-transfer").await;
    
    // Test successful transfer
    let result = send_messages_in_transaction(
        &ctx.producer,
        &[
            ("account-debits", debit_event),
            ("account-credits", credit_event),
            ("transfer-history", transfer_event),
            ("audit-logs", audit_event)
        ],
    ).await;
    
    assert!(result.is_ok());
    
    // Verify all events are present
    let events_count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(4)).await;
    assert_eq!(events_count, 4);
    
    // Test failed transfer (insufficient funds)
    let failed_result = send_messages_in_transaction(
        &ctx.producer,
        &[
            ("transfer-failures", failure_event),
            ("audit-logs", audit_event)
        ],
    ).await;
    
    assert!(failed_result.is_ok());
}
```

## 2. E-Commerce

### Order Processing
**Use Case:** Managing order lifecycle events atomically.

```rust
#[tokio::test]
async fn test_order_processing() {
    let ctx = setup_test("order-processing").await;
    
    // Test successful order
    let result = send_messages_in_transaction(
        &ctx.producer,
        &[
            ("inventory-updates", inventory_event),
            ("shipment-requests", shipment_event),
            ("order-history", order_event),
            ("email-notifications", email_event)
        ],
    ).await;
    
    assert!(result.is_ok());
    
    // Verify order state
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(4)).await;
    assert_eq!(count, 4);
}
```

### Inventory Management
**Use Case:** Maintaining consistency between inventory systems.

```rust
#[tokio::test]
async fn test_inventory_management() {
    let ctx = setup_test("inventory").await;
    
    // Test inventory reservation
    let reserve_result = send_messages_in_transaction(
        &ctx.producer,
        &[
            ("inventory-reserves", reserve_event),
            ("stock-levels", stock_event)
        ],
    ).await;
    
    assert!(reserve_result.is_ok());
    
    // Test backorder handling
    let backorder_result = send_messages_in_transaction(
        &ctx.producer,
        &[
            ("backorders", backorder_event),
            ("customer-notifications", notification_event)
        ],
    ).await;
    
    assert!(backorder_result.is_ok());
}
```

## 3. Event Sourcing Systems

### Event Store Testing
**Use Case:** Testing event consistency and snapshots.

```rust
#[tokio::test]
async fn test_event_store() {
    let ctx = setup_test("event-store").await;
    
    // Test event appending
    let events = vec![event1, event2, event3];
    let result = append_events(&ctx.producer, "aggregate-1", events).await;
    assert!(result.is_ok());
    
    // Test snapshot creation
    let snapshot_result = create_snapshot(&ctx.producer, "aggregate-1").await;
    assert!(snapshot_result.is_ok());
    
    // Verify event count
    let count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(4)).await;
    assert_eq!(count, 4); // 3 events + 1 snapshot
}
```

## 4. IoT and Sensor Data

### Sensor Data Testing
**Use Case:** Testing sensor data processing and aggregation.

```rust
#[tokio::test]
async fn test_sensor_data_processing() {
    let ctx = setup_test("sensor-data").await;
    
    // Test batch processing
    let readings = generate_test_readings(100);
    let result = process_sensor_batch(&ctx.producer, readings).await;
    assert!(result.is_ok());
    
    // Test anomaly detection
    let anomaly_readings = generate_anomaly_readings(10);
    let anomaly_result = process_sensor_batch(&ctx.producer, anomaly_readings).await;
    assert!(anomaly_result.is_ok());
    
    // Verify processed data
    let metrics_count = count_records(&ctx.consumer, IsolationLevel::ReadCommitted, Some(110)).await;
    assert_eq!(metrics_count, 110);
}
```

## Test Implementation Patterns

### 1. Transaction Retry Pattern
```rust
async fn test_with_retry<F>(mut operation: F) -> Result<()>
where
    F: FnMut() -> Future<Output = Result<()>>
{
    for attempt in 1..=3 {
        match operation().await {
            Ok(_) => return Ok(()),
            Err(e) if e.to_string().contains("fenced") => {
                println!("Retry attempt {}: Producer fenced", attempt);
                continue;
            },
            Err(e) => return Err(e),
        }
    }
    Err(Error::new("Max retries exceeded"))
}
```

### 2. Isolation Level Testing Pattern
```rust
async fn test_isolation_levels(ctx: &TestContext) -> Result<()> {
    // Test with read uncommitted
    let uncommitted_consumer = create_consumer(IsolationLevel::ReadUncommitted).await;
    let uncommitted_count = count_records(&uncommitted_consumer, ...).await;
    
    // Test with read committed
    let committed_consumer = create_consumer(IsolationLevel::ReadCommitted).await;
    let committed_count = count_records(&committed_consumer, ...).await;
    
    // Verify isolation behavior
    assert_ne!(uncommitted_count, committed_count);
    Ok(())
}
```

### 3. Multi-Topic Transaction Pattern
```rust
async fn test_multi_topic_transaction(ctx: &TestContext) -> Result<()> {
    let producer = &ctx.producer;
    producer.begin_transaction()?;
    
    for (topic, message) in messages {
        producer.send(Record::to(topic).payload(&message))?;
    }
    
    producer.commit_transaction()?;
    Ok(())
}
```

## Testing Best Practices

1. **Test Setup**
   - Use unique topic names for each test
   - Clean up resources after tests
   - Configure appropriate timeouts
   - Enable detailed logging

2. **Transaction Testing**
   - Test both success and failure scenarios
   - Verify transaction isolation
   - Test concurrent transactions
   - Validate cleanup procedures

3. **Error Handling**
   - Test producer fencing scenarios
   - Handle network failures
   - Test timeout conditions
   - Validate retry mechanisms

4. **Performance Testing**
   - Test with different message sizes
   - Measure transaction latency
   - Test concurrent load
   - Monitor resource usage

## Monitoring and Observability

1. **Metrics to Track**
   - Transaction success/failure rates
   - Transaction latency
   - Producer fencing events
   - Message throughput
   - Resource utilization

2. **Logging Patterns**
```rust
// Transaction logging
println!("Starting transaction: {}", transaction_id);
println!("Transaction result: {:?}", result);

// Performance logging
let start = Instant::now();
let result = operation().await;
println!("Operation took: {:?}", start.elapsed());
```

## Future Improvements

1. **Enhanced Testing**
   - Add more edge cases
   - Improve performance tests
   - Add stress testing
   - Test data recovery

2. **Monitoring**
   - Add Prometheus metrics
   - Implement health checks
   - Add performance monitoring
   - Track resource usage

3. **Documentation**
   - Add more examples
   - Document common issues
   - Add troubleshooting guides
   - Include performance tips 