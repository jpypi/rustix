use crate::services::schema::{users, voteables, votes};

#[derive(Queryable)]
#[derive(Identifiable)]
pub struct User {
    pub id: i32,
    pub user_id: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub user_id: &'a str,
}

// ------------

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Voteable {
    pub id: i32,
    pub value: String,
    pub total_up: i32,
    pub total_down: i32,
}

#[derive(Insertable)]
#[table_name="voteables"]
pub struct NewVoteable<'a> {
    pub value: &'a str,
    pub total_up: i32,
    pub total_down: i32,
}

// ------------

#[derive(Queryable)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[primary_key(user_id, voteable_id)]
pub struct Vote {
    pub user_id: i32,
    pub voteable_id: i32,
    pub up: i32,
    pub down: i32,
}

#[derive(Insertable)]
#[table_name="votes"]
pub struct NewVote {
    pub user_id: i32,
    pub voteable_id: i32,
    pub up: i32,
    pub down: i32,
}
