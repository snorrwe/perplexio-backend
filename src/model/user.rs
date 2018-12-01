#[derive(Serialize, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub auth_token: Option<String>,
    pub googleid: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct UserInfo {
    pub name: String,
}
