-- Add migration script here
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);