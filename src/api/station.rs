// Shortwave - station.rs
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

use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk_pixbuf, glib};
use once_cell::unsync::OnceCell;

use crate::api::StationMetadata;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SwStation)]
    pub struct SwStation {
        #[property(get, set, construct_only)]
        pub uuid: OnceCell<String>,
        #[property(get, set, construct_only)]
        pub is_local: OnceCell<bool>,
        #[property(get, set, construct_only)]
        pub is_orphaned: OnceCell<bool>,
        #[property(get, set, construct_only)]
        pub metadata: OnceCell<StationMetadata>,
        #[property(get, set, construct_only)]
        pub favicon: OnceCell<Option<gdk_pixbuf::Pixbuf>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStation {
        const NAME: &'static str = "SwStation";
        type Type = super::SwStation;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SwStation {}
}

glib::wrapper! {
    pub struct SwStation(ObjectSubclass<imp::SwStation>);
}

impl SwStation {
    pub fn new(
        uuid: &str,
        is_local: bool,
        is_orphaned: bool,
        metadata: StationMetadata,
        favicon: Option<gdk_pixbuf::Pixbuf>,
    ) -> Self {
        glib::Object::builder()
            .property("uuid", uuid)
            .property("is-local", is_local)
            .property("is-orphaned", is_orphaned)
            .property("metadata", metadata)
            .property("favicon", favicon)
            .build()
    }
}
