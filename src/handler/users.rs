use super::super::fairing::DieselConnection;
use super::super::service::auth;
use super::super::service::config::Config;
use super::super::service::db_client::db_client;
use actix_web::http::Cookie;

type Cookies = [Cookie<'static>];

pub fn login(
    _code: Option<String>,
    _cookies: &Cookies,
    _config: &Config,
    _connection: &DieselConnection,
) -> i32 {
    unimplemented!()
    // if code.is_none() {
    //     return get_login_redirect_by_cookie(cookies, config, connection);
    // }
    // let client = auth::client(&config);
    // let token = client.exchange_code(code.unwrap()).unwrap();
    // match auth::user(&token.access_token, &config) {
    //     Some(u) => {
    //         db_client(&Config::get())
    //             .execute(
    //                 "
    //                 UPDATE users
    //                 SET auth_token=$1
    //                 WHERE googleid=$2
    //                 ",
    //                 &[&token.access_token, &u.googleid],
    //             )
    //             .unwrap();
    //         add_auth_cookies(&token.access_token, &mut cookies);
    //         get_login_redirect(&config)
    //     }
    //     None => register(token.access_token, cookies, config),
    // }
}

fn get_login_redirect_by_cookie(
    _cookies: &Cookies,
    _config: &Config,
    _connection: &DieselConnection,
) -> i32 {
    unimplemented!()
    // if auth::logged_in_user_from_cookie(&connection, &mut cookies).is_some() {
    //     get_login_redirect(&config)
    // } else {
    //     let uri = auth::client(&config).authorize_url().to_string();
    //     Redirect::to(uri)
    // }
}

// fn get_login_redirect(config: &Config) -> Redirect {
//     if let Some(url) = &config.on_login_redirect {
//         let url = Absolute::parse(url).unwrap();
//         Redirect::to(url.into_owned())
//     } else {
//         let uri = uri!(crate::handler::graphiql);
//         Redirect::to(uri)
//     }
// }

pub fn register(token: String, cookies: &mut Cookies, config: &Config) -> i32 {
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
    add_auth_cookies(&token, cookies);
    // get_login_redirect(&config)
    unimplemented!()
}

fn add_auth_cookies(_token: &String, _cookies: &mut Cookies) {
    unimplemented!()
    //     let auth_cookie = Cookie::build("Authorization", format!("Bearer {}", token))
    //         .path("/")
    //         .finish();
    //     cookies.add(auth_cookie);
}
