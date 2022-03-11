-- Add migration script here
CREATE TABLE gold_requests (
    user_id INTEGER NOT NULL,
    amount INTEGER NOT NULL
);