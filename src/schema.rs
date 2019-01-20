table! {
    game_participations (id) {
        id -> Int4,
        user_id -> Int4,
        game_id -> Int4,
        start_time -> Nullable<Timestamptz>,
        end_time -> Nullable<Timestamptz>,
    }
}

table! {
    games (id) {
        id -> Int4,
        name -> Varchar,
        owner_id -> Int4,
        puzzle -> Json,
        words -> Array<Text>,
        available_from -> Nullable<Timestamptz>,
        available_to -> Nullable<Timestamptz>,
        published -> Bool,
    }
}

table! {
    solutions (id) {
        id -> Int4,
        user_id -> Int4,
        game_id -> Int4,
        x1 -> Int4,
        y1 -> Int4,
        x2 -> Int4,
        y2 -> Int4,
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

joinable!(game_participations -> games (game_id));
joinable!(game_participations -> users (user_id));
joinable!(games -> users (owner_id));
joinable!(solutions -> games (game_id));
joinable!(solutions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    game_participations,
    games,
    solutions,
    users,
);
