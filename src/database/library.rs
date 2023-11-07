// Shortwave - library.rs
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

use futures::future::join_all;
use glib::{
    clone, Enum, ObjectExt, ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, Sender, ToValue,
};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk_pixbuf, glib};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use url::Url;

use super::models::StationEntry;
use crate::api::{Client, Error, SwStation};
use crate::app::Action;
use crate::database::{connection, queries};
use crate::model::SwStationModel;

#[derive(Display, Copy, Debug, Clone, EnumString, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SwLibraryStatus")]
pub enum SwLibraryStatus {
    Loading,
    Content,
    Empty,
    Offline,
}

impl Default for SwLibraryStatus {
    fn default() -> Self {
        SwLibraryStatus::Loading
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SwLibrary {
        pub model: SwStationModel,
        pub status: RefCell<SwLibraryStatus>,

        pub client: OnceCell<Client>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwLibrary {
        const NAME: &'static str = "SwLibrary";
        type Type = super::SwLibrary;
    }

    impl ObjectImpl for SwLibrary {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "model",
                        "",
                        "",
                        SwStationModel::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "status",
                        "",
                        "",
                        SwLibraryStatus::static_type(),
                        SwLibraryStatus::default() as i32,
                        ParamFlags::READABLE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "model" => obj.model().to_value(),
                "status" => obj.status().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SwLibrary(ObjectSubclass<imp::SwLibrary>);
}

impl SwLibrary {
    pub fn new(sender: Sender<Action>) -> Self {
        let library = glib::Object::new::<Self>(&[]).unwrap();
        library.imp().sender.set(sender).unwrap();

        library
    }

    pub fn model(&self) -> SwStationModel {
        self.imp().model.clone()
    }

    pub fn status(&self) -> SwLibraryStatus {
        *self.imp().status.borrow()
    }

    pub fn refresh_data(&self, server: Option<&Url>) {
        let imp = self.imp();

        if let Some(server) = server {
            if let Some(client) = imp.client.get() {
                client.set_server(server.clone())
            } else {
                let client = Client::new(server.clone());
                imp.client.set(client).unwrap();
            }
        }

        self.load_stations();
    }

    pub fn add_stations(&self, stations: Vec<SwStation>) {
        debug!("Add {} station(s)", stations.len());
        for station in stations {
            self.imp().model.add_station(&station);

            let entry = StationEntry::for_station(&station);
            queries::insert_station(entry).unwrap();
        }

        self.update_library_status();
    }

    pub fn remove_stations(&self, stations: Vec<SwStation>) {
        debug!("Remove {} station(s)", stations.len());
        for station in stations {
            self.imp().model.remove_station(&station);
            queries::delete_station(&station.uuid()).unwrap();
        }

        self.update_library_status();
    }

    pub fn contains_station(station: &SwStation) -> bool {
        queries::contains_station(&station.uuid()).unwrap()
    }

    fn update_library_status(&self) {
        let imp = self.imp();

        if imp.model.n_items() == 0 {
            *imp.status.borrow_mut() = SwLibraryStatus::Empty;
        } else {
            *imp.status.borrow_mut() = SwLibraryStatus::Content;
        }

        self.notify("status");
    }

    fn load_stations(&self) {
        // Load database async
        let future = clone!(@strong self as this => async move {
            // Clear previously loaded stations first
            this.imp().model.clear();

            let entries = queries::stations().unwrap();

            // Print database info
            info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
            info!("Stations: {}", entries.len());

            // Set library status to loading
            let imp = this.imp();
            *imp.status.borrow_mut() = SwLibraryStatus::Loading;
            this.notify("status");

            let futures = entries.into_iter().map(|entry| this.load_station(entry));
            join_all(futures).await;

            this.update_library_status();
        });
        spawn!(future);
    }

    /// Try to add a station to the database.
    async fn load_station(&self, entry: StationEntry) {
        let imp = self.imp();
        let client = imp.client.clone();
        let uuid = entry.uuid.clone();

        // Load custom favicon from database entry if available
        let mut favicon = None;
        if let Some(data) = entry.favicon {
            let loader = gdk_pixbuf::PixbufLoader::new();
            if loader.write(&data).is_ok() && loader.close().is_ok() {
                favicon = loader.pixbuf()
            }
        }

        // If it's a local entry, load the metadata just from the database
        // and don't try to retrieve data from radio-browser.info
        if entry.is_local {
            if let Some(data) = &entry.data {
                match serde_json::from_str(data) {
                    Ok(metadata) => {
                        let station = SwStation::new(uuid, true, false, metadata, favicon.clone());
                        imp.model.add_station(&station);
                    }
                    Err(err) => {
                        warn!(
                            "Unable to deserialize metadata for local station {}: {}",
                            uuid,
                            err.to_string()
                        );
                        // TODO: send notification
                    }
                }
            } else {
                warn!(
                    "No data for local station {}, removing empty entry from database.",
                    uuid
                );
                queries::delete_station(&uuid).unwrap();
            }
            return;
        }

        // Try to update station metadata from radio-browser.info
        let mut is_orphaned = false;
        if let Some(client) = client.get() {
            match client.clone().station_metadata_by_uuid(&uuid).await {
                Ok(metadata) => {
                    let station = SwStation::new(uuid, false, false, metadata, favicon);

                    // Cache data for future use
                    let entry = StationEntry::for_station(&station);
                    queries::update_station(entry).unwrap();

                    // Add station to the library
                    imp.model.add_station(&station);

                    return;
                }
                Err(err) => {
                    is_orphaned = matches!(err, Error::InvalidStation(_));
                    warn!(
                        "Unable to receive data for station {}, trying to use cached data: {}",
                        uuid,
                        err.to_string()
                    );
                }
            }
        }

        // Try using cached metadata from database as fallback
        if let Some(data) = &entry.data {
            match serde_json::from_str(data) {
                Ok(metadata) => {
                    let s = SwStation::new(uuid, false, is_orphaned, metadata, favicon);
                    imp.model.add_station(&s);
                }
                Err(err) => {
                    warn!(
                        "Unable to deserialize metadata for cached station {}: {}",
                        uuid,
                        err.to_string()
                    )
                }
            }
        } else {
            warn!("Unable to load station {}, no cached data available.", uuid);
        }
    }
}
