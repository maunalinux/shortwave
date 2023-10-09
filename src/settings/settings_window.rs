// Shortwave - settings_window.rs
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

use adw::PreferencesWindow;
use gtk::prelude::*;

use crate::settings::{settings_manager, Key};

pub struct SettingsWindow {
    pub widget: PreferencesWindow,

    builder: gtk::Builder,
}

impl SettingsWindow {
    pub fn new(window: &gtk::Window) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/settings_window.ui");
        get_widget!(builder, PreferencesWindow, settings_window);

        settings_window.set_transient_for(Some(window));

        let window = Self {
            widget: settings_window,
            builder,
        };

        window.setup_signals();
        window
    }

    pub fn show(&self) {
        self.widget.set_visible(true);
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Switch, show_notifications_button);
        settings_manager::bind_property(Key::Notifications, &show_notifications_button, "active");
    }
}
