use std::env;
use std::time::SystemTime;

use rand;
use rand::Rng;

use dotenv::dotenv;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use ::services::schema;
use super::models::*;

pub struct Backend {
    connection: PgConnection
}

impl Backend {
    pub fn new() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let connection = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        Self {
            connection
        }
    }

    pub fn add_quote(&self, user: &str, quote: &str) -> i32{
        use self::schema::{users, quotes};

        use self::schema::users::dsl as us;
        let mut res: Vec<User> = us::users.filter(us::user_id.eq(user))
                                          .load(&self.connection).unwrap();
        let user = match res.len() {
            0 => {
                let new_user = NewUser { user_id: user };
                diesel::insert(&new_user).into(users::table)
                    .get_result(&self.connection)
                    .expect("Error creating new user")
            },
            _ => res.pop().unwrap(),
        };

        let new_quote = NewQuote {
            quoter_id: user.id,
            time: SystemTime::now(),
            value: quote,
        };

        diesel::insert(&new_quote).into(quotes::table)
            .get_result::<Quote>(&self.connection)
            .expect("Error adding quote").id
    }


    pub fn get_quote(&self, quote_id: i32) -> Option<(User, Quote)> {
        use self::schema::quotes::dsl as qu;
        use self::schema::users::dsl as us;

        let mut qres: Vec<Quote> = qu::quotes.filter(qu::id.eq(quote_id))
                                   .load(&self.connection).unwrap();
        match qres.len() {
            0 => None,
            _ => {
                let q = qres.pop().unwrap();
                let mut ures = us::users.filter(us::id.eq(q.quoter_id))
                                        .load(&self.connection).unwrap();
                Some((ures.pop().unwrap(), q))
            },
        }
    }

    pub fn random_quote(&self) -> Option<(User, Quote)> {
        use self::schema::quotes::dsl as qu;
        use self::schema::users::dsl as us;

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(0, qu::quotes.count()
                                                .load(&self.connection).unwrap()
                                                .pop().unwrap());

        // Try to query for a quote using a random offset
        let mut qres: Vec<Quote> = qu::quotes.offset(offset).limit(1)
                                   .load(&self.connection).unwrap();
        match qres.len() {
            0 => None,
            _ => {
                // Try to fetch user which created the quote
                let q = qres.pop().unwrap();
                let mut ures = us::users.filter(us::id.eq(q.quoter_id))
                                        .load(&self.connection).unwrap();
                Some((ures.pop().unwrap(), q))
            },
        }
    }
}
