use super::super::model::game::{GameDTO, GameId, GameSubmission};
use super::super::model::puzzle::Puzzle;
use super::super::service::auth;
use super::super::service::config;
use super::super::service::db_client::db_client;
use rocket::http::Cookies;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/games")]
pub fn get_games(config: State<config::Config>) -> Json<Vec<GameId>> {
    let result = db_client(&config)
        .query(
            "
            SELECT g.id, g.name, u.name
            FROM games g
            JOIN users u
            ON g.owner_id=u.id
            ",
            &[],
        )
        .expect("Unexpected error while reading games")
        .iter()
        .map(|row| GameId {
            id: row.get(0),
            name: row.get(1),
            owner: row.get(2),
        })
        .collect();
    Json(result)
}

#[get("/game/<id>")]
pub fn get_game(id: i32, config: State<config::Config>) -> Option<Json<GameDTO>> {
    let conn = db_client(&config);
    conn.query(
        "
    SELECT g.id, g.name, u.name, g.puzzle
    FROM games g
    JOIN users u ON g.owner_id=u.id
    WHERE g.id=$1
    ",
        &[&id],
    )
    .expect("Failed to read games")
    .iter()
    .map(|row| {
        let mut table: Value = row.get(3);
        // TODO only pass solution if owner or expired
        table["solution"].take(); // Remove the solution field
        let game = GameDTO {
            id: GameId {
                id: row.get(0),
                name: row.get(1),
                owner: row.get(2),
            },
            table: table,
        };
        Json(game)
    })
    .next()
}

#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> String {
    let conn = db_client(&config);

    let current_user = auth::logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return "Log in first".to_string();
    }
    let current_user = current_user.unwrap();

    let transaction = conn.transaction().unwrap();

    let puzzle = Puzzle::from_words(game.words.clone(), 500).expect("Failed to create puzzle");
    let (columns, rows) = puzzle.get_shape();
    let json = json!({
        "table": puzzle.get_table(),
        "columns":  columns,
        "rows":  rows,
        "solution": puzzle.get_solutions()
    });

    let res = transaction.query(
        "INSERT INTO games (name, owner_id, puzzle)
        VALUES ($1, $2, $3)
        RETURNING id;",
        &[&game.name, &current_user.id, &json],
    );

    let id: i32 = res
        .expect("Failed to create new game")
        .iter()
        .map(|row| row.get(0))
        .next()
        .unwrap();

    transaction.commit().unwrap();

    id.to_string()
}
