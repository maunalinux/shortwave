// Shortwave - player.rs
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

use std::cell::RefCell;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use glib::{clone, Sender};
use gtk::{gio, glib};

use crate::api::SwStation;
use crate::app::Action;
use crate::audio::backend::*;
#[cfg(unix)]
use crate::audio::controller::MprisController;
use crate::audio::controller::{
    Controller, GCastController, InhibitController, MiniController, SidebarController,
    ToolbarController,
};
use crate::audio::{GCastDevice, Song};
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::SwApplicationWindow;
use crate::{config, path};

#[rustfmt::skip]
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                             //
//  A small overview of the player/gstreamer program structure  :)                                             //
//                                                                                                             //
//   -----------------    -----------------    -------------------    ----------------    ------------------   //
//  | GCastController |  | MprisController |  | SidebarController |  | MiniController |  | SimpleController |  //
//   -----------------    -----------------    -------------------    ----------------    ------------------   //
//            |                        |                   |                      |                            //
//            \-------------------------------------------------------------------/                            //
//                                     |                                                                       //
//                              ------------       -----------    ------                                       //
//                             | Controller |     | Gstreamer |  | Song |                                      //
//                              ------------       -----------    ------                                       //
//                                    |                   \       /                                            //
//                                    |                   ---------                                            //
//                                    |                  | Backend |                                           //
//                                    |                   ---------                                            //
//                                    |                     |                                                  //
//                                    \---           -------/                                                  //
//                                        \         /                                                          //
//                                        -----------                                                          //
//                                       |  Player   |                                                         //
//                                        -----------                                                          //
//                                                                                                             //
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PlaybackState {
    Playing,
    Stopped,
    Loading,
    Failure(String),
}

pub struct Player {
    pub widget: adw::NavigationPage,
    pub toolbar_controller_widget: gtk::Box,
    pub mini_controller_widget: gtk::Box,
    controller: Vec<Box<dyn Controller>>,
    gcast_controller: Rc<GCastController>,

    backend: Rc<RefCell<Backend>>,
    current_station: RefCell<Option<SwStation>>,
    song_title: RefCell<SongTitle>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Player {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/player.ui");
        get_widget!(builder, adw::NavigationPage, player);
        let mut controller: Vec<Box<dyn Controller>> = Vec::new();

        // Sidebar Controller
        let sidebar_controller = SidebarController::new(sender.clone());
        get_widget!(builder, gtk::Box, player_box);
        player_box.prepend(&sidebar_controller.widget);
        controller.push(Box::new(sidebar_controller));

        // Toolbar Controller
        let toolbar_controller = ToolbarController::new(sender.clone());
        let toolbar_controller_widget = toolbar_controller.widget.clone();
        controller.push(Box::new(toolbar_controller));

        // Mini Controller (gets used in phone mode / bottom toolbar)
        let mini_controller = MiniController::new(sender.clone());
        let mini_controller_widget = mini_controller.widget.clone();
        controller.push(Box::new(mini_controller));

        // Mpris Controller (Only available on UNIX platforms)
        #[cfg(unix)]
        let mpris_controller = MprisController::new(sender.clone());
        #[cfg(unix)]
        controller.push(Box::new(mpris_controller));

        // Google Cast Controller
        let gcast_controller = GCastController::new(sender.clone());
        controller.push(Box::new(gcast_controller.clone()));

        // Inhibit Controller
        let inhibit_controller = InhibitController::new();
        controller.push(Box::new(inhibit_controller));

        let controller: Vec<Box<dyn Controller>> = controller;

        // Backend
        let backend = Backend::new(sender.clone());
        player_box.append(&backend.song.listbox.widget);
        let backend = Rc::new(RefCell::new(backend));

        // Current station (needed for notifications)
        let current_station = RefCell::new(None);

        // Song title -> [Current Song] - [Previous Song]
        let song_title = RefCell::new(SongTitle::new());

        let player = Rc::new(Self {
            widget: player,
            toolbar_controller_widget,
            mini_controller_widget,
            controller,
            gcast_controller,
            backend,
            current_station,
            song_title,
            builder,
            sender,
        });

        // Set volume
        let volume = settings_manager::double(Key::PlaybackVolume);
        player.set_volume(volume);

        player.clone().setup_signals();
        player
    }

    pub fn set_station(&self, station: SwStation) {
        *self.current_station.borrow_mut() = Some(station.clone());
        self.set_playback(PlaybackState::Stopped);

        for con in &*self.controller {
            con.set_station(station.clone());
        }

        // Reset song title
        self.song_title.borrow_mut().reset();

        let metadata = station.metadata();

        // We try playing from `url_resolved` first, which is the pre-resolved
        // URL from the API. However, for local stations, we don't do that, so
        // `url_resolved` will be `None`. In that case we just use `url`, which
        // can also be a potential fallback in case the API misses the resolved
        // URL for some reason.

        let mut backend = self.backend.borrow_mut();
        if let Some(url) = metadata.url_resolved {
            debug!("Start playing new URI: {}", url.to_string());
            backend.gstreamer.new_source_uri(url.as_ref());
        } else if let Some(url) = metadata.url {
            debug!("Start playing new URI: {}", url.to_string());
            backend.gstreamer.new_source_uri(url.as_ref());
        } else {
            let text = i18n("Station cannot be streamed. URL is not valid.");
            SwApplicationWindow::default().show_notification(&text);
        }
    }

    pub fn set_playback(&self, playback: PlaybackState) {
        debug!("Set playback: {:?}", playback);
        let mut backend = self.backend.borrow_mut();

        match playback {
            PlaybackState::Playing => {
                backend.gstreamer.set_state(gstreamer::State::Playing);
            }
            PlaybackState::Stopped => {
                // Discard recorded data when the stream stops
                if backend.gstreamer.is_recording() {
                    backend.gstreamer.stop_recording(true);
                }

                // Reset song title
                self.song_title.borrow_mut().reset();

                backend.gstreamer.set_state(gstreamer::State::Null);
            }
            _ => (),
        }
    }

    pub fn toggle_playback(&self) {
        if self.backend.borrow().gstreamer.state() == PlaybackState::Playing {
            self.set_playback(PlaybackState::Stopped);
        } else if self.backend.borrow().gstreamer.state() == PlaybackState::Stopped {
            self.set_playback(PlaybackState::Playing);
        }
    }

    pub fn set_volume(&self, volume: f64) {
        debug!("Set volume: {}", &volume);
        self.backend.borrow().gstreamer.set_volume(volume);

        for con in &*self.controller {
            con.set_volume(volume);
        }

        settings_manager::set_double(Key::PlaybackVolume, volume);
    }

    pub fn save_song(&self, song: Song) {
        if let Err(err) = self.backend.borrow().song.save_song(song) {
            warn!("Cannot save song: {}", err.to_string());

            let text = i18n("Cannot save song.");
            SwApplicationWindow::default().show_notification(&text);
        }
    }

    pub fn connect_to_gcast_device(&self, device: GCastDevice) {
        get_widget!(self.builder, adw::ActionRow, stream_row);
        get_widget!(self.builder, gtk::Revealer, stream_revealer);

        stream_row.set_title(&device.name);
        stream_revealer.set_reveal_child(true);

        self.gcast_controller.connect_to_device(device);
    }

    pub fn disconnect_from_gcast_device(&self) {
        get_widget!(self.builder, gtk::Revealer, stream_revealer);
        stream_revealer.set_reveal_child(false);
        self.gcast_controller.disconnect_from_device();
    }

    pub fn has_station(&self) -> bool {
        self.current_station.borrow().is_some()
    }

    fn setup_signals(self: Rc<Self>) {
        // Wait for new messages from the Gstreamer backend
        let receiver = self
            .backend
            .clone()
            .borrow_mut()
            .gstreamer_receiver
            .take()
            .unwrap();
        receiver.attach(None, clone!(@strong self as this => move |message| this.clone().process_gst_message(message)));

        // Disconnect from gcast device
        get_widget!(self.builder, gtk::Button, disconnect_button);
        disconnect_button.connect_clicked(clone!(@strong self.sender as sender => move |_| {
            send!(sender, Action::PlaybackDisconnectGCastDevice);
        }));
    }

    fn process_gst_message(&self, message: GstreamerMessage) -> glib::ControlFlow {
        match message {
            GstreamerMessage::SongTitleChanged(title) => {
                let backend = &mut self.backend.borrow_mut();
                debug!("Song title has changed to: \"{}\"", title);

                // If we're already recording something, we need to stop it first.
                if backend.gstreamer.is_recording() {
                    let threshold: i64 =
                        settings_manager::integer(Key::RecorderSongDurationThreshold)
                            .try_into()
                            .unwrap();
                    let duration: i64 = backend.gstreamer.current_recording_duration();
                    if duration > threshold {
                        backend.gstreamer.stop_recording(false);

                        let duration = Duration::from_secs(duration.try_into().unwrap());
                        let song = self
                            .song_title
                            .borrow()
                            .create_song(duration)
                            .expect("Unable to create new song");

                        backend.song.add_song(song);
                    } else {
                        debug!("Discard recorded data, song duration ({} sec) is below threshold ({} sec).", duration, threshold);
                        backend.gstreamer.stop_recording(true);
                    }
                }

                // Set new song title
                self.song_title
                    .borrow_mut()
                    .set_current_title(title.clone());
                for con in &*self.controller {
                    con.set_song_title(&title);
                }

                // Start recording new song
                // We don't start recording the "first" detected song, since it is going to be
                // incomplete
                if !self.song_title.borrow().is_first_song() {
                    backend.gstreamer.start_recording(
                        self.song_title
                            .borrow()
                            .current_path()
                            .expect("Unable to get song path"),
                    );
                } else {
                    debug!("Song will not be recorded because it may be incomplete (first song for this station).")
                }

                // Show desktop notification
                if settings_manager::boolean(Key::Notifications) {
                    self.show_song_notification();
                }
            }
            GstreamerMessage::PlaybackStateChanged(state) => {
                for con in &*self.controller {
                    con.set_playback_state(&state);
                }

                // Discard recorded data when a failure occurs,
                // since the song has not been recorded completely.
                if self.backend.borrow().gstreamer.is_recording()
                    && matches!(state, PlaybackState::Failure(_))
                {
                    self.backend.borrow_mut().gstreamer.stop_recording(true);
                }
            }
        }
        glib::ControlFlow::Continue
    }

    fn show_song_notification(&self) {
        let current_station = self.current_station.borrow().clone().unwrap();
        let notification =
            gio::Notification::new(&self.song_title.borrow().current_title().unwrap());
        notification.set_body(Some(&current_station.metadata().name));
        // notification.add_button("Record and save this song",
        // "app.record-and-save-song");

        // Icons won't work at the moment, no idea
        // current_station.favicon.map(|favicon| {
        // FaviconDownloader::get_file(&favicon).map(|file| {
        // let path = file.get_path().unwrap().to_str().unwrap().to_owned();
        // dbg!(&path);
        // let file = gio::File::new_for_path(&path);
        // let icon = gio::FileIcon::new(&file);
        // notification.set_icon(&icon);
        // })
        // });

        let app = gio::Application::default().unwrap();
        let id = format!("{}.SongNotification", config::APP_ID);
        app.send_notification(Some(&id), &notification);
    }
}

pub struct SongTitle {
    current_title: Option<String>,
    previous_title: Option<String>,
}

impl SongTitle {
    pub fn new() -> Self {
        Self {
            current_title: None,
            previous_title: None,
        }
    }

    pub fn set_current_title(&mut self, title: String) {
        if self.current_title.is_none() {
            self.current_title = Some(title);
        } else {
            self.previous_title = self.current_title.take();
            self.current_title = Some(title);
        }
    }

    pub fn current_title(&self) -> Option<String> {
        self.current_title.clone()
    }

    /// Returns song for current title
    pub fn create_song(&self, duration: Duration) -> Option<Song> {
        if let Some(title) = &self.current_title {
            let path = self
                .current_path()
                .expect("Unable to get path for current song");
            return Some(Song::new(title, path, duration));
        }
        None
    }

    /// Returns path for current title
    fn current_path(&self) -> Option<PathBuf> {
        if let Some(title) = &self.current_title {
            let title = title.to_string();
            let filename = sanitize_filename::sanitize(title.clone() + ".ogg");

            let mut path = path::CACHE.clone();
            path.push("recording");

            // Make sure that the path exists
            fs::create_dir_all(path.clone()).expect("Could not create path for recording");

            if !title.is_empty() {
                path.push(filename);
            }
            return Some(path);
        }
        None
    }

    pub fn is_first_song(&self) -> bool {
        self.previous_title.is_none()
    }

    pub fn reset(&mut self) {
        debug!("Cleared song title queue.");
        self.current_title = None;
        self.previous_title = None;
    }
}
