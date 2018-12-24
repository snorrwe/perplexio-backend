use super::service::auth::logged_in_user_from_cookie;

#[get("/")]
pub fn index() -> &'static str {
    "- Hello there\n- General Kenobi"
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
///
/// fn my_controller(mut cookies: Cookies, config: Config) -> Result<(), Custom<&'static str>> {
///     let current_user = logged_in_user!(cookies, config);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! logged_in_user (
    ($cookies: ident, $config: ident) => {
        {
            use rocket::http::Status;
            use rocket::response::status::Custom;
            let current_user = logged_in_user_from_cookie(&mut $cookies, & $config);
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

