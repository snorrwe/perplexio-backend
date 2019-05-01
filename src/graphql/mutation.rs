use super::*;
use games;
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
});

