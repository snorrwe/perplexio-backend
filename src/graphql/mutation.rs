use super::*;
use crate::model::solution::SolutionDTO;
use crate::model::vector::Vector;
use juniper::{self, FieldResult};

pub struct Mutation {}

graphql_object!( Mutation: Context as "Mutation" | &self | {
    field add_game(
        &executor,
        submission: games::GameSubmissionDTO
    ) -> FieldResult<games::GameDTO> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        let connection = unsafe {
            &*connection
        };
        games::add_game(connection, &user, submission)
    }

    /// Start the user's participation in the given game
    field start_participation(
        &executor,
        game_id: i32,
    ) -> FieldResult<bool> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        let connection = unsafe {
            &*connection
        };
        participations::add_participation(connection, user, game_id)
    }

    /// Submit a solution for checking
    /// Contract:
    /// solution must be a list of 4 integers
    /// [x1, y1, x2, y2]
    field submit_solution(&executor, game_id: i32, solution: Vec<i32>) -> FieldResult<bool> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        if solution.len() != 4 {
            Err("Solution contains an invalid number of items")?;
        }
        let solution = SolutionDTO::new(Vector::new(solution[0], solution[1]), Vector::new(solution[2], solution[3]));
        let connection = unsafe {
            &*connection
        };
        solutions::submit_solution(connection, user, game_id, solution)
    }

    field publish_game(&executor, game_id: i32) -> FieldResult<bool> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        let connection = unsafe {
            &*connection
        };
        games::publish_game(connection, user, game_id)
    }

    field regenerate_puzzle(&executor, game_id: i32) -> FieldResult<puzzles::PuzzleDTO> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        let connection = unsafe {
            &*connection
        };
        puzzles::regenerate_puzzle(connection, user, game_id)
    }

    field update_game(&executor, payload: games::GameUpdateDTO) -> FieldResult<games::GameDTO> {
        let context = executor.context();
        let (connection, user) = (context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        let connection = unsafe {
            &*connection
        };
        games::update_game(connection, user, payload)
    }
});

