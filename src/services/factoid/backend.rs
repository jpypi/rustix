use std::env;
use std::time::SystemTime;

use diesel::sql_types::Text;
use diesel::{prelude::*, sql_query, Connection, PgConnection};

use super::super::db::schema::{factoids as fs, users as us};
use super::super::db::user::{self, User};
use super::models::{Factoid, FactoidKind, NewFactoid};

pub struct Backend {
    connection: PgConnection,
}

impl Backend {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        Self { connection }
    }

    pub fn del_factoid(&self, id: i32) -> QueryResult<Factoid> {
        diesel::delete(fs::dsl::factoids.filter(fs::id.eq(id))).get_result(&self.connection)
    }

    pub fn add_factoid(&self, user: &str, kind: FactoidKind, pattern: &str, value: &str) {
        let user = user::fetch_or_create(&self.connection, user).unwrap();

        let f = NewFactoid {
            time: SystemTime::now(),
            user_id: user.id,
            pattern,
            kind,
            value,
        };

        diesel::insert_into(fs::table)
            .values(&f)
            .execute(&self.connection)
            .ok();
    }

    pub fn match_factoids(&self, query: &str) -> Vec<Factoid> {
        sql_query("SELECT * FROM factoids WHERE $1 ILIKE pattern || '%'")
            .bind::<Text, _>(query)
            .load(&self.connection)
            .unwrap()
    }

    pub fn get_user(&self, uid: i32) -> User {
        us::dsl::users
            .filter(us::id.eq(uid))
            .get_result(&self.connection)
            .unwrap()
    }
}
