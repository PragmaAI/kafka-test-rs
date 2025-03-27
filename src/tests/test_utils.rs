use std::time::{Duration, Instant};
use tokio::time::timeout;
use rdkafka::producer::Producer;
use rdkafka::error::KafkaError;

// Common test constants
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const OPERATION_TIMEOUT: Duration = Duration::from_secs(10);
pub const RETRY_DELAY: Duration = Duration::from_secs(1);
pub const MAX_RETRIES: u32 = 3;

/// Helper function to retry operations with timeout
pub async fn retry_with_timeout<F, Fut, T>(operation: F, operation_name: &str) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut retries = 0;
    loop {
        match timeout(OPERATION_TIMEOUT, operation()).await {
            Ok(result) => match result {
                Ok(value) => return Ok(value),
                Err(e) if retries < MAX_RETRIES => {
                    println!("{} failed with error: {}. Retrying...", operation_name, e);
                    retries += 1;
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                Err(e) => return Err(format!("{} failed after {} retries: {}", operation_name, retries, e)),
            },
            Err(_) => {
                if retries < MAX_RETRIES {
                    println!("{} timed out. Retrying...", operation_name);
                    retries += 1;
                    tokio::time::sleep(RETRY_DELAY).await;
                } else {
                    return Err(format!("{} timed out after {} retries", operation_name, retries));
                }
            }
        }
    }
}

/// Helper function to measure operation duration
pub struct OperationTimer {
    start_time: Instant,
    operation_name: String,
}

impl OperationTimer {
    pub fn new(operation_name: &str) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn print_duration(&self) {
        println!("{} completed in {:?}", self.operation_name, self.elapsed());
    }
}

/// Helper function to check producer health
pub fn check_producer_health(producer: &impl Producer) -> Result<(), String> {
    if let Some(error) = producer.client().fatal_error() {
        Err(format!("Producer has fatal error: {:?}", error))
    } else {
        Ok(())
    }
}

/// Helper function to handle transaction errors
pub fn handle_transaction_error(error: KafkaError) -> bool {
    if error.to_string().contains("fenced") {
        println!("Producer was fenced, should retry with new producer");
        true
    } else {
        println!("Transaction error: {:?}", error);
        false
    }
} 