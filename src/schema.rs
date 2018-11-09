table! {
    games (id) {
        id -> Int4,
        name -> Varchar,
        owner_id -> Int4,
        puzzle -> Json,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        googleid -> Varchar,
        auth_token -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    games,
    users,
);
