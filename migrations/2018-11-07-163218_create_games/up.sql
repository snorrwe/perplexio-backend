CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    owner_id INTEGER NOT NULL,
    puzzle JSON NOT NULL,
    CONSTRAINT UC_Game UNIQUE (name, owner_id)
);
