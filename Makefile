# Makefile for Kafka Transaction Tests

# Variables
CARGO = cargo
RUSTC = rustc
KAFKA_PORT = 9092
TEST_TIMEOUT = 300

# Colors for output
GREEN = \033[0;32m
RED = \033[0;31m
YELLOW = \033[1;33m
NC = \033[0m # No Color

# Default target
.PHONY: all
all: build

# Build the project
.PHONY: build
build:
	@echo "$(GREEN)Building project...$(NC)"
	$(CARGO) build
	@echo "$(GREEN)Build complete!$(NC)"

# Run all tests
.PHONY: test
test:
	@echo "$(GREEN)Running all tests...$(NC)"
	$(CARGO) test -- --nocapture
	@echo "$(GREEN)Tests complete!$(NC)"

# Run tests with debug output
.PHONY: test-debug
test-debug:
	@echo "$(GREEN)Running tests with debug output...$(NC)"
	RUST_LOG=debug RUST_BACKTRACE=1 $(CARGO) test -- --nocapture
	@echo "$(GREEN)Debug tests complete!$(NC)"

# Run specific test
.PHONY: test-specific
test-specific:
	@if [ "$(TEST)" = "" ]; then \
		echo "$(RED)Error: Please specify a test name using TEST=<test_name>$(NC)"; \
		echo "Example: make test-specific TEST=test_transaction_commit"; \
		exit 1; \
	fi
	@echo "$(GREEN)Running test: $(TEST)...$(NC)"
	$(CARGO) test $(TEST) -- --nocapture
	@echo "$(GREEN)Test complete!$(NC)"

# Run tests with timeout
.PHONY: test-timeout
test-timeout:
	@echo "$(GREEN)Running tests with $(TEST_TIMEOUT)s timeout...$(NC)"
	timeout $(TEST_TIMEOUT) $(CARGO) test -- --nocapture
	@echo "$(GREEN)Timeout tests complete!$(NC)"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	@echo "$(GREEN)Clean complete!$(NC)"

# Check if Kafka is running
.PHONY: check-kafka
check-kafka:
	@echo "$(GREEN)Checking if Kafka is running...$(NC)"
	@if nc -z localhost $(KAFKA_PORT); then \
		echo "$(GREEN)Kafka is running on port $(KAFKA_PORT)$(NC)"; \
	else \
		echo "$(RED)Error: Kafka is not running on port $(KAFKA_PORT)$(NC)"; \
		echo "Please start Kafka before running tests"; \
		exit 1; \
	fi

# Run tests with Kafka check
.PHONY: test-with-kafka-check
test-with-kafka-check: check-kafka test

# Run specific test with Kafka check
.PHONY: test-specific-with-kafka-check
test-specific-with-kafka-check: check-kafka test-specific

# Format code
.PHONY: fmt
fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	$(CARGO) fmt
	@echo "$(GREEN)Format complete!$(NC)"

# Run clippy linter
.PHONY: lint
lint:
	@echo "$(GREEN)Running clippy linter...$(NC)"
	$(CARGO) clippy
	@echo "$(GREEN)Lint complete!$(NC)"

# Run all checks (build, test, fmt, lint)
.PHONY: check-all
check-all: build fmt lint test

# Help target
.PHONY: help
help:
	@echo "$(GREEN)Available targets:$(NC)"
	@echo "  all                    - Build the project (default)"
	@echo "  build                  - Build the project"
	@echo "  test                   - Run all tests"
	@echo "  test-debug            - Run tests with debug output"
	@echo "  test-specific         - Run specific test (use TEST=<test_name>)"
	@echo "  test-timeout          - Run tests with timeout"
	@echo "  test-with-kafka-check - Run tests after checking Kafka"
	@echo "  test-specific-with-kafka-check - Run specific test after checking Kafka"
	@echo "  clean                 - Clean build artifacts"
	@echo "  fmt                   - Format code"
	@echo "  lint                  - Run clippy linter"
	@echo "  check-all            - Run all checks (build, test, fmt, lint)"
	@echo "  help                 - Show this help message"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make test-specific TEST=test_transaction_commit"
	@echo "  make test-debug"
	@echo "  make check-all" 