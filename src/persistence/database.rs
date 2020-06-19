use super::models;
use super::schema;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::io;
use std::{thread, time};

#[derive(Clone)]
pub struct Database {
    uri: String,
}

impl Database {
    pub fn new(uri: &str) -> Database {
        Database {
            uri: String::from(uri),
        }
    }

    pub fn get_conn(&self) -> Result<PgConnection, diesel::ConnectionError> {
        let conn = PgConnection::establish(&self.uri)?;
        Ok(conn)
    }

    pub fn wait_for_conn(
        &self,
        refresh_time: time::Duration,
        max_retries: u32,
    ) -> Result<(), io::Error> {
        for i in 0..max_retries {
            println!("Waiting for database connexion");
            match self.get_conn() {
                Ok(_conn) => {
                    println!("Database connexion is ready");
                    return Ok(());
                }
                Err(_) => println!(
                    "Database connexion is not ready yet, attempt {}/{}",
                    i, max_retries
                ),
            }
            thread::sleep(refresh_time);
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Could not connect to database after {} attempts",
                max_retries
            ),
        ))
    }

    pub async fn get_non_visited_nodes(
        &self,
        conn: &PgConnection,
        number: i64,
    ) -> QueryResult<Vec<models::Node>> {
        use schema::nodes::dsl::*;
        nodes
            .filter(visited.eq(true))
            .limit(number)
            .load::<models::Node>(conn)
    }
}
