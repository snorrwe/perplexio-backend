CREATE TABLE puzzles(
    game_id INTEGER REFERENCES games(id) NOT NULL PRIMARY KEY,

    game_table VARCHAR NOT NULL,
    table_columns INTEGER NOT NULL,
    table_rows INTEGER NOT NULL,
    solutions INTEGER[4][] NOT NULL,
    words TEXT[] NOT NULL
);

ALTER TABLE games DROP COLUMN puzzle;
ALTER TABLE games DROP COLUMN words;
