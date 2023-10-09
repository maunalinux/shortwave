// Shortwave - main.rs
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

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate gtk_macros;

mod api;
mod audio;
mod database;
mod model;
mod settings;
mod ui;

mod app;
#[rustfmt::skip]
mod config;
mod i18n;
mod path;

use std::env;

use gettextrs::*;
use gtk::{gio, glib};

use crate::app::SwApplication;

fn main() -> glib::ExitCode {
    // Initialize logger
    pretty_env_logger::init();

    // Initialize paths
    path::init().expect("Unable to create paths.");

    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    // Initialize Gstreamer
    gstreamer::init().expect("Failed to initialize Gstreamer");

    // Initialize variables
    glib::set_application_name(config::NAME);
    gtk::Window::set_default_icon_name(config::APP_ID);
    env::set_var("PULSE_PROP_application.icon_name", config::APP_ID);
    env::set_var("PULSE_PROP_application.metadata().name", config::NAME);

    // Setup translations
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::PKGNAME, config::LOCALEDIR).unwrap();
    textdomain(config::PKGNAME).unwrap();

    // Load app resources
    let path = &format!(
        "{}/{}/{}.gresource",
        config::DATADIR,
        config::PKGNAME,
        config::APP_ID
    );
    let res = gio::Resource::load(path).expect("Could not load resources");
    gio::resources_register(&res);

    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire().unwrap();

    // Run app itself
    SwApplication::run()
}
