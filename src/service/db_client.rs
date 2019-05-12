use super::config::Config;
pub use crate::fairing::DieselConnection;
use diesel::pg::PgConnection;
use diesel::ConnectionError;
use postgres::{self, TlsMode};

pub type Connection = postgres::Connection;

pub fn db_client(config: &Config) -> Connection {
    let url = &config.postgres_url;
    Connection::connect(url.clone(), TlsMode::None).expect("Failed to connect to database")
}

pub fn diesel_client(config: &Config) -> Result<DieselConnection, ConnectionError> {
    use diesel::Connection;

    PgConnection::establish(&config.postgres_url).map(|connection| DieselConnection(connection))
}
