use super::config::Config;
use diesel::pg::PgConnection;
use diesel::Connection as DC;
use postgres::{self, TlsMode};

pub type Connection = postgres::Connection;
pub type DieselConnection = PgConnection;

pub fn db_client(config: &Config) -> Connection {
    let url = &config.postgres_url;
    Connection::connect(url.clone(), TlsMode::None).expect("Failed to connect to database")
}

pub fn diesel_client(config: &Config) -> PgConnection {
    PgConnection::establish(&config.postgres_url).expect("Failed to connect to database")
}

