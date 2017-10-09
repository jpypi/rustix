use std::env;

use dotenv::dotenv;
use diesel;
use diesel::result::Error as DieselErr;
use diesel::prelude::*;
use diesel::pg::PgConnection;

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

    pub fn vote(&self, user: &str, entity: &str, up: i32, down: i32) {
        use super::schema::{users, voteables, votes};

        use super::schema::users::dsl as us;
        let user: User = match us::users.filter(us::user_id.eq(user)).load(&self.connection) {
            Ok(mut u) => u.pop().unwrap(),
            Err(_) => {
                let new_user = NewUser { user_id: user };
                diesel::insert(&new_user).into(users::table)
                    .get_result(&self.connection)
                    .expect("Error creating new user")
            }
        };

        println!("User id: {}", user.id);

        use super::schema::voteables::dsl::*;
        let mut voteable: Voteable = match voteables.filter(value.eq(entity)).load(&self.connection) {
            Ok(mut v) => v.pop().unwrap(),
            Err(_) => {
                let new_voteable = NewVoteable {
                    value: entity,
                    total_up: 0,
                    total_down: 0,
                };

                diesel::insert(&new_voteable).into(voteables::table)
                    .get_result(&self.connection)
                    .expect("Error creating new voteable")
            }
        };

        voteable.total_up += up;
        voteable.total_down += down;
        voteable.save_changes::<Voteable>(&self.connection);

        use super::schema::votes::dsl as vts;
        let mut vote: Vote = match vts::votes.filter(vts::user_id.eq(user.id))
                                         .filter(vts::voteable_id.eq(voteable.id))
                                         .load(&self.connection) {
            Ok(mut v) => v.pop().unwrap(),
            Err(_) => {
                let new_vote = NewVote{
                    user_id: user.id,
                    voteable_id: voteable.id,
                    up: 0,
                    down: 0,
                };

                diesel::insert(&new_vote).into(votes::table)
                    .get_result(&self.connection)
                    .expect("Error creating new vote")
            }
        };

        vote.up += up;
        vote.down += down;
        vote.save_changes::<Vote>(&self.connection);
    }

    pub fn get_upvotes(&self, entity: &str) -> Result<Voteable, DieselErr> {
        use super::schema::voteables::dsl::*;

        match voteables.filter(value.eq(entity)).load(&self.connection) {
            Ok(mut v) => Ok(v.pop().unwrap()),
            Err(e) => Err(e),
        }
    }
}
