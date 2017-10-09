-- Your SQL goes here
CREATE TABLE quotes (
    id SERIAL PRIMARY KEY,
    quoter_id INT REFERENCES users (id),
    time TIMESTAMP,
    value TEXT
);
