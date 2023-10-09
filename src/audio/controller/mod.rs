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

mod gcast_controller;
mod inhibit_controller;
mod mini_controller;
#[cfg(unix)]
mod mpris_controller;
mod sidebar_controller;
mod toolbar_controller;

pub use gcast_controller::GCastController;
pub use inhibit_controller::InhibitController;
pub use mini_controller::MiniController;
#[cfg(unix)]
pub use mpris_controller::MprisController;
pub use sidebar_controller::SidebarController;
pub use toolbar_controller::ToolbarController;

use crate::api::SwStation;
use crate::audio::PlaybackState;

pub trait Controller {
    fn set_station(&self, station: SwStation);
    fn set_playback_state(&self, playback_state: &PlaybackState);
    fn set_volume(&self, volume: f64);
    fn set_song_title(&self, title: &str);
}
