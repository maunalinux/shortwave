// Shortwave - inhibit_controller.rs
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

use std::cell::Cell;

use gtk::gio;
use gtk::prelude::*;

use crate::api::SwStation;
use crate::app::SwApplication;
use crate::audio::{Controller, PlaybackState};
use crate::ui::SwApplicationWindow;

#[derive(Debug, Default)]
pub struct InhibitController {
    cookie: Cell<u32>,
}

impl InhibitController {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Controller for InhibitController {
    fn set_station(&self, _station: SwStation) {}

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        let app = gio::Application::default()
            .unwrap()
            .downcast_ref::<SwApplication>()
            .unwrap()
            .clone();
        let window = SwApplicationWindow::default();

        if playback_state == &PlaybackState::Playing || playback_state == &PlaybackState::Loading {
            if self.cookie.get() == 0 {
                let msg = Some("Playback active");
                let cookie = app.inhibit(Some(&window), gtk::ApplicationInhibitFlags::SUSPEND, msg);
                self.cookie.set(cookie);

                debug!("Install inhibitor")
            }
        } else if self.cookie.get() != 0 {
            app.uninhibit(self.cookie.get());
            self.cookie.set(0);

            debug!("Remove inhibitor");
        }
    }

    fn set_volume(&self, _volume: f64) {}

    fn set_song_title(&self, _title: &str) {}
}
