// Shortwave - error.rs
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

use gtk::glib;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serde deserializer error: {0}")]
    Deserializer(#[from] serde_json::error::Error),

    #[error("GLib Error: {0}")]
    GLib(#[from] glib::error::Error),

    #[error("Input/Output error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] isahc::Error),

    #[error("Invalid station UUID: {0}")]
    InvalidStation(String),

    #[error("Unsupported url scheme")]
    UnsupportedUrlScheme,
}
