use diesel::prelude::*;
use diesel::PgConnection;

use super::schema::users::{self, dsl::*};

#[derive(Queryable, Identifiable, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub user_id: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub user_id: &'a str,
}

pub fn fetch_or_create(connection: &PgConnection, user: &str) -> QueryResult<User> {
    users
        .filter(user_id.eq(user))
        .get_result(connection)
        .or_else(|_| {
            let new_user = NewUser { user_id: user };
            diesel::insert_into(users::table)
                .values(new_user)
                .get_result(connection)
        })
}
