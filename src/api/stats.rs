// Shortwave - stats.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
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

#[derive(Default, Debug, Clone, PartialEq, Eq, serde_derive::Deserialize)]
pub struct Stats {
    pub supported_version: i64,
    pub software_version: String,
    pub status: String,
    pub stations: i64,
    pub stations_broken: i64,
    pub tags: i64,
    pub clicks_last_hour: i64,
    pub clicks_last_day: i64,
    pub languages: i64,
    pub countries: i64,
}
