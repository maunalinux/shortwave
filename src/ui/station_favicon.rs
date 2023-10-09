// Shortwave - station_favicon.rs
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

use gdk_pixbuf::Pixbuf;
use gtk::gdk_pixbuf;
use gtk::prelude::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FaviconSize {
    Mini = 48,
    Small = 64,
    Big = 192,
}

#[derive(Debug)]
pub struct StationFavicon {
    pub widget: gtk::Box,
    image: gtk::Image,
    stack: gtk::Stack,
}

impl StationFavicon {
    pub fn new(size: FaviconSize) -> Self {
        let builder =
            gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/station_favicon.ui");
        get_widget!(builder, gtk::Box, station_favicon);
        get_widget!(builder, gtk::Image, image);
        get_widget!(builder, gtk::Stack, stack);
        get_widget!(builder, gtk::Image, placeholder);

        image.set_size_request(size as i32, size as i32);
        placeholder.set_pixel_size((size as i32).div_euclid(2));

        Self {
            widget: station_favicon,
            image,
            stack,
        }
    }

    pub fn set_pixbuf(&self, pixbuf: &Pixbuf) {
        self.image.set_from_pixbuf(Some(pixbuf));
        self.stack.set_visible_child_name("image");
    }

    pub fn reset(&self) {
        self.stack.set_visible_child_name("placeholder");
    }
}
