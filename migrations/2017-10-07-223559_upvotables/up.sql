-- Your SQL goes here
CREATE TABLE voteables (
    id SERIAL PRIMARY KEY,
    value varchar(255) NOT NULL,
    total_up INT NOT NULL DEFAULT 0,
    total_down INT NOT NULL DEFAULT 0
);
