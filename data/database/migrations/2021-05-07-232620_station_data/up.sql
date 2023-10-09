CREATE TABLE librarytmp (
    uuid TEXT NOT NULL PRIMARY KEY,
    is_local BOOLEAN NOT NULL DEFAULT FALSE,
    data TEXT
);

INSERT INTO librarytmp (uuid)
    SELECT stationuuid FROM library;

DROP TABLE library;
ALTER TABLE librarytmp RENAME TO library;
