// Shortwave - mpris_controller.rs
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use glib::{clone, Sender};
use gtk::glib;
use gtk::prelude::*;
use mpris_player::{Metadata, MprisPlayer, OrgMprisMediaPlayer2Player, PlaybackStatus};

use crate::api::{FaviconDownloader, SwStation};
use crate::app::Action;
use crate::audio::{Controller, PlaybackState};
use crate::config;
use crate::ui::SwApplicationWindow;

pub struct MprisController {
    sender: Sender<Action>,
    mpris: Arc<MprisPlayer>,

    song_title: Cell<Option<String>>,
    station: Cell<Option<SwStation>>,
    volume: Rc<RefCell<f64>>,
}

impl MprisController {
    pub fn new(sender: Sender<Action>) -> Self {
        let mpris = MprisPlayer::new(
            config::APP_ID.to_string(),
            config::NAME.to_string(),
            config::APP_ID.to_string(),
        );
        mpris.set_can_raise(true);
        mpris.set_can_play(false);
        mpris.set_can_seek(false);
        mpris.set_can_set_fullscreen(false);
        mpris.set_can_pause(true);

        let volume = Rc::new(RefCell::new(0.0));

        let controller = Self {
            sender,
            mpris,
            song_title: Cell::new(None),
            station: Cell::new(None),
            volume,
        };

        controller.setup_signals();
        controller
    }

    fn update_metadata(&self) {
        let mut metadata = Metadata::new();

        let station = self.station.take();
        let song_title = self.song_title.take();

        if let Some(station) = station.clone() {
            station.metadata().favicon.map(|favicon| {
                FaviconDownloader::file(&favicon).map(|file| {
                    let path = format!(
                        "file://{}",
                        file.path().unwrap().to_str().unwrap().to_owned()
                    );
                    metadata.art_url = Some(path);
                })
            });
            metadata.artist = Some(vec![station.metadata().name]);
        }
        if let Some(song_title) = song_title.clone() {
            metadata.title = Some(song_title);
        }

        self.station.set(station);
        self.song_title.set(song_title);

        self.mpris.set_metadata(metadata);
    }

    fn setup_signals(&self) {
        // mpris raise
        self.mpris.connect_raise(move || {
            let window = SwApplicationWindow::default();
            window.present_with_time((glib::monotonic_time() / 1000) as u32);
        });

        // mpris play / pause
        self.mpris.connect_play_pause(
            clone!(@weak self.mpris as mpris, @strong self.sender as sender => move || {
                match mpris.get_playback_status().unwrap().as_ref() {
                    "Paused" => send!(sender, Action::PlaybackSet(true)),
                    "Stopped" => send!(sender, Action::PlaybackSet(true)),
                    _ => send!(sender, Action::PlaybackSet(false)),
                };
            }),
        );

        // mpris play
        self.mpris
            .connect_play(clone!(@strong self.sender as sender => move || {
                send!(sender, Action::PlaybackSet(true));
            }));

        // mpris stop
        self.mpris
            .connect_stop(clone!(@strong self.sender as sender => move || {
                send!(sender, Action::PlaybackSet(false));
            }));

        // mpris pause
        self.mpris
            .connect_pause(clone!(@strong self.sender as sender => move || {
                send!(sender, Action::PlaybackSet(false));
            }));

        // mpris volume
        self.mpris.connect_volume(clone!(@strong self.sender as sender, @weak self.volume as old_volume => move |new_volume| {
            // if *old_volume.borrow() != new_volume {
            if (*old_volume.borrow() - new_volume).abs() > std::f64::EPSILON {
                send!(sender, Action::PlaybackSetVolume(new_volume));
                *old_volume.borrow_mut() = new_volume;
            }
        }));
    }
}

impl Controller for MprisController {
    fn set_station(&self, station: SwStation) {
        self.station.set(Some(station));
        self.update_metadata();
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        self.mpris.set_can_play(true);

        match playback_state {
            PlaybackState::Playing => self.mpris.set_playback_status(PlaybackStatus::Playing),
            _ => self.mpris.set_playback_status(PlaybackStatus::Stopped),
        };
    }

    fn set_volume(&self, volume: f64) {
        *self.volume.borrow_mut() = volume;
        self.mpris.set_volume(volume).unwrap();
    }

    fn set_song_title(&self, title: &str) {
        self.song_title.set(Some(title.to_string()));
        self.update_metadata();
    }
}
