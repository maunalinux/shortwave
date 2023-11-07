// Shortwave - station_sorter.rs
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

use std::cell::{Cell, RefCell};

use glib::{Enum, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

use crate::api::SwStation;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SwStationSorter {
        pub descending: Cell<bool>,
        pub sorting: RefCell<SwSorting>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationSorter {
        const NAME: &'static str = "SwStationSorter";
        type Type = super::SwStationSorter;
        type ParentType = gtk::Sorter;
    }

    impl ObjectImpl for SwStationSorter {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("descending", "", "", false, ParamFlags::READWRITE),
                    ParamSpecEnum::new(
                        "sorting",
                        "",
                        "",
                        SwSorting::static_type(),
                        SwSorting::default() as i32,
                        ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "descending" => obj.descending().to_value(),
                "sorting" => obj.sorting().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "descending" => obj.set_descending(value.get().unwrap()),
                "sorting" => obj.set_descending(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    impl SorterImpl for SwStationSorter {
        fn order(&self, _sorter: &Self::Type) -> gtk::SorterOrder {
            gtk::SorterOrder::Total
        }

        fn compare(
            &self,
            _sorter: &Self::Type,
            item1: &glib::Object,
            item2: &glib::Object,
        ) -> gtk::Ordering {
            let a = &item1.clone().downcast::<SwStation>().unwrap();
            let b = &item2.clone().downcast::<SwStation>().unwrap();
            super::SwStationSorter::station_cmp(a, b, *self.sorting.borrow(), self.descending.get())
                .into()
        }
    }
}

glib::wrapper! {
    pub struct SwStationSorter(ObjectSubclass<imp::SwStationSorter>) @extends gtk::Sorter;
}

impl SwStationSorter {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn descending(&self) -> bool {
        self.imp().descending.get()
    }

    pub fn set_descending(&self, descending: bool) {
        self.imp().descending.set(descending);
        self.changed(gtk::SorterChange::Different);
    }

    pub fn sorting(&self) -> SwSorting {
        *self.imp().sorting.borrow()
    }

    pub fn set_sorting(&self, sorting: SwSorting) {
        *self.imp().sorting.borrow_mut() = sorting;
        self.changed(gtk::SorterChange::Different);
    }

    fn station_cmp(
        a: &SwStation,
        b: &SwStation,
        sorting: SwSorting,
        descending: bool,
    ) -> std::cmp::Ordering {
        let mut station_a = a.clone();
        let mut station_b = b.clone();

        if descending {
            std::mem::swap(&mut station_a, &mut station_b);
        }

        match sorting {
            SwSorting::Default => std::cmp::Ordering::Equal,
            SwSorting::Name => station_a.metadata().name.cmp(&station_b.metadata().name),
            SwSorting::Language => station_a
                .metadata()
                .language
                .cmp(&station_b.metadata().language),
            SwSorting::Country => station_a
                .metadata()
                .country
                .cmp(&station_b.metadata().country),
            SwSorting::State => station_a.metadata().state.cmp(&station_b.metadata().state),
            SwSorting::Codec => station_a.metadata().codec.cmp(&station_b.metadata().codec),
            SwSorting::Votes => station_a.metadata().votes.cmp(&station_b.metadata().votes),
            SwSorting::Bitrate => station_a
                .metadata()
                .bitrate
                .cmp(&station_b.metadata().bitrate),
        }
    }
}

impl Default for SwStationSorter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Display, Copy, Debug, Clone, EnumString, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SwSorting")]
pub enum SwSorting {
    Default,
    Name,
    Language,
    Country,
    State,
    Codec,
    Votes,
    Bitrate,
}

impl Default for SwSorting {
    fn default() -> Self {
        SwSorting::Default
    }
}
