CREATE TABLE librarytmp (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	stationuuid	TEXT NOT NULL
);

INSERT INTO librarytmp (stationuuid)
    SELECT uuid FROM library
    WHERE is_local IS FALSE;

DROP TABLE library;
ALTER TABLE librarytmp RENAME TO library;
