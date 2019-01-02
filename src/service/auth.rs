use super::super::model::user::User;
use super::super::schema;
use super::config;
use super::db_client::{db_client, DieselConnection};
use diesel::prelude::*;
use oauth2::Config;
use regex::Regex;
use reqwest;
use rocket::http::Cookies;
use serde_json::Value;

pub fn client(config: &config::Config) -> Config {
    let google_client_id = config.google_client_id.clone();
    let google_client_secret = config.google_client_secret.clone();

    Config::new(
        google_client_id,
        google_client_secret,
        "https://accounts.google.com/o/oauth2/v2/auth",
        "https://www.googleapis.com/oauth2/v3/token",
    ).add_scope("https://www.googleapis.com/auth/userinfo.profile")
    .set_redirect_url(config.oauth_redirect_url.clone())
    .set_state("1234")
}

/// Extract the auth token from the cookies and
/// get the `User` using the token stored in our database
pub fn logged_in_user_from_cookie(
    connection: &DieselConnection,
    cookies: &mut Cookies,
) -> Option<User> {
    let re = Regex::new("^Bearer ").unwrap();
    cookies
        .get("Authorization")
        .map(|cookie| {
            let token = re.replace(cookie.value(), "").into_owned();
            logged_in_user(connection, &token)
        }).unwrap_or(None)
}

/// Get the `User` using the token stored in our database
pub fn logged_in_user(connection: &DieselConnection, token: &str) -> Option<User> {
    use self::schema::users::dsl::{auth_token, users};

    users
        .filter(auth_token.eq(token))
        .get_result(connection)
        .ok()
}

/// Get the User via the Google OAuth API and retrieve the corresponding `User` object or `None` if
/// it was not found
pub fn user(token: &str, config: &config::Config) -> Option<User> {
    let user_info = get_user_from_google(token);
    db_client(&config)
        .query(
            "SELECT id, name, auth_token, googleid
             FROM users
             WHERE googleid=$1",
            &[&user_info["id"].to_string()],
        ).expect("Unexpected error while retrieving user data")
        .iter()
        .map(|row| User {
            id: row.get(0),
            name: row.get(1),
            auth_token: row.get(2),
            googleid: row.get(3),
        }).next()
}

/// Retrieve a user from the Google OAuth API using a token
pub fn get_user_from_google(token: &str) -> Value {
    let client = reqwest::Client::new();
    let mut response = client
        .get(&format!("https://www.googleapis.com/plus/v1/people/me"))
        .bearer_auth(token.clone())
        .send()
        .expect("Error while getting user info");

    // TODO: check response code
    let body = response.text().expect("Error getting response body");

    serde_json::from_str(&body).expect("Failed to deserialize response")
}
