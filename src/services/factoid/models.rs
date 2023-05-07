use std::string::String;
use std::time::SystemTime;

use diesel::types::SingleValue;
use diesel_derive_enum::DbEnum;

use super::super::db::schema::factoids;

#[derive(Debug, DbEnum)]
#[DieselType = "Factoid_kind"]
pub enum FactoidKind {
    Reply,
    Action,
}

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Factoid {
    pub id: i32,
    pub time: SystemTime,
    pub user_id: i32,
    pub pattern: String,
    pub kind: FactoidKind,
    pub value: String,
}

#[derive(Insertable)]
#[table_name = "factoids"]
pub struct NewFactoid<'a> {
    pub time: SystemTime,
    pub user_id: i32,
    pub pattern: &'a str,
    pub kind: FactoidKind,
    pub value: &'a str,
}
