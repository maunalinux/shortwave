// Shortwave - models.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use super::schema::*;
use crate::api::SwStation;

/// Representation of a station within the database.
#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = library)]
pub struct StationEntry {
    /// Unique ID that corresponds to the RadioBrowser stationuuid for non-local
    /// stations.
    pub uuid: String,

    /// Whether this station is local.
    pub is_local: bool,

    /// Serialized station metadata. For local stations, this is mandatory.
    pub data: Option<String>,

    /// Binary blob containing an optional local station favicon.
    pub favicon: Option<Vec<u8>>,
}

impl StationEntry {
    /// Create a station entry for the station.
    pub fn for_station(station: &SwStation) -> Self {
        let metadata = station.metadata();

        let favicon = if let Some(pixbuf) = station.favicon() {
            if let Ok(data) = pixbuf.save_to_bufferv("png", &[]) {
                Some(data)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            uuid: station.uuid(),
            is_local: station.is_local(),
            data: Some(serde_json::to_string(&metadata).unwrap()),
            favicon,
        }
    }
}
