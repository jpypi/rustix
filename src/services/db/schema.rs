table! {
    use diesel::sql_types::*;
    use crate::services::factoid::models::*;

    factoids (id) {
        id -> Int4,
        time -> Timestamp,
        user_id -> Int4,
        pattern -> Text,
        kind -> Factoid_kind,
        value -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    quotes (id) {
        id -> Int4,
        quoter_id -> Int4,
        time -> Timestamp,
        value -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        user_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    voteables (id) {
        id -> Int4,
        value -> Varchar,
        total_up -> Int4,
        total_down -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    votes (user_id, voteable_id) {
        user_id -> Int4,
        voteable_id -> Int4,
        up -> Int4,
        down -> Int4,
    }
}

joinable!(factoids -> users (user_id));
joinable!(quotes -> users (quoter_id));
joinable!(votes -> users (user_id));
joinable!(votes -> voteables (voteable_id));

allow_tables_to_appear_in_same_query!(factoids, quotes, users, voteables, votes,);
