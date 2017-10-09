-- Your SQL goes here
CREATE TABLE votes (
    user_id int REFERENCES users (id),
    voteable_id int REFERENCES voteables (id),
    up int NOT NULL DEFAULT 0,
    down int NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, voteable_id)
);
