use super::super::model::solution::SolutionDTO;
use super::super::model::user::User;
use super::super::model::vector::Vector;
use super::super::service::auth::logged_in_user_from_cookie;
use super::super::service::config;
use super::super::service::db_client::db_client;
use postgres;
use postgres::rows::Rows;
use postgres::transaction::Transaction;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::{from_value, Value};
use std::collections::HashSet;

#[get("/solutions/<game_id>")]
pub fn get_solution_by_game_id(
    game_id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<Vec<SolutionDTO>>, Custom<&'static str>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::Unauthorized, "Log in first"));
    }
    let current_user = current_user.unwrap();
    let result = db_client(&config)
        .query(
            "
            SELECT s.x1, s.y1, s.x2, s.y2
            FROM solutions s
            WHERE s.game_id=$1 AND s.user_id=$2
            ",
            &[&game_id, &current_user.id],
        )
        .expect("Unexpected error while reading solutions")
        .iter()
        .map(|row| {
            (
                Vector::new(row.get(0), row.get(1)),
                Vector::new(row.get(2), row.get(3)),
            )
        })
        .collect();
    Ok(Json(result))
}

/// Submit solutions for evaluation
/// Must be logged in
/// Each solution is a tuple of vectors
/// These vectors must be sorted
/// Returns if the solution is correct for each solution submitted
#[post("/solutions/<game_id>", data = "<solutions>")]
pub fn submit_solutions(
    game_id: i32,
    solutions: Json<Vec<SolutionDTO>>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<Vec<bool>>, Custom<&'static str>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::Unauthorized, "Log in first"));
    }
    let current_user = current_user.unwrap();

    let client = db_client(&config);
    let transaction = client.transaction().expect("Failed to start transaction");

    let current_solutions = get_current_solutions(&transaction, &current_user, game_id);
    let puzzle_solutions = get_current_puzzle_solutions(&transaction, game_id);
    if puzzle_solutions.is_none() {
        return Err(Custom(Status::NotFound, "Game does not exist"));
    }
    let puzzle_solutions = puzzle_solutions.unwrap();

    let result = solutions
        .iter()
        .map(|solution| {
            if current_solutions.contains(solution) {
                // Solution already submitted
                true
            } else if puzzle_solutions.contains(&solution) {
                insert_solution(&transaction, &current_user, game_id, solution)
                    .expect("Failed to insert into solutions, aborting");
                true
            } else {
                false
            }
        })
        .collect();

    let current_solutions = get_number_of_current_solutions(&transaction, &current_user, game_id);
    if current_solutions == puzzle_solutions.len() {
        // TODO: user finished
    }

    transaction
        .commit()
        .expect("Failed to commit transaction, aborting");

    Ok(Json(result))
}

fn insert_solution(
    transaction: &Transaction,
    current_user: &User,
    game_id: i32,
    solution: &SolutionDTO,
) -> Result<Rows, postgres::Error> {
    transaction.query(
        " 
        INSERT INTO solutions (user_id, game_id, x1, y1, x2, y2)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
        &[
            &current_user.id,
            &game_id,
            &solution.0.x,
            &solution.0.y,
            &solution.1.x,
            &solution.1.y,
        ],
    )
}

fn get_number_of_current_solutions(
    transaction: &Transaction,
    current_user: &User,
    game_id: i32,
) -> usize {
    transaction
        .query(
            "
            SELECT COUNT(*)
            FROM solutions s
            WHERE s.user_id=$1 AND s.game_id=$2 
            ",
            &[&current_user.id, &game_id],
        )
        .expect("Failed to read solutions")
        .iter()
        .map(|row| row.get(0))
        .map(|n: i64| n as usize)
        .next()
        .unwrap()
}

fn get_current_solutions(
    transaction: &Transaction,
    current_user: &User,
    game_id: i32,
) -> HashSet<SolutionDTO> {
    transaction
        .query(
            "
            SELECT s.x1, s.y1, s.x2, s.y2
            FROM solutions s
            WHERE s.user_id=$1 AND s.game_id=$2 
            ",
            &[&current_user.id, &game_id],
        )
        .expect("Failed to read solutions")
        .iter()
        .map(|row| {
            (
                Vector {
                    x: row.get(0),
                    y: row.get(1),
                },
                Vector {
                    x: row.get(2),
                    y: row.get(3),
                },
            )
        })
        .collect()
}

fn get_current_puzzle_solutions(
    transaction: &Transaction,
    game_id: i32,
) -> Option<HashSet<(Vector, Vector)>> {
    transaction
        .query(
            "
            SELECT g.puzzle
            FROM games g
            WHERE g.id=$1
            ",
            &[&game_id],
        )
        .expect("Failed to read games")
        .iter()
        .map(|row| {
            let mut json: Value = row.get(0);
            let json = json["solutions"].take();
            from_value(json).expect("Failed to deserialize Puzzle")
        })
        .next()
}
