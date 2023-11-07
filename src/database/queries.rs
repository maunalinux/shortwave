// Shortwave - queries.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::models::StationEntry;
use super::schema::library;
use crate::database;
use crate::diesel::prelude::*;

macro_rules! connect_db {
    () => {
        database::connection::connection().get().unwrap()
    };
}

pub fn stations() -> Result<Vec<StationEntry>, diesel::result::Error> {
    let con = connect_db!();
    let entries = library::table.load::<StationEntry>(&con)?;
    Ok(entries)
}

pub fn contains_station(uuid: &str) -> Result<bool, diesel::result::Error> {
    let con = connect_db!();
    let entries = library::table
        .filter(library::uuid.eq(uuid))
        .load::<StationEntry>(&con)?;
    Ok(!entries.is_empty())
}

pub fn insert_station(entry: StationEntry) -> Result<(), diesel::result::Error> {
    let con = connect_db!();
    diesel::insert_into(library::table)
        .values(entry)
        .execute(&*con)?;
    Ok(())
}

pub fn update_station(entry: StationEntry) -> Result<(), diesel::result::Error> {
    let con = connect_db!();
    diesel::replace_into(library::table)
        .values(entry)
        .execute(&*con)?;
    Ok(())
}

pub fn delete_station(uuid: &str) -> Result<(), diesel::result::Error> {
    let con = connect_db!();
    diesel::delete(library::table.filter(library::uuid.eq(uuid))).execute(&*con)?;
    Ok(())
}
