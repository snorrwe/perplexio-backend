ALTER TABLE game_participations ADD COLUMN duration INTEGER NULL;
ALTER TABLE game_participations ALTER COLUMN start_time SET NOT NULL;
