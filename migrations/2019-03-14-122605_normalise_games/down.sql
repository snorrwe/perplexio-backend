DROP TABLE puzzles;
ALTER TABLE games ADD COLUMN puzzle JSON;
ALTER TABLE games ADD COLUMN words VARCHAR[];
