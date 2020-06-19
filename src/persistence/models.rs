use super::schema::nodes;
use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct Node {
    pub id: i32,
    pub node: String,
    pub visited: bool,
}
