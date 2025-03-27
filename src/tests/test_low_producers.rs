//! Test data production using low level producers.
// copied from https://github.com/confluentinc/librdkafka/blob/master/tests/test_low_producers.rs

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use maplit::hashmap;
use uuid::Uuid;

use rdkafka::config::ClientConfig;
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use rdkafka::message::{Header, Message, OwnedHeaders, OwnedMessage};
use rdkafka::producer::{
    BaseProducer, BaseRecord, DeliveryResult, NoCustomPartitioner, Partitioner,
    Producer, ProducerContext, ThreadedProducer,
};
use rdkafka::types::RDKafkaRespErr;
use rdkafka::util::current_time_millis;
use rdkafka::{ClientContext, Statistics};
use rdkafka_sys;

use super::test_utils::{OperationTimer, check_producer_health};

#[derive(Clone)]
struct PrintingContext {
    _n: i64, // Add data for memory access validation
}

impl ClientContext for PrintingContext {
    // Access and use all stats.
    fn stats(&self, stats: Statistics) {
        let stats_str = format!("{:?}", stats);
        println!("Stats received: {} bytes", stats_str.len());
    }
}

impl ProducerContext for PrintingContext {
    type DeliveryOpaque = usize;

    fn delivery(&self, delivery_result: &DeliveryResult, delivery_opaque: Self::DeliveryOpaque) {
        println!("Delivery: {:?} {:?}", delivery_result, delivery_opaque);
    }
}

#[derive(Clone)]
struct CollectingContext<Part: Partitioner = NoCustomPartitioner> {
    stats: Arc<Mutex<Vec<Statistics>>>,
    results: Arc<Mutex<Vec<(OwnedMessage, Option<KafkaError>, usize)>>>,
    partitioner: Option<Part>,
}

impl CollectingContext {
    fn new() -> CollectingContext {
        CollectingContext {
            stats: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            partitioner: None,
        }
    }
}

impl<Part: Partitioner> CollectingContext<Part> {
    fn new_with_custom_partitioner(partitioner: Part) -> CollectingContext<Part> {
        CollectingContext {
            stats: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            partitioner: Some(partitioner),
        }
    }
}

impl<Part: Partitioner + Send + Sync> ClientContext for CollectingContext<Part> {
    fn stats(&self, stats: Statistics) {
        let mut stats_vec = self.stats.lock().unwrap();
        (*stats_vec).push(stats);
    }
}

impl<Part: Partitioner + Send + Sync> ProducerContext<Part> for CollectingContext<Part> {
    type DeliveryOpaque = usize;

    fn delivery(&self, delivery_result: &DeliveryResult, delivery_opaque: Self::DeliveryOpaque) {
        let mut results = self.results.lock().unwrap();
        match *delivery_result {
            Ok(ref message) => (*results).push((message.detach(), None, delivery_opaque)),
            Err((ref err, ref message)) => {
                (*results).push((message.detach(), Some(err.clone()), delivery_opaque))
            }
        }
    }

    fn get_custom_partitioner(&self) -> Option<&Part> {
        self.partitioner.as_ref()
    }
}

// Partitioner sending all messages to single, defined partition.
#[derive(Clone)]
pub struct FixedPartitioner {
    partition: i32,
}

impl FixedPartitioner {
    fn new(partition: i32) -> Self {
        Self { partition }
    }
}

impl Partitioner for FixedPartitioner {
    fn partition(
        &self,
        _topic_name: &str,
        _key: Option<&[u8]>,
        _partition_cnt: i32,
        _is_paritition_available: impl Fn(i32) -> bool,
    ) -> i32 {
        self.partition
    }
}

#[derive(Clone)]
pub struct PanicPartitioner {}

impl Partitioner for PanicPartitioner {
    fn partition(
        &self,
        _topic_name: &str,
        _key: Option<&[u8]>,
        _partition_cnt: i32,
        _is_paritition_available: impl Fn(i32) -> bool,
    ) -> i32 {
        panic!("partition() panic");
    }
}

/// Default Kafka configuration with common settings
fn default_config(config_overrides: HashMap<&str, &str>) -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "30000")
        .set("socket.timeout.ms", "30000")
        .set("request.timeout.ms", "30000")
        .set("metadata.request.timeout.ms", "30000")
        .set("retry.backoff.ms", "1000")
        .set("message.send.max.retries", "3");

    for (key, value) in config_overrides {
        config.set(key, value);
    }
    config
}

/// Creates a base producer with the given context and configuration
fn base_producer_with_context<Part: Partitioner, C: ProducerContext<Part>>(
    context: C,
    config_overrides: HashMap<&str, &str>,
) -> BaseProducer<C, Part> {
    default_config(config_overrides)
        .create_with_context::<C, BaseProducer<_, Part>>(context)
        .unwrap()
}

/// Creates a threaded producer with the given context and configuration
fn threaded_producer_with_context<Part, C>(
    context: C,
    config_overrides: HashMap<&str, &str>,
) -> ThreadedProducer<C, Part>
where
    Part: Partitioner + Send + Sync + 'static,
    C: ProducerContext<Part>,
{
    default_config(config_overrides)
        .create_with_context::<C, ThreadedProducer<_, _>>(context)
        .unwrap()
}

/// Generates a random test topic name
fn rand_test_topic(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

/// Tests queue full scenario
#[test]
fn test_base_producer_queue_full() {
    let timer = OperationTimer::new("Queue full test");
    let producer = base_producer_with_context(
        PrintingContext { _n: 123 },
        hashmap! { "queue.buffering.max.messages" => "10" }
    );
    let topic_name = rand_test_topic("test_base_producer_queue_full");

    // Send messages and collect results
    let results = (0..30)
        .map(|id| {
            producer.send(
                BaseRecord::with_opaque_to(&topic_name, id)
                    .payload("payload")
                    .key("key")
                    .timestamp(current_time_millis()),
            )
        })
        .collect::<Vec<_>>();

    // Wait for all messages to be processed
    while producer.in_flight_count() > 0 {
        producer.poll(Duration::from_millis(100));
    }

    // Verify results
    let errors = results
        .iter()
        .filter(|&e| {
            matches!(
                e,
                &Err((
                    KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull),
                    _
                ))
            )
        })
        .count();

    let success = results.iter().filter(|&r| r.is_ok()).count();

    assert_eq!(results.len(), 30);
    assert_eq!(success, 10);
    assert_eq!(errors, 20);
    
    timer.print_duration();
}

/// Tests producer timeout scenario
#[test]
fn test_base_producer_timeout() {
    let timer = OperationTimer::new("Timeout test");
    let context = CollectingContext::new();
    let producer = base_producer_with_context(
        context.clone(),
        hashmap! {
            "message.timeout.ms" => "1000",
            "bootstrap.servers" => "1.2.3.4", // Invalid server to force timeout
            "socket.timeout.ms" => "1000",
            "request.timeout.ms" => "1000"
        },
    );
    let topic_name = rand_test_topic("test_base_producer_timeout");

    // Send messages and collect results
    let send_results = (0..10)
        .map(|id| {
            producer.send(
                BaseRecord::with_opaque_to(&topic_name, id)
                    .payload("A")
                    .key("B"),
            )
        })
        .collect::<Vec<_>>();

    // All sends should succeed initially since they're just queued
    assert!(send_results.iter().all(|r| r.is_ok()));

    // Poll until messages are processed or timeout occurs
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    while start.elapsed() < timeout {
        producer.poll(Duration::from_millis(100));
        if let Ok(results) = context.results.lock() {
            if results.len() == 10 {
                break;
            }
        }
    }

    // Verify results
    let results = context.results.lock().unwrap();
    assert_eq!(results.len(), 10, "Expected 10 messages to be processed");
    
    for (msg, err, _) in results.iter() {
        assert_eq!(msg.payload_view::<str>(), Some(Ok("A")));
        assert_eq!(msg.key_view::<str>(), Some(Ok("B")));
        assert!(matches!(
            err,
            Some(KafkaError::MessageProduction(RDKafkaErrorCode::MessageTimedOut))
        ), "Expected message timeout error, got {:?}", err);
    }
    
    timer.print_duration();
}

/// Tests message headers
#[test]
fn test_base_producer_headers() {
    let timer = OperationTimer::new("Headers test");
    let context = CollectingContext::new();
    let producer = base_producer_with_context(context.clone(), HashMap::new());
    let topic_name = rand_test_topic("test_base_producer_headers");

    // Send messages with headers
    let results_count = (0..10)
        .map(|id| {
            let mut record = BaseRecord::with_opaque_to(&topic_name, id).payload("A");
            if id % 2 == 0 {
                record = record.headers(
                    OwnedHeaders::new()
                        .insert(Header {
                            key: "header1",
                            value: Some(&[1, 2, 3, 4]),
                        })
                        .insert(Header {
                            key: "header2",
                            value: Some("value2"),
                        })
                        .insert(Header {
                            key: "header3",
                            value: Some(&[]),
                        })
                        .insert::<Vec<u8>>(Header {
                            key: "header4",
                            value: None,
                        }),
                );
            }
            producer.send::<str, str>(record)
        })
        .filter(|r| r.is_ok())
        .count();

    // Wait for messages to be processed
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    while start.elapsed() < timeout {
        producer.poll(Duration::from_millis(100));
        if let Ok(results) = context.results.lock() {
            if results.len() == 10 {
                break;
            }
        }
    }

    // Verify results
    assert_eq!(results_count, 10);
    assert_eq!(context.results.lock().unwrap().len(), 10);
    
    timer.print_duration();
}

/// Tests threaded producer message sending
#[test]
fn test_threaded_producer_send() {
    let timer = OperationTimer::new("Threaded producer test");
    let context = CollectingContext::new();
    let producer = threaded_producer_with_context(context.clone(), HashMap::new());
    let topic_name = rand_test_topic("test_threaded_producer_send");

    // Send messages
    let results_count = (0..10)
        .map(|id| {
            producer.send(
                BaseRecord::with_opaque_to(&topic_name, id)
                    .payload("A")
                    .key("B"),
            )
        })
        .filter(|r| r.is_ok())
        .count();

    assert_eq!(results_count, 10);
    producer.flush(Duration::from_secs(10)).unwrap();

    // Verify results
    let delivery_results = context.results.lock().unwrap();
    let mut ids = HashSet::new();
    for &(ref message, ref error, id) in &(*delivery_results) {
        assert_eq!(message.payload_view::<str>(), Some(Ok("A")));
        assert_eq!(message.key_view::<str>(), Some(Ok("B")));
        assert_eq!(error, &None);
        ids.insert(id);
    }
    
    timer.print_duration();
}

/// Tests fatal error handling
#[test]
fn test_fatal_errors() {
    let timer = OperationTimer::new("Fatal errors test");
    let producer = base_producer_with_context(
        PrintingContext { _n: 123 },
        HashMap::new()
    );

    // Check initial state
    assert_eq!(producer.client().fatal_error(), None);

    // Simulate fatal error
    let msg = CString::new("fake error").unwrap();
    unsafe {
        rdkafka_sys::rd_kafka_test_fatal_error(
            producer.client().native_ptr(),
            RDKafkaRespErr::RD_KAFKA_RESP_ERR_OUT_OF_ORDER_SEQUENCE_NUMBER,
            msg.as_ptr(),
        );
    }

    // Verify error state
    assert_eq!(
        producer.client().fatal_error(),
        Some((
            RDKafkaErrorCode::OutOfOrderSequenceNumber,
            "test_fatal_error: fake error".into()
        ))
    );
    
    timer.print_duration();
}

#[test]
fn test_register_custom_partitioner_linger_non_zero_key_null() {
    // Custom partitioner is not used when sticky.partitioning.linger.ms > 0 and key is null.
    // https://github.com/confluentinc/librdkafka/blob/081fd972fa97f88a1e6d9a69fc893865ffbb561a/src/rdkafka_msg.c#L1192-L1196
    let context = CollectingContext::new_with_custom_partitioner(PanicPartitioner {});
    let mut config_overrides = HashMap::new();
    config_overrides.insert("sticky.partitioning.linger.ms", "10");
    let producer = base_producer_with_context(context.clone(), config_overrides);

    producer
        .send(
            BaseRecord::<(), str, usize>::with_opaque_to(
                &rand_test_topic("test_register_custom_partitioner_linger_non_zero_key_null"),
                0,
            )
            .payload(""),
        )
        .unwrap();

    // Poll until messages are processed or timeout occurs
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    while start.elapsed() < timeout {
        producer.poll(Duration::from_millis(100));
        if let Ok(results) = context.results.lock() {
            if results.len() == 1 {
                break;
            }
        }
    }

    let delivery_results = context.results.lock().unwrap();
    assert_eq!(delivery_results.len(), 1);
    for (_, error, _) in &(*delivery_results) {
        assert_eq!(*error, None);
    }
}

#[test]
fn test_custom_partitioner_base_producer() {
    let context = CollectingContext::new_with_custom_partitioner(FixedPartitioner::new(0));
    let producer = base_producer_with_context(context.clone(), HashMap::new());
    let topic_name = rand_test_topic("test_custom_partitioner_base_producer");

    let results_count = (0..10)
        .map(|id| {
            producer.send(
                BaseRecord::with_opaque_to(&topic_name, id)
                    .payload("")
                    .key(""),
            )
        })
        .filter(|r| r.is_ok())
        .count();

    assert_eq!(results_count, 10);

    // Poll until messages are processed or timeout occurs
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    while start.elapsed() < timeout {
        producer.poll(Duration::from_millis(100));
        if let Ok(results) = context.results.lock() {
            if results.len() == 10 {
                break;
            }
        }
    }

    let delivery_results = context.results.lock().unwrap();
    for (message, error, _) in &(*delivery_results) {
        assert_eq!(error, &None);
        assert_eq!(message.partition(), 0);
    }
}

#[test]
fn test_custom_partitioner_threaded_producer() {
    let context = CollectingContext::new_with_custom_partitioner(FixedPartitioner::new(0));
    let producer = threaded_producer_with_context(context.clone(), HashMap::new());
    let topic_name = rand_test_topic("test_custom_partitioner_threaded_producer");

    let results_count = (0..10)
        .map(|id| {
            producer.send(
                BaseRecord::with_opaque_to(&topic_name, id)
                    .payload("")
                    .key(""),
            )
        })
        .filter(|r| r.is_ok())
        .count();

    assert_eq!(results_count, 10);

    // Poll until messages are processed or timeout occurs
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    while start.elapsed() < timeout {
        if let Ok(results) = context.results.lock() {
            if results.len() == 10 {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let delivery_results = context.results.lock().unwrap();
    for (message, error, _) in &(*delivery_results) {
        assert_eq!(error, &None);
        assert_eq!(message.partition(), 0);
    }
}
                                                