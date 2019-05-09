use super::super::schema::puzzles;

#[derive(Queryable)]
pub struct PuzzleEntity {
    pub game_id: i32,
    pub game_table: String,
    pub table_columns: i32,
    pub table_rows: i32,
    pub solutions: Vec<i32>,
    pub words: Vec<String>,
}

#[derive(Insertable)]
#[table_name = "puzzles"]
pub struct PuzzleInsert<'a> {
    pub game_id: i32,
    pub game_table: String,
    pub table_columns: i32,
    pub table_rows: i32,
    pub solutions: Vec<i32>,
    pub words: &'a Vec<String>,
}

#[derive(AsChangeset)]
#[table_name = "puzzles"]
pub struct PuzzleUpdate {
    pub game_table: String,
    pub table_columns: i32,
    pub table_rows: i32,
    pub solutions: Vec<i32>,
    pub words: Option<Vec<String>>,
}
