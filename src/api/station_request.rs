// Shortwave - station_request.rs
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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct StationRequest {
    pub name: Option<String>,
    pub name_exact: Option<bool>,
    pub country: Option<String>,
    pub country_exact: Option<bool>,
    pub state: Option<String>,
    pub state_exact: Option<bool>,
    pub language: Option<String>,
    pub language_exact: Option<bool>,
    pub tag: Option<String>,
    pub tag_exact: Option<bool>,
    pub bitrate_min: Option<u32>,
    pub bitrate_max: Option<u32>,
    pub order: Option<String>,
    pub reverse: Option<bool>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub hidebroken: Option<bool>,
}

impl StationRequest {
    pub fn search_for_name(name: Option<String>, limit: u32) -> Self {
        Self {
            name,
            limit: Some(limit),
            hidebroken: Some(true),
            order: Some(String::from("votes")),
            reverse: Some(true),
            ..Self::default()
        }
    }

    pub fn url_encode(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}
