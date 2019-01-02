use super::super::schema::solutions;
pub use super::vector::Vector;

pub type SolutionDTO = (Vector, Vector);

#[derive(Insertable)]
#[table_name = "solutions"]
pub struct SolutionForm {
    pub user_id: i32,
    pub game_id: i32,
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

#[derive(Queryable)]
pub struct SolutionEntity {
    pub id: i32,
    pub user_id: i32,
    pub game_id: i32,
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}
