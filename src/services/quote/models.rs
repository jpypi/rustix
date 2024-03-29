use std::time::SystemTime;
use super::super::db::schema::quotes;

#[derive(Queryable, Identifiable, AsChangeset, Debug, Clone)]
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
