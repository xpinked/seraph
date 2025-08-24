use broccoli_queue::{error::BroccoliError, queue::BroccoliQueue};
use seraph_workers::code_nodes::{CONCURRENCY, CodeNodeTask, QUEUE_NAME, WORKER_NAME, consumer};
use tracing::instrument::WithSubscriber;

#[tokio::main]
async fn main() -> Result<(), BroccoliError> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).with_test_writer().init();

    let config = seraph_core::config::Config::from_env();
    let _db = seraph_core::sea_orm::Database::connect(&config.db_url).await.unwrap();
    let _db = std::sync::Arc::new(_db);

    let queue = BroccoliQueue::builder(&config.redis_url)
        .pool_connections(5)
        .failed_message_retry_strategy(Default::default())
        .build()
        .with_current_subscriber()
        .await?;

    tracing::info!("Worker {WORKER_NAME} started, listening for tasks...");
    tracing::info!("Currently connected to Redis at {}", config.redis_url);
    tracing::info!("Listening on queue: {QUEUE_NAME}");
    tracing::info!("Concurrency set to {CONCURRENCY}");

    queue
        .process_messages(
            QUEUE_NAME,
            Some(CONCURRENCY as usize),
            None,
            move |msg: broccoli_queue::brokers::broker::BrokerMessage<CodeNodeTask>| {
                let value = _db.clone();
                async move { consumer(msg, value).await }
            },
        )
        .await
        .unwrap();
    Ok(())
}
