use super::*;
use crate::model::solution::SolutionDTO;
use crate::model::vector::Vector;
use juniper::{self, FieldResult};

pub struct Mutation;

graphql_object!(Mutation: Context | &self | {
    field add_game(
        &executor,
        submission: games::GameSubmissionDTO
    ) -> FieldResult<games::GameDTO> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        games::add_game(connection, &user, submission)
    }

    /// Start the user's participation in the given game
    field start_participation(
        &executor,
        game_id: i32,
    ) -> FieldResult<bool> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        participations::add_participation(connection, user, game_id)
    }

    /// Submit a solution for checking
    /// Contract:
    /// solution must be a list of 4 integers
    /// [x1, y1, x2, y2]
    field submit_solution(&executor, game_id: i32, solution: Vec<i32>) -> FieldResult<bool> {
        let context = executor.context();
        let (connection, user) = (&context.connection, &context.user);
        let user = user.as_ref().ok_or("You need to log in first")?;
        if solution.len() != 4 {
            Err("Solution contains an invalid number of items")?;
        }
        let solution = SolutionDTO::new(Vector::new(solution[0], solution[1]), Vector::new(solution[2], solution[3]));
        solutions::submit_solution(connection, user, game_id, solution)
    }
});

