use uuid::Uuid;

pub fn rand_test_topic(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}
