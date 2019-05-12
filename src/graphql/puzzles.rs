use super::*;
use crate::entity::puzzle_entities::{PuzzleEntity, PuzzleUpdate};
use crate::model::puzzle::Puzzle;
use crate::model::user::User;
use crate::schema;
use diesel::dsl::{delete, update};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use juniper::{self, FieldResult};

#[derive(GraphQLObject, Debug)]
pub struct PuzzleDTO {
    pub game_id: i32,
    pub game_table: Vec<String>,
    pub columns: i32,
    pub rows: i32,
    pub words: Vec<String>,
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
        .get_result::<PuzzleEntity>(connection)
        .map(|entity| Puzzle::from(entity))
        .map(|puzzle| {
            let (columns, rows) = puzzle.get_shape();
            PuzzleDTO {
                game_id: game_id,
                game_table: puzzle.render_table(),
                columns: columns as i32,
                rows: rows as i32,
                words: puzzle.get_words().clone(),
            }
        })?;

    Ok(result)
}

pub fn regenerate_puzzle(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<PuzzleDTO> {
    use self::schema::game_participations as gp;
    use self::schema::games as g;
    use self::schema::puzzles as p;
    use self::schema::solutions as s;

    let result = connection.transaction::<_, DieselError, _>(|| {
        let puzzle = p::table
            .filter(p::dsl::game_id.eq(game_id))
            .inner_join(g::table)
            .filter(g::dsl::owner_id.eq(current_user.id))
            .select(p::table::all_columns())
            .get_result::<PuzzleEntity>(connection)?;

        delete(s::table)
            .filter(s::game_id.eq(game_id))
            .execute(connection)?;

        delete(gp::table)
            .filter(gp::game_id.eq(game_id))
            .execute(connection)?;

        let puzzle = Puzzle::from_words(puzzle.words, 200).map_err(|e| {
            error!("Failed to generate puzzle {:?}", e);
            DieselError::RollbackTransaction
        })?;

        let puzzle = PuzzleUpdate::from(puzzle);

        update(p::table)
            .filter(p::dsl::game_id.eq(game_id))
            .set(puzzle)
            .get_result::<PuzzleEntity>(connection)
    })?;

    let result = Puzzle::from(result);

    let (columns, rows) = result.get_shape();

    let result = PuzzleDTO {
        game_id: game_id,
        game_table: result.render_table(),
        columns: columns as i32,
        rows: rows as i32,
        words: result.get_words().clone(),
    };

    Ok(result)
}

