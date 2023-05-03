use std::env;

use dotenv::dotenv;
use diesel::{
    pg::PgConnection,
    prelude::*,
};

use crate::services::schema::{
    users,
    users::dsl as us,
    voteables,
    voteables::dsl::*,
    votes,
    votes::dsl as vts,
};

use super::models::*;
use super::super::db::user::*;


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

    pub fn vote(&self, user: &str, entity: &str, up: i32, down: i32) -> QueryResult<()> {

        let entity = &entity.to_lowercase();

        let mut res: Vec<User> = us::users.filter(us::user_id.eq(user))
                                          .load(&self.connection).unwrap();
        let user = match res.len() {
            0 => {
                let new_user = NewUser { user_id: user };
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .get_result(&self.connection)
                    .expect("Error creating new user")
            },
            _ => res.pop().unwrap(),
        };

        let mut res: Vec<Voteable> = voteables.filter(value.eq(entity))
                                              .load(&self.connection)?;
        let mut voteable = match res.len() {
            0 => {
                let new_voteable = NewVoteable {
                    value: entity,
                    total_up: 0,
                    total_down: 0,
                };

                diesel::insert_into(voteables::table)
                        .values(&new_voteable)
                        .get_result(&self.connection)?
            },
            _ => res.pop().unwrap(),
        };

        voteable.total_up += up;
        voteable.total_down += down;
        voteable.save_changes::<Voteable>(&self.connection)?;

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

                diesel::insert_into(votes::table).values(&new_vote)
                    .get_result(&self.connection)
                    .expect("Error creating new vote")
            },
            _ => res.pop().unwrap(),
        };

        vote.up += up;
        vote.down += down;
        vote.save_changes::<Vote>(&self.connection)?;

        Ok(())
    }

    pub fn get_upvotes(&self, entity: &str) -> QueryResult<Option<Voteable>> {
        let entity = &entity.to_lowercase();

        let mut res = voteables.filter(value.eq(entity))
                               .load(&self.connection)?;
        match res.len() {
            0 => Ok(None),
            _ => Ok(Some(res.pop().unwrap())),
        }
    }

    pub fn voteables_rank_desc(&self, n: i64) -> QueryResult<Vec<Voteable>> {
        voteables.order((total_up - total_down).desc())
                                               .limit(n)
                                               .load(&self.connection)
    }

    pub fn voteables_rank_asc(&self, n: i64) -> QueryResult<Vec<Voteable>> {
        voteables.order((total_up - total_down).asc())
                                               .limit(n)
                                               .load(&self.connection)
    }

    pub fn votes_rank(&self, item: &str, n: i64) -> QueryResult<Vec<(String, i32, i32)>> {
        /*
        SELECT votes.up,votes.down,users.user_id FROM
            votes
        JOIN users ON
            votes.user_id = users.id
        JOIN voteables ON
            voteables.id = votes.voteable_id
        WHERE
            voteables.value = '$item'
        ORDER BY
            (votes.up - votes.down) DESC;
        LIMIT $n
        */

        votes::table.inner_join(users::table)
                    .inner_join(voteables::table)
                    .select((users::user_id,
                             votes::up,
                             votes::down))
                    .filter(voteables::value.eq(item))
                    .order((votes::up - votes::down).desc())
                    .limit(n)
                    .load(&self.connection)
    }

    pub fn user_ranks(&self, user: &str, n: i64) -> QueryResult<Vec<(String, i32, i32)>>{
        /*
        SELECT voteables.value, votes.up, votes.down FROM
            votes
        JOIN voteables ON
            votes.voteable_id = voteables.id
        JOIN users ON
            votes.user_id = users.id
        WHERE
            users.user_id = '$user'
        ORDER BY
            (votes.up - votes.down) DESC
        LIMIT $n;
        */
        votes::table.inner_join(voteables::table)
                    .inner_join(users::table)
                    .select((voteables::value,
                             votes::up,
                             votes::down))
                    .filter(users::user_id.eq(user))
                    .order((votes::up - votes::down).desc())
                    .limit(n)
                    .load(&self.connection)
    }

    pub fn user_ranks_asc(&self, user: &str, n: i64) -> QueryResult<Vec<(String, i32, i32)>>{
        /*
        SELECT voteables.value, votes.up, votes.down FROM
            votes
        JOIN voteables ON
            votes.voteable_id = voteables.id
        JOIN users ON
            votes.user_id = users.id
        WHERE
            users.user_id = '$user'
        ORDER BY
            (votes.up - votes.down) ASC
        LIMIT $n;
        */
        votes::table.inner_join(voteables::table)
                    .inner_join(users::table)
                    .select((voteables::value,
                             votes::up,
                             votes::down))
                    .filter(users::user_id.eq(user))
                    .order((votes::up - votes::down).asc())
                    .limit(n)
                    .load(&self.connection)
    }
}
