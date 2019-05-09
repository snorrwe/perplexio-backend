use super::super::schema::solutions;
pub use super::vector::Vector;

#[derive(Debug, Clone, GraphQLObject, Eq, PartialEq)]
pub struct SolutionDTO {
    pub solution1: Vector,
    pub solution2: Vector,
}

impl SolutionDTO {
    pub fn new(s1: Vector, s2: Vector) -> Self {
        Self {
            solution1: s1,
            solution2: s2,
        }
    }
}

impl From<(Vector, Vector)> for SolutionDTO {
    fn from(p: (Vector, Vector)) -> Self {
        Self {
            solution1: p.0,
            solution2: p.1,
        }
    }
}

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
