// Shortwave - mod.rs
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

mod backend;
mod controller;

pub use controller::{Controller, GCastController};

mod gcast_discoverer;
mod player;
mod song;

pub use gcast_discoverer::{GCastDevice, GCastDiscoverer, GCastDiscovererMessage};
pub use player::{PlaybackState, Player};
pub use song::Song;
