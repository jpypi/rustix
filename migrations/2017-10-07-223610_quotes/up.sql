-- Your SQL goes here
CREATE TABLE quotes (
    id SERIAL PRIMARY KEY,
    quoter_id INT REFERENCES users (id) NOT NULL,
    time TIMESTAMP NOT NULL,
    value TEXT NOT NULL
);
