# Kafka Transaction Tests

This project contains comprehensive tests for Kafka transactions using the `rdkafka` library in Rust. The tests verify various aspects of Kafka transactions including message ordering, transaction boundaries, and recovery scenarios.

## Features

- Transaction commit and abort testing
- Message ordering verification
- Concurrent transaction handling
- Multiple topics in single transaction
- Transaction timeout and fencing
- Transaction recovery after producer crashes
- Isolation level testing (read committed vs read uncommitted)
- Consumer group management
- Transaction retry logic
- Message size limits
- Transaction cleanup and resource management
- Resource limits testing
- Performance monitoring and metrics
- Health checks and observability

## Test Structure

The tests are organized into several key components:

### Test Categories

1. **Core Functionality Tests** (`src/tests/core_functionality.rs`)
   - Basic transaction operations
   - Message ordering
   - Multi-topic transactions
   - Transaction recovery
   - Transaction isolation

2. **Error Handling Tests** (`src/tests/error_handling.rs`)
   - Producer fencing
   - Transaction timeouts
   - Network failures
   - Invalid states
   - Edge cases

3. **Performance Tests** (`src/tests/performance.rs`)
   - Basic performance
   - Message size impact
   - Concurrent transactions
   - Isolation level impact

4. **Monitoring Tests** (`src/tests/monitoring.rs`)
   - Basic monitoring
   - Size monitoring
   - Success/failure monitoring
   - Health checks
   - Metrics collection

### Test Context
```rust
struct TestContext {
    topic_name: String,
    producer: FutureProducer,
    consumer: StreamConsumer,
}
```
This struct encapsulates the test resources and provides a consistent way to manage Kafka producers and consumers.

### Helper Functions
- `setup_test`: Creates test context with a new topic
- `send_messages_in_transaction`: Manages transaction-aware message sending
- `count_records`: Verifies message counts with isolation level support
- `cleanup_test`: Ensures proper resource cleanup

### Test Cases

1. **Transaction Boundaries**
   - Verifies proper transaction initialization
   - Tests transaction state management
   - Ensures proper error handling for invalid operations

2. **Message Ordering**
   - Verifies messages are received in the correct order
   - Tests strict ordering within transactions
   - Validates message content and sequence

3. **Transaction Commit/Abort**
   - Tests successful transaction commits
   - Verifies transaction abort behavior
   - Checks message visibility with different isolation levels

4. **Concurrent Transactions**
   - Tests multiple transactions running simultaneously
   - Verifies message consistency across transactions
   - Checks transaction isolation

5. **Multiple Topics**
   - Tests transactions spanning multiple topics
   - Verifies atomicity across topics
   - Checks message ordering in multi-topic scenarios

6. **Transaction Timeout**
   - Tests transaction timeout behavior
   - Verifies producer fencing
   - Checks message visibility after timeout

7. **Transaction Recovery**
   - Tests recovery after producer crashes
   - Verifies transaction state recovery
   - Checks message consistency after recovery

8. **Transaction Retry Logic**
   - Tests handling of network failures
   - Verifies producer recovery after network issues
   - Checks message consistency during retries

9. **Message Size Limits**
   - Tests behavior with messages exceeding Kafka limits
   - Verifies proper handling of large messages
   - Tests message key size limits

10. **Consumer Group Management**
    - Tests consumer group behavior with transactions
    - Verifies proper group coordination
    - Tests partition assignment and rebalancing

11. **Concurrent Consumers**
    - Tests multiple consumers with different isolation levels
    - Verifies proper isolation level behavior
    - Tests message visibility for different isolation levels

12. **Transaction Cleanup**
    - Verifies proper cleanup of transaction state
    - Tests for resource leaks
    - Validates cleanup after transaction completion

13. **Error Handling**
    - Tests comprehensive error handling scenarios
    - Verifies proper error propagation
    - Tests transaction state error handling

14. **Resource Limits**
    - Tests message count limits in transactions
    - Verifies transaction size limits
    - Tests resource constraints under load

15. **Transaction Metadata**
    - Tests message metadata handling
    - Verifies transaction ID tracking
    - Tests header management

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test tests::core_functionality
cargo test tests::error_handling
cargo test tests::performance
cargo test tests::monitoring

# Run specific test with output
cargo test test_name -- --nocapture
```

## Documentation

- [Core Testing Guide](docs/core.md) - Best practices and patterns for testing Kafka transactions
- [Use Cases](docs/usecases.md) - Real-world scenarios and testing patterns
- [Error Handling](docs/error_handling.md) - Comprehensive error handling strategies
- [Performance Testing](docs/performance.md) - Performance testing guidelines and metrics

## Progress

- Completed: 15 tests
- Pending: 2 tests
- Total: 17 tests

See [todo.md](todo.md) for detailed progress tracking and upcoming test cases.

## Contributing

1. Fork the repository
2. Create your feature branch
3. Add tests for new functionality
4. Update documentation
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.