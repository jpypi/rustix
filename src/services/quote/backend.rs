use std::env;
use std::time::SystemTime;

use rand;
use rand::Rng;

use dotenv::dotenv;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use ::services::schema::{
    users as us,
    quotes as qu,
    quotes::dsl::*,
    users::dsl::*,
};
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

    pub fn add_quote(&self, user: &str, quote: &str) -> QueryResult<i32>{
        let user: User = match users.filter(user_id.eq(user))
                                    .get_result(&self.connection) {
            Ok(u) => u,
            Err(_) => {
                let new_user = NewUser { user_id: user };
                diesel::insert(&new_user).into(us::table)
                    .get_result(&self.connection)?
            }
        };

        let new_quote = NewQuote {
            quoter_id: user.id,
            time: SystemTime::now(),
            value: quote,
        };

        Ok(diesel::insert(&new_quote).into(qu::table)
            .get_result::<Quote>(&self.connection)?.id)
    }


    pub fn get_quote(&self, quote_id: i32) -> QueryResult<(User, Quote)> {
        let qres: Quote = quotes.filter(qu::dsl::id.eq(quote_id))
                                        .first(&self.connection)?;
        let ures = users.filter(us::dsl::id.eq(qres.quoter_id))
                            .get_result(&self.connection)?;
        Ok((ures, qres))
    }

    pub fn del_quote(&self, quote_id: i32) -> QueryResult<Quote>{
        diesel::delete(quotes.filter(qu::dsl::id.eq(quote_id)))
                             .get_result(&self.connection)
    }

    pub fn random_quote(&self) -> QueryResult<(User, Quote)> {
        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(0, quotes.count().get_result(&self.connection)?);

        // Try to query for a quote using a random offset
        let qres: Quote = quotes.offset(offset).first(&self.connection)?;
        let ures = users.filter(us::dsl::id.eq(qres.quoter_id))
                            .first(&self.connection)?;
        Ok((ures, qres))
    }
}
