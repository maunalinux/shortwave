CREATE TABLE librarytemp (
	id		INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	stationuuid	TEXT NOT NULL
);

INSERT INTO librarytemp(id, stationuuid)
SELECT id,station_id
FROM library;

DROP TABLE library;

ALTER TABLE librarytemp
RENAME TO library;
