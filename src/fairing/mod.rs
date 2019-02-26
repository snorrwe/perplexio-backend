use diesel::pg::PgConnection;

#[database("perplexio")]
pub struct DieselConnection(PgConnection);
