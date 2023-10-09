// Shortwave - about_window.rs
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

use gtk::prelude::*;

use crate::config;
use crate::i18n::*;
use crate::ui::SwApplicationWindow;

pub fn show(parent: &SwApplicationWindow) {
    let vcs_tag = format!("Git Commit: {}", config::VCS_TAG);
    let version = match config::PROFILE {
        "development" => format!("{}-devel", config::VERSION),
        _ => config::VERSION.to_string(),
    };

    let window = adw::AboutWindow::new();
    window.set_transient_for(Some(parent));
    window.set_application_icon(config::APP_ID);
    window.set_application_name(config::NAME);
    window.set_designers(&["Tobias Bernard"]);
    window.set_comments(&i18n("Listen to internet radio"));
    window.set_copyright("© 2019-2023 Felix Häcker");
    window.set_debug_info(&vcs_tag);
    window.set_developer_name("Felix Häcker");
    window.set_developers(&[
        "Felix Häcker <haeckerfelix@gnome.org>",
        "Maximiliano Sandoval <msandova@gnome.org>",
        "Elias Projahn",
    ]);
    window.set_issue_url("https://gitlab.gnome.org/World/Shortwave/-/issues");
    window.set_license_type(gtk::License::Gpl30);
    window.set_translator_credits(&i18n("translator-credits"));
    window.set_version(&version);
    window.set_website("https://gitlab.gnome.org/World/Shortwave");

    window.show();
}
