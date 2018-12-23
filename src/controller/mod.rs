pub mod games;
pub mod users;
pub mod solutions;
pub mod participations;

#[get("/")]
pub fn index() -> &'static str {
    "- Hello there\n- General Kenobi"
}

