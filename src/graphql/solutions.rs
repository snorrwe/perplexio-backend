use super::participations::end_participation;
use crate::fairing::DieselConnection;
use crate::model::solution::{SolutionDTO, SolutionEntity, SolutionForm};
use crate::model::user::User;
use crate::model::vector::Vector;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use juniper::FieldResult;

pub fn get_solution_by_game_id(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<Vec<SolutionDTO>> {
    let result = get_users_solutions(&connection, &current_user, game_id);
    Ok(result)
}

/// Return all solutions submitted for a game by the user
pub fn get_users_solutions(
    client: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> Vec<SolutionDTO> {
    use crate::schema::solutions::dsl::{game_id as gid, solutions, user_id};

    solutions
        .filter(user_id.eq(current_user.id).and(gid.eq(game_id)))
        .get_results::<SolutionEntity>(&client.0)
        .expect("Failed to get solutions")
        .iter()
        .map(|solution| {
            (
                Vector::new(solution.x1, solution.y1),
                Vector::new(solution.x2, solution.y2),
            )
        })
        .map(SolutionDTO::from)
        .collect()
}

pub fn submit_solution(
    client: &DieselConnection,
    current_user: &User,
    game_id: i32,
    solution: SolutionDTO,
) -> FieldResult<bool> {
    let puzzle_solutions =
        get_current_puzzle_solutions(&client, game_id).ok_or("Game does not exist")?;
    let result = &puzzle_solutions.iter().find(|s| **s == solution);
    if result.is_none() {
        return Ok(false);
    }
    let result = result.unwrap();
    let current_solutions = get_users_solutions(&client, &current_user, game_id);
    client.transaction::<_, DieselError, _>(|| {
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
                .execute(&client.0)?;
        }
        if current_solutions.len() + 1 == puzzle_solutions.len() {
            end_participation(client, current_user, game_id).map_err(|e| {
                error!("Failed to end participation {:?}", e);
                DieselError::RollbackTransaction
            })?;
        }
        Ok(())
    })?;
    Ok(true)
}

fn get_current_puzzle_solutions(client: &DieselConnection, gid: i32) -> Option<Vec<SolutionDTO>> {
    use crate::schema::solutions::dsl;

    dsl::solutions
        .filter(dsl::game_id.eq(gid))
        .select((dsl::x1, dsl::x2, dsl::y1, dsl::y2))
        .get_results::<(i32, i32, i32, i32)>(&client.0)
        .optional()
        .expect("Failed to read solutions")
        .map(|v| {
            v.into_iter()
                .map(|(x1, x2, y1, y2)| (Vector::new(x1, y1), Vector::new(x2, y2)))
                .map(SolutionDTO::from)
                .collect()
        })
}