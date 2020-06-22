use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Serialize, Debug)]
pub struct Node {
    pub id: i32,
    pub node: String,
    pub visited: bool,
}
