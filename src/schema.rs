table! {
    games (id) {
        id -> Int4,
        name -> Varchar,
        owner_id -> Int4,
        puzzle -> Json,
        words -> Array<Text>,
        available_from -> Nullable<Timestamptz>,
        available_to -> Nullable<Timestamptz>,
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

joinable!(games -> users (owner_id));

allow_tables_to_appear_in_same_query!(
    games,
    users,
);
