use std::env;
use std::time::SystemTime;

use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

use dotenv::dotenv;
use diesel::prelude::*;
use diesel::pg::PgConnection;


use super::super::db::user::{self, *};
use super::super::db::schema::{
    users as us,
    users::dsl::users,
    quotes as qu,
    quotes::dsl::*,
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
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self {
            connection,
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn add_quote(&self, user: &str, quote: &str) -> QueryResult<i32>{
        let user = user::fetch_or_create(&self.connection, user).unwrap();

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
        let ures = us::dsl::users.filter(us::dsl::id.eq(qres.quoter_id))
                                 .get_result(&self.connection)?;
        Ok((ures, qres))
    }

    pub fn del_quote(&self, quote_id: i32) -> QueryResult<Quote>{
        diesel::delete(quotes.filter(qu::dsl::id.eq(quote_id)))
                             .get_result(&self.connection)
    }

    pub fn random_quote(&mut self) -> QueryResult<(User, Quote)> {
        let n_quotes = quotes.count().get_result(&self.connection)?;
        if n_quotes == 0 {
            return Err(diesel::result::Error::NotFound);
        }

        let offset = self.rng.gen_range(0..n_quotes);

        // Try to query for a quote using a random offset
        let qres: Quote = quotes.offset(offset).first(&self.connection)?;
        let ures = us::dsl::users.filter(us::dsl::id.eq(qres.quoter_id))
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

    pub fn search_quotes(&mut self, text: &str) -> QueryResult<Vec<Quote>> {
        let qfilter = quotes.filter(qu::dsl::value.ilike(format!("%{}%", text)));
        qfilter.load(&self.connection)
    }

    pub fn quote_by(&mut self, user_id: &str) -> QueryResult<(User, Quote)> {
        let res: Result<Vec<(User, Quote)>, diesel::result::Error> =
            quotes.inner_join(us::table)
                  .filter(us::user_id.eq(user_id))
                  .select((us::all_columns, qu::all_columns))
                  .load(&self.connection);

        if let Ok(quotes_res) = res {
            if let Some((u, q)) = quotes_res.choose(&mut self.rng) {
                return Ok((u.clone(), q.clone()));
            }
        }
        Err(diesel::result::Error::NotFound)
    }
}
