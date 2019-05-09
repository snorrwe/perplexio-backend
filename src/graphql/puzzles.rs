use super::*;
use crate::entity::puzzle_entities::PuzzleEntity;
use crate::model::puzzle::Puzzle;
use crate::model::user::User;
use crate::schema;
use diesel::prelude::*;
use juniper::{self, FieldResult};

#[derive(GraphQLObject, Debug)]
pub struct PuzzleDTO {
    pub game_id: i32,
    pub game_table: Vec<String>,
    pub columns: i32,
    pub rows: i32,
}

pub fn fetch_puzzle_by_game_id(
    connection: &DieselConnection,
    _current_user: &User,
    game_id: i32,
) -> FieldResult<PuzzleDTO> {
    use self::schema::puzzles::dsl;

    // TODO: check if user has permission
    let result = dsl::puzzles
        .filter(dsl::game_id.eq(game_id))
        .get_result::<PuzzleEntity>(&connection.0)
        .map(|entity| Puzzle::from(entity))
        .map(|puzzle| {
            let (columns, rows) = puzzle.get_shape();
            PuzzleDTO {
                game_id: game_id,
                game_table: puzzle.render_table(),
                columns: columns as i32,
                rows: rows as i32,
            }
        })?;

    Ok(result)
}
