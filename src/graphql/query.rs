use super::*;
use crate::model::solution::SolutionDTO;
use games;
use juniper::{self, FieldResult};
use puzzles;

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

    /// Get the participations for the given game
    /// Requires user to be the owner
    field all_participations_by_game(
        &executor,
        game_id: i32,
    ) -> FieldResult<Vec<participations::GameParticipationDTO>> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or_else(||"You have to log in first")?;
        participations::get_all_participations(connection, user, game_id)
    }

    /// Get the participations of the current user
    field participations(&executor) -> FieldResult<Vec<participations::GameParticipationDTO>> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or_else(||"You have to log in first")?;
        participations::get_participations(connection, user)
    }

    /// Get the user's participation belonging to the game
    field participation(
        &executor,
        game_id: i32,
    ) -> FieldResult<participations::GameParticipationDTO> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or_else(||"You have to log in first")?;
        participations::get_participation(connection, user, game_id)
    }

    field get_solution_by_game_id(&executor, game_id: i32) -> FieldResult<Vec<SolutionDTO>> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or_else(||"You have to log in first")?;
        solutions::get_solution_by_game_id(connection, user, game_id)
    }
});
