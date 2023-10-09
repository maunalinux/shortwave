// Shortwave - station_dialog.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use futures_util::future::FutureExt;
use glib::{subclass, Properties};
use gtk::{gdk, glib, CompositeTemplate};
use inflector::Inflector;
use once_cell::unsync::OnceCell;
use shumate::prelude::*;

use crate::api::{FaviconDownloader, SwStation};
use crate::app::SwApplication;
use crate::database::SwLibrary;
use crate::i18n;
use crate::ui::{FaviconSize, StationFavicon, SwApplicationWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_dialog.ui")]
    #[properties(wrapper_type = super::SwStationDialog)]
    pub struct SwStationDialog {
        #[template_child]
        favicon_box: TemplateChild<gtk::Box>,
        #[template_child]
        local_station_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        orphaned_station_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        title_label: TemplateChild<gtk::Label>,
        #[template_child]
        homepage_label: TemplateChild<gtk::Label>,
        #[template_child]
        library_add_child: TemplateChild<gtk::FlowBoxChild>,
        #[template_child]
        library_remove_child: TemplateChild<gtk::FlowBoxChild>,
        #[template_child]
        information_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        language_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        tags_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        codec_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        bitrate_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        votes_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        stream_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        location_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        country_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        state_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        map_box: TemplateChild<gtk::Box>,
        #[template_child]
        map: TemplateChild<shumate::Map>,
        #[template_child]
        map_license: TemplateChild<shumate::License>,
        marker: shumate::Marker,

        #[property(get, set, construct_only)]
        station: OnceCell<SwStation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationDialog {
        const NAME: &'static str = "SwStationDialog";
        type ParentType = adw::Window;
        type Type = super::SwStationDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SwStationDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let window = SwApplicationWindow::default();
            self.obj().set_transient_for(Some(&window));

            self.setup_widgets();
        }
    }

    impl WidgetImpl for SwStationDialog {}

    impl WindowImpl for SwStationDialog {}

    impl AdwWindowImpl for SwStationDialog {}

    #[gtk::template_callbacks]
    impl SwStationDialog {
        fn setup_widgets(&self) {
            let station = self.obj().station();
            let metadata = station.metadata();

            // Download & set station favicon
            let station_favicon = StationFavicon::new(FaviconSize::Big);
            self.favicon_box.append(&station_favicon.widget);

            if let Some(pixbuf) = station.favicon() {
                station_favicon.set_pixbuf(&pixbuf);
            } else if let Some(favicon) = metadata.favicon.as_ref() {
                let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Big as i32)
                    .map(move |pixbuf| {
                        if let Ok(pixbuf) = pixbuf {
                            station_favicon.set_pixbuf(&pixbuf)
                        }
                    });
                spawn!(fut);
            }

            // Title
            self.obj().set_title(Some(&metadata.name));
            self.title_label.set_text(&metadata.name);

            // Homepage
            if let Some(ref homepage) = metadata.homepage {
                let url = homepage.to_string().replace('&', "&amp;");
                let domain = homepage.domain().unwrap();
                let markup = format!("<a href=\"{}\">{}</a>", &url, &domain);

                self.homepage_label.set_visible(true);
                self.homepage_label.set_markup(&markup);
                self.homepage_label.set_tooltip_text(Some(&url));
            }

            // Action pill buttons
            if SwLibrary::contains_station(&station) {
                self.library_remove_child.set_visible(true);
            } else {
                self.library_add_child.set_visible(true);
            }

            // Local station info row
            if self.station.get().unwrap().is_local() {
                self.local_station_group.set_visible(true);
                self.information_group.set_visible(false);
            }

            // Orphaned station info row
            if self.station.get().unwrap().is_orphaned() {
                self.orphaned_station_group.set_visible(true);
            }

            // Tags
            if !metadata.tags.is_empty() {
                self.tags_row.set_visible(true);
                self.tags_row.set_subtitle(&metadata.formatted_tags());
            }

            // Language
            if !metadata.language.is_empty() {
                self.language_row.set_visible(true);
                self.language_row
                    .set_subtitle(&metadata.language.to_title_case());
            }

            // Votes
            self.votes_row.set_subtitle(&metadata.votes.to_string());

            // Location
            if !metadata.country.is_empty() {
                self.location_group.set_visible(true);
                self.country_row.set_visible(true);
                self.country_row.set_subtitle(&metadata.country);
            }
            if !metadata.state.is_empty() {
                self.location_group.set_visible(true);
                self.state_row.set_visible(true);
                self.state_row.set_subtitle(&metadata.state);
            }

            // Map
            let long: f64 = metadata.geo_long.unwrap_or(0.0).into();
            let lat: f64 = metadata.geo_lat.unwrap_or(0.0).into();
            if long != 0.0 || lat != 0.0 {
                self.setup_map_widget();
                self.map_box.set_visible(true);
                self.marker.set_location(lat, long);
                self.map.center_on(lat, long);
            }

            // Codec
            if !metadata.codec.is_empty() {
                self.codec_row.set_visible(true);
                self.codec_row.set_subtitle(&metadata.codec);
            }

            // Bitrate
            if metadata.bitrate != 0 {
                self.bitrate_row.set_visible(true);
                let bitrate = i18n::i18n_f("{} kbit/s", &[&metadata.bitrate.to_string()]);
                self.bitrate_row.set_subtitle(&bitrate);
            }

            // Stream url
            let url = if let Some(url_resolved) = metadata.url_resolved {
                url_resolved.to_string()
            } else {
                metadata.url.map(|x| x.to_string()).unwrap_or_default()
            };
            let url = url.replace('&', "&amp;");
            let subtitle = format!("<a href=\"{}\">{}</a>", &url, &url);

            self.stream_row.set_subtitle(&subtitle);
            self.stream_row.set_tooltip_text(Some(&url));
        }

        fn setup_map_widget(&self) {
            let registry = shumate::MapSourceRegistry::with_defaults();

            let source = registry.by_id(shumate::MAP_SOURCE_OSM_MAPNIK).unwrap();
            self.map.set_map_source(&source);

            let viewport = self.map.viewport().unwrap();
            viewport.set_reference_map_source(Some(&source));
            viewport.set_zoom_level(6.0);

            let layer = shumate::MapLayer::new(&source, &viewport);
            self.map.add_layer(&layer);

            let marker_layer = shumate::MarkerLayer::new(&viewport);
            marker_layer.add_marker(&self.marker);
            self.map.add_layer(&marker_layer);

            let marker_img = gtk::Image::from_icon_name("mark-location-symbolic");
            marker_img.add_css_class("map-pin");
            marker_img.set_icon_size(gtk::IconSize::Large);
            self.marker.set_child(Some(&marker_img));

            self.map_license.append_map_source(&source);
        }

        #[template_callback]
        fn add_station(&self) {
            let obj = self.obj();

            let station = obj.station();
            SwApplication::default()
                .library()
                .add_stations(vec![station]);

            obj.hide();
            obj.close();
        }

        #[template_callback]
        fn remove_station(&self) {
            let obj = self.obj();

            let station = obj.station();
            SwApplication::default()
                .library()
                .remove_stations(vec![station]);

            obj.hide();
            obj.close();
        }

        #[template_callback]
        fn start_playback(&self) {
            let obj = self.obj();
            let station = obj.station();

            let app = SwApplication::default();
            app.imp().player.set_station(station);

            obj.hide();
            obj.close();
        }

        #[template_callback]
        fn copy_stream_clipboard(&self) {
            let metadata = self.obj().station().metadata();

            if let Some(url_resolved) = metadata.url_resolved {
                let display = gdk::Display::default().unwrap();
                let clipboard = display.clipboard();
                clipboard.set_text(url_resolved.as_ref());
            }
        }
    }
}

glib::wrapper! {
    pub struct SwStationDialog(ObjectSubclass<imp::SwStationDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl SwStationDialog {
    pub fn new(station: &SwStation) -> Self {
        glib::Object::builder().property("station", station).build()
    }
}
