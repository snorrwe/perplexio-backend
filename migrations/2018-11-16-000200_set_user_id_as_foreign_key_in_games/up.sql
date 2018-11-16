ALTER TABLE games
ADD CONSTRAINT fk_user
FOREIGN KEY (owner_id)
REFERENCES users(id);
