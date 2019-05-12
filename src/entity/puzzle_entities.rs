use super::super::schema::puzzles;
use crate::model::puzzle::Puzzle;
use arrayvec::ArrayVec;

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

impl From<Puzzle> for PuzzleUpdate {
    fn from(puzzle: Puzzle) -> Self {
        let (col, row) = puzzle.get_shape();
        let solutions = puzzle
            .get_solutions()
            .into_iter()
            .map(|(v1, v2)| [v1.x, v1.y, v2.x, v2.y].into())
            .collect::<Vec<ArrayVec<_>>>();

        Self {
            game_table: puzzle.get_table().into_iter().collect(),
            table_columns: col as i32,
            table_rows: row as i32,
            solutions: solutions.into_iter().flatten().collect(),
            words: Some(puzzle.get_words().clone()),
        }
    }
}
