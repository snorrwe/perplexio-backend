#[derive(Clone, Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub googleid: String,
    pub auth_token: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub name: String,
}
