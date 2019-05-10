#[derive(Clone, Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub googleid: String,
    pub auth_token: Option<String>,
}

#[derive(Clone, Debug, GraphQLObject)]
pub struct UserInfo {
    pub name: String,
}

impl From<User> for UserInfo {
    fn from(u: User)->Self{
        Self{
            name: u.name
        }
    }
}
