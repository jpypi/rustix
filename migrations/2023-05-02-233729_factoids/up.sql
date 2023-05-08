-- Your SQL goes here
CREATE TYPE factoid_kind AS ENUM ('reply', 'action');

CREATE TABLE factoids (
    id SERIAL PRIMARY KEY,
    time TIMESTAMP NOT NULL,
    user_id INT REFERENCES users(id) NOT NULL,
    pattern TEXT NOT NULL,
    kind factoid_kind NOT NULL,
    value TEXT NOT NULL
);