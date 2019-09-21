use super::participations::end_participation;
use crate::entity::game_entities::GameEntity;
use crate::model::solution::{SolutionDTO, SolutionEntity, SolutionForm};
use crate::model::user::User;
use crate::model::vector::Vector;
use crate::DieselConnection;
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use juniper::FieldResult;

pub fn get_solution_by_game_id(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<Vec<SolutionDTO>> {
    let r = get_users_solutions(&connection, &current_user, game_id).map_err(|e| {
        error!("Failed to read users solutions {:?}", e);
        "Failed to fetch solutions"
    })?;
    Ok(r)
}

/// Return all solutions submitted for a game by the user
pub fn get_users_solutions(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> Result<Vec<SolutionDTO>, DieselError> {
    use crate::schema::solutions::dsl::{game_id as gid, solutions, user_id};

    let result = solutions
        .filter(user_id.eq(current_user.id).and(gid.eq(game_id)))
        .get_results::<SolutionEntity>(connection)?
        .into_iter()
        .map(|solution| {
            (
                Vector::new(solution.x1, solution.y1),
                Vector::new(solution.x2, solution.y2),
            )
        })
        .map(SolutionDTO::from)
        .collect();
    Ok(result)
}

/// Return all solutions of the game. Only if the current user is the owner of the game
pub fn get_all_solutions(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<Vec<SolutionDTO>> {
    use crate::schema::games::dsl;

    if dsl::games
        .filter(dsl::id.eq(game_id).and(dsl::owner_id.eq(current_user.id)))
        .count()
        .get_result::<i64>(connection)?
        == 0
    {
        Err("Game not found")?;
    }
    let result = get_current_puzzle_solutions(connection, game_id)
        .ok_or("Unexpected error retrieving the game")?;
    Ok(result)
}

pub fn submit_solution(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
    solution: SolutionDTO,
) -> FieldResult<bool> {
    {
        use crate::schema::games::{self, dsl as g};

        let now = Utc::now();

        let game: GameEntity = games::table
            .filter(g::id.eq(game_id))
            .get_result(connection)?;

        if game.available_to.map(|a| a < now).unwrap_or(false) {
            Err("Game has expired")?;
        }
        if game.available_from.map(|a| now < a).unwrap_or(true) {
            Err("Game not available")?;
        }
    }

    let puzzle_solutions =
        get_current_puzzle_solutions(&connection, game_id).ok_or("Game does not exist")?;
    let result = &puzzle_solutions.iter().find(|s| **s == solution);
    if result.is_none() {
        return Ok(false);
    }
    let result = result.unwrap();
    let current_solutions =
        get_users_solutions(&connection, &current_user, game_id).map_err(|e| {
            error!("Failed to read users solutions {:?}", e);
            "Failed to fetch solutions"
        })?;
    connection.transaction::<_, DieselError, _>(|| {
        if !current_solutions.contains(&result) {
            use crate::schema::solutions::dsl;

            insert_into(dsl::solutions)
                .values(SolutionForm {
                    game_id: game_id,
                    user_id: current_user.id,
                    x1: solution.solution1.x,
                    y1: solution.solution1.y,
                    x2: solution.solution2.x,
                    y2: solution.solution2.y,
                })
                .execute(connection)?;
        }
        if current_solutions.len() + 1 == puzzle_solutions.len() {
            end_participation(connection, current_user, game_id).map_err(|e| {
                error!("Failed to end participation {:?}", e);
                DieselError::RollbackTransaction
            })?;
        }
        Ok(())
    })?;
    Ok(true)
}

fn get_current_puzzle_solutions(
    connection: &DieselConnection,
    gid: i32,
) -> Option<Vec<SolutionDTO>> {
    use crate::schema::puzzles::dsl;

    dsl::puzzles
        .filter(dsl::game_id.eq(gid))
        .select(dsl::solutions)
        .get_result(connection)
        .optional()
        .expect("Failed to read solutions")
        .map(|v: Vec<i32>| {
            v.as_slice()
                .chunks(4)
                .map(|s| {
                    debug_assert!(s.len() == 4);
                    (Vector::new(s[0], s[1]), Vector::new(s[2], s[3]))
                })
                .map(SolutionDTO::from)
                .collect()
        })
}

