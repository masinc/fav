-- Add migration script here
CREATE TABLE favorites
(
    id         INTEGER PRIMARY KEY NOT NULL,
    path       TEXT UNIQUE         NOT NULL,
    created_at TEXT                NOT NULL DEFAULT (current_timestamp)
);

CREATE INDEX favorites_path ON favorites (path);

CREATE TABLE aliases
(
    id          INTEGER PRIMARY KEY,
    favorite_id INTEGER,
    name        TEXT UNIQUE NOT NULL,
    created_at  TEXT        NOT NULL DEFAULT (current_timestamp),
    FOREIGN KEY (favorite_id)
        REFERENCES favorites (id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

CREATE INDEX aliases_name ON aliases (name);
