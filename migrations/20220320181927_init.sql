-- Add migration script here
CREATE TABLE ts (
    user_id INTEGER NOT NULL PRIMARY KEY,
    client_uuid TEXT UNIQUE,
    insertion_pending INTEGER NOT NULL,
    removal_pending INTEGER NOT NULL
)
