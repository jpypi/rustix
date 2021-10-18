table! {
    quotes (id) {
        id -> Int4,
        quoter_id -> Int4,
        time -> Timestamp,
        value -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        user_id -> Text,
    }
}

table! {
    voteables (id) {
        id -> Int4,
        value -> Varchar,
        total_up -> Int4,
        total_down -> Int4,
    }
}

table! {
    votes (user_id, voteable_id) {
        user_id -> Int4,
        voteable_id -> Int4,
        up -> Int4,
        down -> Int4,
    }
}

joinable!(quotes -> users (quoter_id));
joinable!(votes -> users (user_id));
joinable!(votes -> voteables (voteable_id));

allow_tables_to_appear_in_same_query!(
    quotes,
    users,
    voteables,
    votes,
);
