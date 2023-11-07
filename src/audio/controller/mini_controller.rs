// Shortwave - mini_controller.rs
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

use std::cell::RefCell;
use std::rc::Rc;

use glib::{clone, Sender};
use gtk::glib;
use gtk::prelude::*;

use crate::api::SwStation;
use crate::app::Action;
use crate::audio::{Controller, PlaybackState};

pub struct MiniController {
    pub widget: gtk::Box,
    sender: Sender<Action>,
    station: Rc<RefCell<Option<SwStation>>>,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    loading_button: gtk::Button,
    volume_button: gtk::VolumeButton,
    volume_signal_id: glib::signal::SignalHandlerId,
}

impl MiniController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/mini_controller.ui");
        get_widget!(builder, gtk::Box, mini_controller);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Revealer, subtitle_revealer);
        get_widget!(builder, gtk::Stack, playback_button_stack);
        get_widget!(builder, gtk::Button, start_playback_button);
        get_widget!(builder, gtk::Button, stop_playback_button);
        get_widget!(builder, gtk::Button, loading_button);
        get_widget!(builder, gtk::VolumeButton, volume_button);

        // volume_button | We need the volume_signal_id later to block the signal
        let volume_signal_id =
            volume_button.connect_value_changed(clone!(@strong sender => move |_, value| {
                send!(sender, Action::PlaybackSetVolume(value));
            }));

        let station = Rc::new(RefCell::new(None));

        let controller = Self {
            widget: mini_controller,
            sender,
            station,
            title_label,
            subtitle_label,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            loading_button,
            volume_button,
            volume_signal_id,
        };

        controller.setup_signals();
        controller
    }

    fn setup_signals(&self) {
        // start_playback_button
        self.start_playback_button.connect_clicked(
            clone!(@strong self.sender as sender => move |_| {
                send!(sender, Action::PlaybackSet(true));
            }),
        );

        // stop_playback_button
        self.stop_playback_button.connect_clicked(
            clone!(@strong self.sender as sender => move |_| {
                send!(sender, Action::PlaybackSet(false));
            }),
        );

        // loading_button
        self.loading_button
            .connect_clicked(clone!(@strong self.sender as sender => move |_| {
                send!(sender, Action::PlaybackSet(false));
            }));
    }
}

impl Controller for MiniController {
    fn set_station(&self, station: SwStation) {
        self.title_label.set_text(&station.metadata().name);
        self.title_label
            .set_tooltip_text(Some(station.metadata().name.as_str()));
        *self.station.borrow_mut() = Some(station);

        self.subtitle_revealer.set_reveal_child(false);
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        let child_name = match playback_state {
            PlaybackState::Playing => "stop_playback",
            PlaybackState::Stopped => "start_playback",
            PlaybackState::Loading => "loading",
            PlaybackState::Failure(_) => "start_playback",
        };
        self.playback_button_stack
            .set_visible_child_name(child_name);
    }

    fn set_volume(&self, volume: f64) {
        // We need to block the signal, otherwise we risk creating a endless loop
        glib::signal::signal_handler_block(&self.volume_button, &self.volume_signal_id);
        self.volume_button.set_value(volume);
        glib::signal::signal_handler_unblock(&self.volume_button, &self.volume_signal_id);
    }

    fn set_song_title(&self, title: &str) {
        if !title.is_empty() {
            self.subtitle_label.set_text(title);
            self.subtitle_label.set_tooltip_text(Some(title));
            self.subtitle_revealer.set_reveal_child(true);
        } else {
            self.subtitle_label.set_text("");
            self.subtitle_label.set_tooltip_text(None);
            self.subtitle_revealer.set_reveal_child(false);
        }
    }
}
