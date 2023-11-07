// Shortwave - toolbar_controller.rs
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

use futures_util::future::FutureExt;
use glib::{clone, Sender};
use gtk::glib;
use gtk::prelude::*;

use crate::api::{FaviconDownloader, SwStation};
use crate::app::Action;
use crate::audio::{Controller, PlaybackState};
use crate::ui::{FaviconSize, StationFavicon, SwApplicationWindow, SwView};

pub struct ToolbarController {
    pub widget: gtk::Box,
    sender: Sender<Action>,
    station: Rc<RefCell<Option<SwStation>>>,

    station_favicon: Rc<StationFavicon>,
    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    action_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    loading_button: gtk::Button,
    toolbox_gesture: gtk::GestureClick,
}

impl ToolbarController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/toolbar_controller.ui");
        get_widget!(builder, gtk::Box, toolbar_controller);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Revealer, subtitle_revealer);
        get_widget!(builder, gtk::Revealer, action_revealer);
        get_widget!(builder, gtk::Stack, playback_button_stack);
        get_widget!(builder, gtk::Button, start_playback_button);
        get_widget!(builder, gtk::Button, stop_playback_button);
        get_widget!(builder, gtk::Button, loading_button);
        get_widget!(builder, gtk::GestureClick, toolbox_gesture);

        let station = Rc::new(RefCell::new(None));

        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = Rc::new(StationFavicon::new(FaviconSize::Mini));
        favicon_box.append(&station_favicon.widget);

        let controller = Self {
            widget: toolbar_controller,
            sender,
            station,
            station_favicon,
            title_label,
            subtitle_label,
            action_revealer,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            loading_button,
            toolbox_gesture,
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

        // show_player_button
        self.toolbox_gesture.connect_pressed(
            clone!(@strong self.sender as sender => move |_, _, _, _| {
                SwApplicationWindow::default().set_view(SwView::Player);
            }),
        );
    }
}

impl Controller for ToolbarController {
    fn set_station(&self, station: SwStation) {
        self.action_revealer.set_reveal_child(true);
        self.title_label.set_text(&station.metadata().name);
        self.title_label
            .set_tooltip_text(Some(station.metadata().name.as_str()));
        *self.station.borrow_mut() = Some(station.clone());

        // Download & set icon

        let station_favicon = self.station_favicon.clone();

        if let Some(pixbuf) = station.favicon() {
            station_favicon.set_pixbuf(&pixbuf);
        } else if let Some(favicon) = station.metadata().favicon {
            let fut =
                FaviconDownloader::download(favicon, FaviconSize::Mini as i32).map(move |pixbuf| {
                    if let Ok(pixbuf) = pixbuf {
                        station_favicon.set_pixbuf(&pixbuf)
                    }
                });
            spawn!(fut);
        }

        // reset everything else
        self.station_favicon.reset();
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
            .set_visible_child_name(child_name)
    }

    fn set_volume(&self, _volume: f64) {
        // We don't have to do anything here.
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
