CREATE TABLE librarytemp (
	id 		integer NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	station_id 	integer NOT NULL
);

INSERT INTO librarytemp(id, station_id)
SELECT id,stationuuid
FROM library;

DROP TABLE library;

ALTER TABLE librarytemp
RENAME TO library;
