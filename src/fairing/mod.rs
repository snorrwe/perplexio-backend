use actix::{Actor, SyncContext};
use diesel::pg::PgConnection;
use std::ops::Deref;

pub struct DieselConnection(pub PgConnection);

impl Deref for DieselConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Actor for DieselConnection {
    type Context = SyncContext<Self>;
}
