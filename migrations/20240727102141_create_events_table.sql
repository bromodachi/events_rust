-- Add migration script here
CREATE TABLE events(
    id BIGINT PRIMARY KEY NOT NULL,
    external_id VARCHAR(255) NOT NULL
);
CREATE INDEX events_external_id_idx ON events (external_id);

CREATE TABLE events_key_value(
    events_id BIGINT,
    key VARCHAR(255) NOT NULL,
    value VARCHAR(255)
);
CREATE INDEX events_id_and_key_idx ON events_key_value (events_id, key);
CREATE INDEX events_id_idx ON events_key_value (events_id);