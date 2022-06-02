-- Your SQL goes here
CREATE TABLE movie_channels(
    id BLOB NOT NULL PRIMARY KEY,
    uri TEXT NOT NULL,
    vc BIGINT NOT NULL UNIQUE,
    bot_msg BIGINT NOT NULL,
    creator BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL
)