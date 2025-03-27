use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord, Producer},
    error::{KafkaError, RDKafkaErrorCode},
    util::Timeout,
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
};
use std::time::Duration;
use uuid::Uuid;

/// Represents the isolation level for transactions
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadCommitted,
    ReadUncommitted,
}

/// Represents the test context with necessary resources
pub struct TestContext {
    pub topic_name: String,
    pub producer: FutureProducer,
    pub consumer: StreamConsumer,
}

/// Sets up a test environment with a unique topic and consumer
pub async fn setup_test(test_name: &str) -> TestContext {
    let topic_name = format!("test-topic-{}-{}", test_name, Uuid::new_v4());
    
    // Create admin client to create topic
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Failed to create admin client");

    // Create topic with 3 partitions
    let topic = NewTopic::new(&topic_name, 3, TopicReplication::Fixed(1));
    admin_client
        .create_topics(&[topic], &AdminOptions::new())
        .await
        .expect("Failed to create topic");

    // Wait for topic to be created
    tokio::time::sleep(Duration::from_secs(1)).await;

    let producer = create_producer(test_name).await;
    let consumer = create_consumer(IsolationLevel::ReadCommitted).await;
    
    // Subscribe to the topic
    consumer.subscribe(&[&topic_name])
        .expect("Failed to subscribe to topic");

    TestContext {
        topic_name,
        producer,
        consumer,
    }
}

/// Creates a producer with the given client ID
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

    let producer: FutureProducer = config.create()
        .expect("Failed to create producer");
    producer.init_transactions(Timeout::After(Duration::from_secs(10)))
        .expect("Failed to initialize transactions");
    producer
}

/// Creates a consumer with the specified isolation level
pub async fn create_consumer(isolation_level: IsolationLevel) -> StreamConsumer {
    let group_id = format!("test-group-{}", Uuid::new_v4());
    let isolation_level_str = match isolation_level {
        IsolationLevel::ReadCommitted => "read_committed",
        IsolationLevel::ReadUncommitted => "read_uncommitted",
    };

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", "localhost:9092")
        .set("group.id", group_id)
        .set("isolation.level", isolation_level_str)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("fetch.min.bytes", "1")
        .set("fetch.wait.max.ms", "100")
        .set("transaction.timeout.ms", "10000")
        .set("debug", "all");

    config.create()
        .expect("Failed to create consumer")
}

/// Sends messages in a transaction
pub async fn send_messages_in_transaction(
    producer: &FutureProducer,
    topic: &str,
    count: usize,
    key_prefix: &str,
) -> Result<(), KafkaError> {
    for attempt in 1..=3 {
        println!("Attempt {} to send messages in transaction", attempt);
        
        match producer.begin_transaction() {
            Ok(_) => {
                let mut success = true;
                for i in 0..count {
                    let key = format!("{}-{}", key_prefix, i);
                    let payload = format!("message-{}", i);
                    
                    match producer.send(
                        FutureRecord::to(topic)
                            .key(&key)
                            .payload(&payload),
                        Duration::from_secs(5),
                    ).await {
                        Ok(_) => continue,
                        Err((e, _)) => {
                            println!("Failed to send message: {:?}", e);
                            success = false;
                            break;
                        }
                    }
                }
                
                if success {
                    match producer.commit_transaction(Timeout::After(Duration::from_secs(5))) {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            println!("Failed to commit transaction: {:?}", e);
                            continue;
                        }
                    }
                } else {
                    let _ = producer.abort_transaction(Timeout::After(Duration::from_secs(5)));
                }
            }
            Err(e) => {
                println!("Failed to begin transaction: {:?}", e);
                continue;
            }
        }
    }
    
    Err(KafkaError::Global(RDKafkaErrorCode::Fail))
}

/// Counts the number of records in a consumer
pub async fn count_records(
    consumer: &StreamConsumer,
    isolation_level: IsolationLevel,
    expected_count: Option<usize>,
) -> usize {
    let mut count = 0;
    let timeout = match isolation_level {
        IsolationLevel::ReadUncommitted => Duration::from_millis(500),
        IsolationLevel::ReadCommitted => Duration::from_secs(5),
    };
    let start = std::time::Instant::now();

    // Poll for messages until timeout or expected count is reached
    while start.elapsed() < timeout {
        match consumer.recv().await {
            Ok(_message) => {
                count += 1;
                if let Some(expected) = expected_count {
                    if count >= expected {
                        // If we got all expected messages, return immediately
                        return count;
                    }
                }
            }
            Err(e) => {
                println!("Error receiving message: {:?}", e);
                if e.to_string().contains("Local: Timeout") {
                    // If we hit a timeout and have no expected count, return what we have
                    if expected_count.is_none() {
                        return count;
                    }
                    // Otherwise, continue polling
                    continue;
                }
                break;
            }
        }
    }

    count
}

/// Cleans up test resources
pub async fn cleanup_test(ctx: TestContext) {
    // Create admin client to delete topic
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Failed to create admin client");

    // Delete the topic
    admin_client
        .delete_topics(&[&ctx.topic_name], &AdminOptions::new())
        .await
        .expect("Failed to delete topic");

    // Unsubscribe the consumer
    ctx.consumer.unsubscribe();
    
    // Wait a bit to ensure cleanup is complete
    tokio::time::sleep(Duration::from_secs(1)).await;
} 