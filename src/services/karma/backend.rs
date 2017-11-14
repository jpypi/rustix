use std::env;

use dotenv::dotenv;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use super::models::*;
use ::services::schema;

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
        use self::schema::{users, voteables, votes};

        let entity = &entity.to_lowercase();

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

        use self::schema::voteables::dsl::*;
        let mut res: Vec<Voteable> = voteables.filter(value.eq(entity))
                                              .load(&self.connection).unwrap();
        let mut voteable= match res.len() {
            0 => {
                let new_voteable = NewVoteable {
                    value: entity,
                    total_up: 0,
                    total_down: 0,
                };

                diesel::insert(&new_voteable).into(voteables::table)
                    .get_result(&self.connection)
                    .expect("Error creating new voteable")
            },

            _ => res.pop().unwrap(),
        };

        voteable.total_up += up;
        voteable.total_down += down;
        voteable.save_changes::<Voteable>(&self.connection);

        use ::services::schema::votes::dsl as vts;
        let mut res: Vec<Vote> = vts::votes.filter(vts::user_id.eq(user.id))
                                           .filter(vts::voteable_id.eq(voteable.id))
                                           .load(&self.connection).unwrap();
        let mut vote = match res.len() {
            0 => {
                let new_vote = NewVote{
                    user_id: user.id,
                    voteable_id: voteable.id,
                    up: 0,
                    down: 0,
                };

                diesel::insert(&new_vote).into(votes::table)
                    .get_result(&self.connection)
                    .expect("Error creating new vote")
            },
            _ => res.pop().unwrap(),
        };

        vote.up += up;
        vote.down += down;
        vote.save_changes::<Vote>(&self.connection);
    }

    pub fn get_upvotes(&self, entity: &str) -> Option<Voteable> {
        use self::schema::voteables::dsl::*;

        let entity = &entity.to_lowercase();

        let mut res = voteables.filter(value.eq(entity))
                               .load(&self.connection).unwrap();
        match res.len() {
            0 => None,
            _ => Some(res.pop().unwrap()),
        }
    }
}
