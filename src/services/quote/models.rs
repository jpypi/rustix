use std::time::SystemTime;
use crate::services::schema::{users, quotes};

#[derive(Queryable)]
#[derive(Identifiable)]
pub struct User {
    pub id: i32,
    pub user_id: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub user_id: &'a str,
}

// ------------

#[derive(Queryable)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
pub struct Quote {
    pub id: i32,
    pub quoter_id: i32,
    pub time: SystemTime,
    pub value: String,
}

#[derive(Insertable)]
#[table_name="quotes"]
pub struct NewQuote<'a> {
    pub quoter_id: i32,
    pub time: SystemTime,
    pub value: &'a str,
}
