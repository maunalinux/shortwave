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

mod gstreamer_backend;
mod song_backend;

use std::convert::TryInto;

use glib::{Receiver, Sender};
use gstreamer_backend::GstreamerBackend;
pub use gstreamer_backend::GstreamerMessage;
use gtk::glib;
use song_backend::SongBackend;

use crate::app::Action;
use crate::settings::{settings_manager, Key};

pub struct Backend {
    pub gstreamer: GstreamerBackend,
    pub gstreamer_receiver: Option<Receiver<GstreamerMessage>>,

    pub song: SongBackend,
}

impl Backend {
    pub fn new(sender: Sender<Action>) -> Self {
        // Song backend
        let save_count: usize = settings_manager::integer(Key::RecorderSaveCount)
            .try_into()
            .unwrap();
        let song = SongBackend::new(sender.clone(), save_count);
        song.delete_songs(); // Delete old songs

        // Gstreamer backend
        let (gstreamer_sender, gstreamer_receiver) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let gstreamer_receiver = Some(gstreamer_receiver);
        let gstreamer = GstreamerBackend::new(gstreamer_sender, sender);

        Self {
            gstreamer,
            gstreamer_receiver,
            song,
        }
    }
}
