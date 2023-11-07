// Shortwave - station_model.rs
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
use std::convert::TryInto;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;

use crate::api::SwStation;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SwStationModel {
        pub map: RefCell<IndexMap<String, SwStation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationModel {
        const NAME: &'static str = "SwStationModel";
        type Type = super::SwStationModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SwStationModel {}

    impl ListModelImpl for SwStationModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SwStation::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position.try_into().unwrap())
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SwStationModel(ObjectSubclass<imp::SwStationModel>) @implements gio::ListModel;
}

impl SwStationModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_station(&self, station: &SwStation) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&station.uuid()) {
                warn!(
                    "Station {:?} already exists in model",
                    station.metadata().name
                );
                return;
            }

            map.insert(station.uuid(), station.clone());
            (map.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_station(&self, station: &SwStation) {
        let mut map = self.imp().map.borrow_mut();

        match map.get_index_of(&station.uuid()) {
            Some(pos) => {
                map.remove(&station.uuid());
                self.items_changed(pos.try_into().unwrap(), 1, 0);
            }
            None => warn!("Station {:?} not found in model", station.metadata().name),
        }
    }

    pub fn clear(&self) {
        let len = self.n_items();
        self.imp().map.borrow_mut().clear();
        self.items_changed(0, len, 0);
    }
}

impl Default for SwStationModel {
    fn default() -> Self {
        Self::new()
    }
}
