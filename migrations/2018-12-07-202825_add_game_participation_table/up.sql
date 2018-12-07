CREATE TABLE game_participations(
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    game_id INTEGER REFERENCES games(id) NOT NULL,
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    CONSTRAINT UC_COMPLETION UNIQUE (user_id, game_id)
)
