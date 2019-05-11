use super::model::user::User;
use super::service::auth::logged_in_user_from_cookie;
use super::DieselConnection;
use std::ops::Deref;

/// Rocket Request Guard object to retreive the User before the handler code
///
#[derive(Debug)]
pub struct LoggedInUser(pub User);

#[derive(Debug)]
pub enum LoggedInUserError {
    NotFound,
    DbError,
}

impl Deref for LoggedInUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<User> for LoggedInUser {
    fn into(self) -> User {
        self.0
    }
}

// impl<'a, 'r> FromRequest<'a, 'r> for LoggedInUser {
//     type Error = LoggedInUserError;
//
//     fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
//         let mut cookies = req.cookies();
//         let connection: DieselConnection = req.guard().map_failure(|e| {
//             error!("Error retreiving db connection {:?}", e);
//             (Status::InternalServerError, LoggedInUserError::DbError)
//         })?;
//
//         let user = logged_in_user_from_cookie(&connection, &mut cookies);
//
//         match user {
//             Some(user) => Outcome::Success(Self(user)),
//             None => Outcome::Failure((Status::Unauthorized, LoggedInUserError::NotFound)),
//         }
//     }
// }
