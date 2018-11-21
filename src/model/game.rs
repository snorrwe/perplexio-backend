use serde_json::Value;

#[derive(Serialize)]
pub struct GameId {
    pub id: i32,
    pub name: String,
    pub owner: String,
}

#[derive(Deserialize, Debug)]
pub struct GameSubmission {
    pub name: String,
    pub words: Vec<String>,
}

#[derive(Serialize)]
pub struct GameDTO {
    pub id: GameId,
    pub table: Value,
    pub is_owner: bool,
}
