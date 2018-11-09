#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub auth_token: Option<String>,
    pub googleid: String,
}

