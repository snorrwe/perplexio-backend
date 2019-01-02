#[get("/")]
pub fn index() -> &'static str {
    "- Hello there\n- General Kenobi\n- You're a bold one"
}

/// Gets the currently logged in user or returns `Err(Custom(Status::Unauthorized...))`
/// from the calling function
/// $cookies must be mutable `Cookies` instance
/// $config must be `Config` instance
///
/// This is a convenience macro, use with caution
///
/// # Example:
/// ```
/// #[macro_use] extern crate perplexio;
/// use rocket::http::{Cookies, Status};
/// use rocket::response::status::Custom;
/// use perplexio::service::config::Config;
/// use perplexio::service::auth::logged_in_user_from_cookie;
/// use perplexio::service::db_client::diesel_client;
///
/// fn my_controller(mut cookies: Cookies, config: Config) -> Result<(), Custom<&'static str>> {
///     let client = diesel_client(&config);
///     let current_user = logged_in_user!(client, cookies);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! logged_in_user (
    ($connection: ident, $cookies: ident) => {
        {
            use rocket::http::Status;
            use rocket::response::status::Custom;
            let current_user = logged_in_user_from_cookie(& $connection, &mut $cookies);
            if current_user.is_none() {
                return Err(Custom(Status::Unauthorized, "Log in first"));
            }
            current_user.unwrap()
        };
    }
);

pub mod games;
pub mod participations;
pub mod solutions;
pub mod users;
