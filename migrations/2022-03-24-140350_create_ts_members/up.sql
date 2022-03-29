-- Your SQL goes here
CREATE TABLE ts_members (
    user_id BIGINT NOT NULL,
    client_uuid TEXT NOT NULL,
    insertion_pending TINYINT NOT NULL DEFAULT 1,
    removal_pending TINYINT NOT NULL DEFAULT 0,
    instance TEXT NOT NULL,
    PRIMARY KEY (user_id, removal_pending, instance),
    UNIQUE (client_uuid, removal_pending, instance)
)
