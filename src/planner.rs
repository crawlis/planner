use crate::nats::{NatsPublisher, NatsSubscriber};
use crate::persistence::database;
use crate::persistence::models;
use serde::Deserialize;
use std::error::Error;
use std::time;

pub struct PlannerConfig {
    nats_subscriber_uri: String,
    nats_subscriber_subject: String,
    nats_publisher_uri: String,
    nats_publisher_subject: String,
    database_uri: String,
}

impl PlannerConfig {
    pub fn new(
        nats_subscriber_uri: String,
        nats_subscriber_subject: String,
        nats_publisher_uri: String,
        nats_publisher_subject: String,
        database_uri: String,
    ) -> PlannerConfig {
        PlannerConfig {
            nats_subscriber_uri,
            nats_subscriber_subject,
            nats_publisher_uri,
            nats_publisher_subject,
            database_uri,
        }
    }
}

pub struct Planner {
    config: PlannerConfig,
    nats_subscriber: NatsSubscriber,
    nats_publisher: NatsPublisher,
    database: database::Database,
}

impl Planner {
    pub fn new(config: PlannerConfig) -> Result<Planner, std::io::Error> {
        let nats_subscriber =
            NatsSubscriber::new(&config.nats_subscriber_uri, &config.nats_subscriber_subject)?;
        let nats_publisher =
            NatsPublisher::new(&config.nats_publisher_uri, &config.nats_publisher_subject)?;
        let database = database::Database::new(&config.database_uri);
        Ok(Planner {
            config,
            nats_subscriber,
            nats_publisher,
            database,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        self.database
            .wait_for_conn(time::Duration::from_secs(2), 10)?;
        self.database.run_migrations()?;

        loop {}

        Ok(())
    }

    async fn persist_crawling_results(
        &self,
        crawling_results: CrawlingResults,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Deserialize)]
struct CrawlingResults {
    parent: String,
    urls: Vec<String>,
}
