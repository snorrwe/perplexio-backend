use super::super::fairing::DieselConnection;
use super::super::model::user::UserInfo;
use super::super::service::auth;
use super::super::service::config::Config;
use super::super::service::db_client::db_client;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::Absolute;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::json::Json;

#[get("/login?<code>")]
pub fn login(
    code: Option<String>,
    mut cookies: Cookies,
    config: State<Config>,
    connection: DieselConnection,
) -> Redirect {
    if code.is_none() {
        return get_login_redirect_by_cookie(cookies, config, connection);
    }
    let client = auth::client(&config);
    let token = client.exchange_code(code.unwrap()).unwrap();
    match auth::user(&token.access_token, &config) {
        Some(u) => {
            db_client(&Config::get())
                .execute(
                    "
                    UPDATE users
                    SET auth_token=$1
                    WHERE googleid=$2
                    ",
                    &[&token.access_token, &u.googleid],
                )
                .unwrap();
            add_auth_cookies(&token.access_token, &mut cookies);
            get_login_redirect(&config)
        }
        None => register(token.access_token, cookies, config),
    }
}

fn get_login_redirect_by_cookie(
    mut cookies: Cookies,
    config: State<Config>,
    connection: DieselConnection,
) -> Redirect {
    if auth::logged_in_user_from_cookie(&connection, &mut cookies).is_some() {
        get_login_redirect(&config)
    } else {
        let uri = auth::client(&config).authorize_url().to_string();
        Redirect::to(uri)
    }
}

fn get_login_redirect(config: &Config) -> Redirect {
    if let Some(url) = &config.on_login_redirect {
        let url: Absolute = Absolute::parse(url).unwrap();
        Redirect::to(url.into_owned())
    } else {
        let uri = uri!(user_info);
        Redirect::to(uri)
    }
}

#[get("/register?<token>")]
pub fn register(token: String, mut cookies: Cookies, config: State<Config>) -> Redirect {
    let user_info = auth::get_user_from_google(&token);

    db_client(&config)
        .query(
            "
            INSERT INTO users (name, auth_token, googleid)
            VALUES ($1, $2, $3)
            ",
            &[
                &user_info["displayName"].to_string(),
                &token,
                &user_info["id"].to_string(),
            ],
        )
        .expect("Failed to insert new user");
    add_auth_cookies(&token, &mut cookies);
    get_login_redirect(&config)
}

fn add_auth_cookies(token: &String, cookies: &mut Cookies) {
    let auth_cookie = Cookie::build("Authorization", format!("Bearer {}", token))
        .path("/")
        .finish();
    cookies.add(auth_cookie);
}
