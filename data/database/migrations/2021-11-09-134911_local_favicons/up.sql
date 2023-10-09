CREATE TABLE librarytmp (
    uuid TEXT NOT NULL PRIMARY KEY,
    is_local BOOLEAN NOT NULL DEFAULT FALSE,
    data TEXT,
    favicon BLOB
);

INSERT INTO librarytmp (uuid, is_local, data)
    SELECT uuid, is_local, data FROM library;

DROP TABLE library;
ALTER TABLE librarytmp RENAME TO library;
