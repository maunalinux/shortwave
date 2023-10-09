// Shortwave - song.rs
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

use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
    pub duration: Duration,
}

impl Song {
    pub fn new(title: &str, path: PathBuf, duration: Duration) -> Self {
        debug!("Created new song: \"{}\", {:?}", title, path);

        Self {
            title: title.to_string(),
            path,
            duration,
        }
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
