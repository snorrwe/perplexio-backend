use super::super::fairing::DieselConnection;
use super::super::model::participation::GameParticipation;
use super::super::model::solution::{SolutionDTO, SolutionEntity, SolutionForm};
use super::super::model::user::User;
use super::super::model::vector::Vector;
use super::super::service::auth::logged_in_user_from_cookie;
use super::participations::{end_participation, get_participation_inner, insert_participation};
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket_contrib::json::Json;
use serde_json::{from_value, Value};
use std::collections::HashSet;

#[get("/solutions/<game_id>")]
pub fn get_solution_by_game_id(
    game_id: i32,
    mut cookies: Cookies,
    connection: DieselConnection,
) -> Result<Json<HashSet<SolutionDTO>>, Custom<&'static str>> {
    let current_user = logged_in_user!(connection, cookies);
    let result = get_users_solutions(&connection, &current_user, game_id);
    Ok(Json(result))
}

/// Submit solutions for evaluation
/// Must be logged in
/// Each solution is a tuple of vectors
/// These tuples must be sorted
/// Returns if the solution is correct for each solution submitted
/// Ends the game participation if all solutions were found
/// Maximum 5 solutions may be submitted at a time
#[post("/solutions/<game_id>", data = "<solutions>")]
pub fn submit_solutions(
    game_id: i32,
    solutions: Json<Vec<SolutionDTO>>,
    mut cookies: Cookies,
    connection: DieselConnection,
) -> Result<Json<Vec<bool>>, Custom<&'static str>> {
    if solutions.len() > 5 {
        return Err(Custom(
            Status::BadRequest,
            "Too many solutions for a single request",
        ));
    }
    let current_user = logged_in_user!(connection, cookies);
    let puzzle_solutions = get_current_puzzle_solutions(&connection, game_id);
    if puzzle_solutions.is_none() {
        return Err(Custom(Status::NotFound, "Game does not exist"));
    }
    let puzzle_solutions = puzzle_solutions
        .unwrap()
        .iter()
        .map(|x| *x)
        .collect::<HashSet<SolutionDTO>>();
    let mut result = Err(Custom(
        Status::InternalServerError,
        "Submit solutions could not finish",
    ));
    connection
        .transaction::<_, DieselError, _>(|| {
            let current_solutions = get_users_solutions(&connection, &current_user, game_id);
            let results = solutions
                .iter()
                .map(|solution: &SolutionDTO| {
                    if current_solutions.contains(&solution) {
                        true
                    } else if puzzle_solutions.contains(&solution) {
                        insert_solution(&connection, &current_user, game_id, &solution)
                            .expect("Failed to insert solution");
                        true
                    } else {
                        false
                    }
                })
                .collect();
            result = Ok(Json(results));
            Ok(())
        })
        .expect("Failed to commit transaction");
    let current_solutions = get_number_of_current_solutions(&connection, &current_user, game_id);
    if current_solutions == puzzle_solutions.len() {
        finish_game(&current_user, game_id, &connection);
    }
    result
}

fn finish_game(current_user: &User, game_id: i32, connection: &DieselConnection) {
    let participation = get_participation_inner(current_user, game_id, connection);
    if let Some(participation) = participation {
        if participation.end_time.is_none() {
            end_participation(connection, current_user, game_id, None)
                .expect("Failed to update participation");
        }
    } else {
        insert_participation(
            GameParticipation {
                user_id: current_user.id,
                game_id: game_id,
                start_time: Utc::now(),
                end_time: Some(Utc::now()),
            },
            &connection,
        );
    }
}

fn insert_solution(
    connection: &DieselConnection,
    current_user: &User,
    gid: i32,
    solution: &SolutionDTO,
) -> Result<SolutionEntity, DieselError> {
    use super::super::schema::solutions::dsl::*;

    insert_into(solutions)
        .values(SolutionForm {
            game_id: gid,
            user_id: current_user.id,
            x1: solution.0.x,
            y1: solution.0.y,
            x2: solution.1.x,
            y2: solution.1.y,
        })
        .get_result(&connection.0)
}

fn get_number_of_current_solutions(
    connection: &DieselConnection,
    current_user: &User,
    gid: i32,
) -> usize {
    use super::super::schema::solutions::dsl::*;

    solutions
        .filter(user_id.eq(current_user.id).and(game_id.eq(gid)))
        .count()
        .first::<i64>(&connection.0)
        .expect("Failed to read solutions") as usize
}

/// Return all solutions submitted for a game by the user
pub fn get_users_solutions(
    client: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> HashSet<SolutionDTO> {
    use super::super::schema::solutions::dsl::{game_id as gid, solutions, user_id};

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
        .collect()
}

fn get_current_puzzle_solutions(client: &DieselConnection, gid: i32) -> Option<Vec<SolutionDTO>> {
    use super::super::schema::games::dsl::*;

    games
        .filter(id.eq(gid))
        .select(puzzle)
        .get_result(&client.0)
        .ok()
        .map_or(None, |mut p: Value| {
            let json = p["solutions"].take();
            let solutions = from_value(json).expect("Failed to deserialize Puzzle");
            Some(solutions)
        })
}
