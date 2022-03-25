-- Your SQL goes here
CREATE TABLE ts_members (
    user_id BIGINT NOT NULL,
    client_uuid TEXT NOT NULL,
    insertion_pending TINYINT NOT NULL,
    removal_pending TINYINT NOT NULL,
    PRIMARY KEY (user_id, removal_pending),
    UNIQUE (client_uuid, removal_pending)
)