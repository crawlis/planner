use crate::nats::{NatsPublisher, NatsSubscriber};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::error;
use std::hash::{Hash, Hasher};
use std::io;
use url::Url;

pub struct PlannerConfig {
    nats_subscriber_uri: String,
    nats_subscriber_subject: String,
    nats_publisher_uri: String,
    nats_publisher_subject: String,
    starting_url: String,
}

impl PlannerConfig {
    pub fn new(
        nats_subscriber_uri: String,
        nats_subscriber_subject: String,
        nats_publisher_uri: String,
        nats_publisher_subject: String,
        starting_url: String,
    ) -> PlannerConfig {
        PlannerConfig {
            nats_subscriber_uri,
            nats_subscriber_subject,
            nats_publisher_uri,
            nats_publisher_subject,
            starting_url,
        }
    }
}

pub struct Planner {
    config: PlannerConfig,
    nats_subscriber: NatsSubscriber,
    nats_publisher: NatsPublisher,
}

impl Planner {
    pub fn new(config: PlannerConfig) -> Result<Planner, io::Error> {
        let nats_subscriber =
            NatsSubscriber::new(&config.nats_subscriber_uri, &config.nats_subscriber_subject)?;
        let nats_publisher =
            NatsPublisher::new(&config.nats_publisher_uri, &config.nats_publisher_subject)?;
        Ok(Planner {
            config: config,
            nats_subscriber,
            nats_publisher,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn error::Error>> {
        self.plan_next_urls(vec![self.config.starting_url.clone()])
            .await;
        loop {
            if let Some(message) = self.nats_subscriber.get_next_message() {
                match serde_json::from_slice::<CrawlingResults>(&message.data) {
                    Ok(crawling_results) => self.plan_next_urls(crawling_results.urls).await,
                    Err(err) => eprintln!("Could not deserialize message: {}", err),
                }
            }
        }
    }

    async fn plan_next_urls(&self, urls: Vec<String>) {
        urls.iter()
            .filter_map(|url| Url::parse(url).ok())
            .map(|url| url.into_string())
            .for_each(|url| {
                let key = format!("{}", calculate_hash(&url));
                if let Ok(message) = serde_json::to_vec(&url) {
                    if let Err(err) = self.nats_publisher.publish(&key, message) {
                        eprintln!("Could not publish url: {}", err);
                    }
                }
            });
    }
}

#[derive(Deserialize)]
struct CrawlingResults {
    parent: String,
    urls: Vec<String>,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
