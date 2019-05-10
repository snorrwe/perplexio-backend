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
    current_user: &User,
    game_id: i32,
) -> FieldResult<PuzzleDTO> {
    use self::schema::game_participations as gp;
    use self::schema::games as g;
    use self::schema::puzzles::dsl;

    let result = dsl::puzzles
        .inner_join(g::table)
        .left_outer_join(gp::table.on(gp::dsl::game_id.eq(dsl::game_id)))
        .filter(dsl::game_id.eq(game_id))
        .filter(
            // check if the user has permissions
            gp::user_id
                .eq(current_user.id)
                .or(g::dsl::owner_id.eq(current_user.id)),
        )
        .select(dsl::puzzles::all_columns())
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

