use crate::nats::NatsPublisher;
use crate::persistence::{database, models};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::error;
use std::hash::{Hash, Hasher};
use std::io;
use url::Url;

pub struct PlannerConfig {
    nats_publisher_uri: String,
    nats_publisher_subject: String,
    database_uri: String,
    starting_url: String,
}

impl PlannerConfig {
    pub fn new(
        nats_publisher_uri: String,
        nats_publisher_subject: String,
        database_uri: String,
        starting_url: String,
    ) -> PlannerConfig {
        PlannerConfig {
            nats_publisher_uri,
            nats_publisher_subject,
            database_uri,
            starting_url,
        }
    }
}

pub struct Planner {
    config: PlannerConfig,
    nats_publisher: NatsPublisher,
    database: database::Database,
}

impl Planner {
    pub fn new(config: PlannerConfig) -> io::Result<Planner> {
        let nats_publisher =
            NatsPublisher::new(&config.nats_publisher_uri, &config.nats_publisher_subject)?;
        let database = database::Database::new(&config.database_uri);
        Ok(Planner {
            config: config,
            nats_publisher,
            database,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn error::Error>> {
        loop {
            let database_conn = self.database.get_conn()?;
            match self
                .database
                .get_non_visited_nodes(&database_conn, 10)
                .await
            {
                Ok(nodes) => {
                    self.plan_next_urls(nodes.iter().map(|node| node.node).collect())
                        .await
                }
                Err(err) => {
                    eprintln!("Could not retrieve non-visited: {}", err);
                    self.plan_next_urls(vec![self.config.starting_url.clone()])
                        .await;
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
