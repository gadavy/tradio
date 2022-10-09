-- Add migration script here
CREATE TABLE IF NOT EXISTS radio_stations
(
	id          INTEGER   NOT NULL PRIMARY KEY AUTOINCREMENT,
	created_at  TIMESTAMP NOT NULL,
	updated_at  TIMESTAMP NOT NULL,
	external_id TEXT UNIQUE,
	name        TEXT      NOT NULL,
	url         TEXT      NOT NULL,
	codec       TEXT      NOT NULL,
	bitrate     INTEGER   NOT NULL,
	tags        TEXT      NOT NULL,
	country     TEXT      NOT NULL
);
