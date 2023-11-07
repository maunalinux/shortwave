// Shortwave - mod.rs
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

pub mod pages;

pub mod about_window;
mod create_station_dialog;
pub mod featured_carousel;
mod song_listbox;
mod song_row;
mod station_dialog;
mod station_favicon;
mod station_flowbox;
mod station_row;
mod streaming_dialog;
mod window;

pub use create_station_dialog::SwCreateStationDialog;
pub use featured_carousel::SwFeaturedCarousel;
pub use song_listbox::SongListBox;
pub use song_row::SwSongRow;
pub use station_dialog::SwStationDialog;
pub use station_favicon::{FaviconSize, StationFavicon};
pub use station_flowbox::SwStationFlowBox;
pub use station_row::SwStationRow;
pub use streaming_dialog::SwStreamingDialog;
pub use window::{SwApplicationWindow, SwView};
