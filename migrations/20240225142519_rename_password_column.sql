-- Add migration script here
ALTER TABLE users
RENAME password to password_hash;