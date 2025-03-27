# Kafka Transaction Testing Suite

A comprehensive test suite for testing Kafka transactions using the `rdkafka` library in Rust. This project provides a robust set of tests and utilities for verifying transaction behavior, error handling, performance, and monitoring capabilities.

## Features

- **Core Functionality Tests**
  - Basic transaction operations
  - Message ordering verification
  - Multi-topic transaction support
  - Transaction recovery testing
  - Transaction isolation testing

- **Error Handling**
  - Producer fencing scenarios
  - Transaction timeout handling
  - Network failure recovery
  - Invalid state detection
  - Edge case testing

- **Performance Testing**
  - Concurrent transaction testing
  - Message size impact analysis
  - Isolation level performance comparison
  - Resource usage monitoring

- **Monitoring Capabilities**
  - Transaction timing measurements
  - Memory usage tracking
  - Success/failure monitoring
  - Health checks
  - Metrics collection

## Prerequisites

- Rust 1.70 or later
- Kafka cluster running locally (default: localhost:9092)
- Docker (optional, for running Kafka in a container)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/kafka-tests.git
cd kafka-tests
```

2. Build the project:
```bash
cargo build
```

## Running Tests

### Basic Test Execution

```bash
cargo test
```

### Running Specific Test Categories

```bash
# Run core functionality tests
cargo test core_functionality

# Run error handling tests
cargo test error_handling

# Run performance tests
cargo test performance

# Run monitoring tests
cargo test monitoring
```

### Running with Verbose Output

```bash
cargo test -- --nocapture
```

## Test Categories

### 1. Core Functionality Tests
- Basic transaction operations
- Message ordering verification
- Multi-topic transaction support
- Transaction recovery
- Transaction isolation

### 2. Error Handling Tests
- Producer fencing scenarios
- Transaction timeout handling
- Network failure recovery
- Invalid state detection
- Edge case testing

### 3. Performance Tests
- Concurrent transaction testing
- Message size impact analysis
- Isolation level performance comparison
- Resource usage monitoring

### 4. Monitoring Tests
- Transaction timing measurements
- Memory usage tracking
- Success/failure monitoring
- Health checks
- Metrics collection

## Project Structure

```
kafka-tests/
├── src/
│   ├── tests/
│   │   ├── core_functionality.rs
│   │   ├── error_handling.rs
│   │   ├── monitoring.rs
│   │   ├── performance.rs
│   │   ├── test_low_producers.rs
│   │   ├── test_transactions.rs
│   │   └── test_utils.rs
│   ├── lib.rs
│   └── main.rs
├── docs/
│   ├── core.md
│   └── lessons.md
├── Cargo.toml
└── README.md
```

## Configuration

The test suite uses the following default configuration:

```rust
ClientConfig::new()
    .set("bootstrap.servers", "localhost:9092")
    .set("client.id", client_id)
    .set("transactional.id", format!("transactional-{}", client_id))
    .set("enable.idempotence", "true")
    .set("transaction.timeout.ms", "10000")
    .set("message.timeout.ms", "5000")
    .set("request.timeout.ms", "5000")
```

## Best Practices

1. **Test Setup**
   - Use unique topic names for each test
   - Implement proper cleanup after tests
   - Handle producer/consumer lifecycle
   - Manage transaction state

2. **Error Handling**
   - Implement retry mechanisms
   - Handle producer fencing
   - Verify error types and messages
   - Ensure proper error recovery

3. **Resource Management**
   - Clean up resources after tests
   - Monitor memory usage
   - Handle timeouts properly
   - Implement proper shutdown

4. **Performance Testing**
   - Use concurrent transactions
   - Measure transaction timing
   - Monitor resource usage
   - Track success/failure rates

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [rdkafka](https://github.com/fede1024/rdkafka) - Rust wrapper for librdkafka
- [librdkafka](https://github.com/confluentinc/librdkafka) - The Apache Kafka C/C++ client library
- [Kafka](https://kafka.apache.org/) - Distributed streaming platform