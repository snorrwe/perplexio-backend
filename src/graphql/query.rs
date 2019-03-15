use super::*;
use games;
use puzzles;
use juniper::{self, FieldResult};

pub struct Query;

graphql_object!(Query: Context |&self| {
    /// "version of the api"
    field apiVersion() -> &str {
        "0.1.0"
    }

    field games(
        &executor,
        page: Option<i32>
    ) -> FieldResult<games::PaginatedGames> {
        let context = executor.context();
        let (connection,user) = (&context.connection, &context.user);
        games::fetch_games(connection, user, page)
    }

    field game(
        &executor,
        id: i32
    ) -> FieldResult<games::GameDTO> {
        let context = executor.context();
        let (connection,user) = (&context.connection, &context.user);
        games::fetch_game_by_id(connection, user, id)
    }

    field puzzle(
        &executor,
        game_id: i32
    ) -> FieldResult<puzzles::PuzzleDTO> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or_else(||"You have to log in first")?;
        puzzles::fetch_puzzle_by_game_id(connection, &user, game_id)
    }
});

