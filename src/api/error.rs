// Shortwave - error.rs
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

use std::rc::Rc;

use gtk::glib;
use thiserror::Error;

#[derive(Clone, Error, Debug, glib::Boxed)]
#[boxed_type(name = "SwError")]
pub enum Error {
    #[error("Serde deserializer error: {0}")]
    Deserializer(#[from] Rc<serde_json::error::Error>),

    #[error("GLib Error: {0}")]
    GLib(#[from] glib::error::Error),

    #[error("Input/Output error: {0}")]
    Io(#[from] Rc<std::io::Error>),

    #[error("Network error: {0}")]
    Network(#[from] isahc::Error),

    #[error("Invalid station UUID: {0}")]
    InvalidStation(String),

    #[error("Unsupported url scheme")]
    UnsupportedUrlScheme,

    #[error("No radiobrowser server available")]
    NoServerAvailable,
}
