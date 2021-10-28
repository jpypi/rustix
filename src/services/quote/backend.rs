use std::env;
use std::time::SystemTime;

use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;

use dotenv::dotenv;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::services::schema::{
    users as us,
    quotes as qu,
    quotes::dsl::*,
    users::dsl::*,
};
use super::models::*;


pub struct Backend {
    connection: PgConnection,
    rng: SmallRng,
}

impl Backend {
    pub fn new() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let connection = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        Self {
            connection: connection,
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn add_quote(&self, user: &str, quote: &str) -> QueryResult<i32>{
        let user: User = match users.filter(user_id.eq(user))
                                    .get_result(&self.connection) {
            Ok(u) => u,
            Err(_) => {
                let new_user = NewUser { user_id: user };
                diesel::insert_into(us::table)
                    .values(&new_user)
                    .get_result(&self.connection)?
            }
        };

        let new_quote = NewQuote {
            quoter_id: user.id,
            time: SystemTime::now(),
            value: quote,
        };

        Ok(diesel::insert_into(qu::table)
           .values(&new_quote)
           .get_result::<Quote>(&self.connection)?.id
        )
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

    pub fn random_quote(&mut self) -> QueryResult<(User, Quote)> {
        let offset = self.rng.gen_range(0..quotes.count().get_result(&self.connection)?);

        // Try to query for a quote using a random offset
        let qres: Quote = quotes.offset(offset).first(&self.connection)?;
        let ures = users.filter(us::dsl::id.eq(qres.quoter_id))
                            .first(&self.connection)?;
        Ok((ures, qres))
    }

    pub fn search_quote(&mut self, text: &str) -> QueryResult<(User, Quote)> {
        let qfilter = quotes.filter(qu::dsl::value.ilike(format!("%{}%", text)));

        // Compute a random offset so we get random ones if there are multiple
        let count = qfilter.clone().count().get_result(&self.connection)?;
        if count > 0 {
            let offset = self.rng.gen_range(0..count);

            let qres: Quote = qfilter.offset(offset).first(&self.connection)?;
            let ures = users.filter(us::dsl::id.eq(qres.quoter_id))
                            .get_result(&self.connection)?;

            Ok((ures, qres))
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
}
