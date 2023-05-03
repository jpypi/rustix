use crate::services::schema::users;

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