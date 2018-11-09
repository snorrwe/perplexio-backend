-- Your SQL goes here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    googleid VARCHAR NOT NULL UNIQUE,
    auth_token VARCHAR UNIQUE
)
