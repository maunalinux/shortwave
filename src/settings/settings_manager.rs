// Shortwave - settings_manager.rs
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

use gio::prelude::*;
use gtk::{gio, glib};

use crate::config;
use crate::settings::Key;

pub fn create_action(key: Key) -> gio::Action {
    let settings = settings();
    settings.create_action(&key.to_string())
}

pub fn settings() -> gio::Settings {
    gio::Settings::new(config::APP_ID)
}

pub fn bind_property<P: IsA<glib::Object>>(key: Key, object: &P, property: &str) {
    let settings = settings();
    settings
        .bind(key.to_string().as_str(), object, property)
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();
}

#[allow(dead_code)]
pub fn string(key: Key) -> String {
    let settings = settings();
    settings.string(&key.to_string()).to_string()
}

#[allow(dead_code)]
pub fn set_string(key: Key, value: String) {
    let settings = settings();
    settings.set_string(&key.to_string(), &value).unwrap();
}

#[allow(dead_code)]
pub fn boolean(key: Key) -> bool {
    let settings = settings();
    settings.boolean(&key.to_string())
}

#[allow(dead_code)]
pub fn set_boolean(key: Key, value: bool) {
    let settings = settings();
    settings.set_boolean(&key.to_string(), value).unwrap();
}

#[allow(dead_code)]
pub fn integer(key: Key) -> i32 {
    let settings = settings();
    settings.int(&key.to_string())
}

#[allow(dead_code)]
pub fn set_integer(key: Key, value: i32) {
    let settings = settings();
    settings.set_int(&key.to_string(), value).unwrap();
}

#[allow(dead_code)]
pub fn double(key: Key) -> f64 {
    let settings = settings();
    settings.double(&key.to_string())
}

#[allow(dead_code)]
pub fn set_double(key: Key, value: f64) {
    let settings = settings();
    settings.set_double(&key.to_string(), value).unwrap();
}
