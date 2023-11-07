// Shortwave - station_dialog.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use futures_util::future::FutureExt;
use glib::{clone, subclass, Sender};
use gtk::{gdk, glib, CompositeTemplate};
use inflector::Inflector;
use once_cell::unsync::OnceCell;
use shumate::prelude::*;

use crate::api::{FaviconDownloader, SwStation};
use crate::app::{Action, SwApplication};
use crate::database::SwLibrary;
use crate::i18n;
use crate::ui::{FaviconSize, StationFavicon, SwApplicationWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_dialog.ui")]
    pub struct SwStationDialog {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub dialog_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub favicon_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub local_station_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub orphaned_station_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub homepage_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub library_add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub library_add_child: TemplateChild<gtk::FlowBoxChild>,
        #[template_child]
        pub library_remove_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub library_remove_child: TemplateChild<gtk::FlowBoxChild>,
        #[template_child]
        pub start_playback_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub information_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub language_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub language_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tags_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub tags_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub codec_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub codec_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub bitrate_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub bitrate_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub votes_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub votes_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub stream_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stream_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub copy_stream_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub location_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub country_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub country_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub state_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub state_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub map_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub map: TemplateChild<shumate::Map>,
        #[template_child]
        pub map_license: TemplateChild<shumate::License>,
        pub marker: shumate::Marker,

        pub station: OnceCell<SwStation>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationDialog {
        const NAME: &'static str = "SwStationDialog";
        type ParentType = adw::Window;
        type Type = super::SwStationDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwStationDialog {
        fn constructed(&self, obj: &Self::Type) {
            // Setup the libshumate map widget
            // Based on ashpd-demo
            // https://github.com/bilelmoussaoui/ashpd/blob/66d4dc0020181a7174451150ecc711344082b5ce/ashpd-demo/src/portals/desktop/location.rs
            let registry = shumate::MapSourceRegistry::with_defaults();

            let source = registry.by_id(&shumate::MAP_SOURCE_OSM_MAPNIK).unwrap();
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
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for SwStationDialog {}

    impl WindowImpl for SwStationDialog {}

    impl AdwWindowImpl for SwStationDialog {}
}

glib::wrapper! {
    pub struct SwStationDialog(ObjectSubclass<imp::SwStationDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl SwStationDialog {
    pub fn new(sender: Sender<Action>, station: SwStation) -> Self {
        let dialog: Self = glib::Object::new(&[]).unwrap();

        let imp = dialog.imp();
        imp.station.set(station).unwrap();
        imp.sender.set(sender).unwrap();

        let window = SwApplicationWindow::default();
        dialog.set_transient_for(Some(&window));

        dialog.setup_widgets();
        dialog.setup_signals();
        dialog
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        let station = imp.station.get().unwrap();
        let metadata = station.metadata();

        // Download & set station favicon
        let station_favicon = StationFavicon::new(FaviconSize::Big);
        imp.favicon_box.append(&station_favicon.widget);

        if let Some(pixbuf) = station.favicon() {
            station_favicon.set_pixbuf(&pixbuf);
        } else if let Some(favicon) = metadata.favicon.as_ref() {
            let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Big as i32).map(
                move |pixbuf| {
                    if let Ok(pixbuf) = pixbuf {
                        station_favicon.set_pixbuf(&pixbuf)
                    }
                },
            );
            spawn!(fut);
        }

        // Title + Homepage
        imp.title_label.set_text(&metadata.name);
        imp.dialog_title.set_title(&metadata.name);

        if let Some(ref homepage) = metadata.homepage {
            let url = homepage.to_string().replace('&', "&amp;");
            let domain = homepage.domain().unwrap();

            imp.homepage_label.set_visible(true);
            imp.homepage_label
                .set_markup(&format!("<a href=\"{}\">{}</a>", &url, &domain));
            imp.homepage_label.set_tooltip_text(Some(&url));
        }

        // Action pill buttons
        if SwLibrary::contains_station(imp.station.get().unwrap()) {
            imp.library_remove_child.set_visible(true);
        } else {
            imp.library_add_child.set_visible(true);
        }

        // General information group
        if !metadata.tags.is_empty() {
            imp.tags_row.set_visible(true);
            imp.tags_label.set_text(&metadata.formatted_tags());
        }

        if !metadata.language.is_empty() {
            imp.language_row.set_visible(true);
            imp.language_label
                .set_text(&metadata.language.to_title_case());
        }

        imp.votes_label.set_text(&metadata.votes.to_string());

        if imp.station.get().unwrap().is_local() {
            imp.local_station_group.set_visible(true);
            imp.information_group.set_visible(false);
        }

        if imp.station.get().unwrap().is_orphaned() {
            imp.orphaned_station_group.set_visible(true);
        }

        // Location & Map
        if !metadata.country.is_empty() {
            imp.location_group.set_visible(true);
            imp.country_row.set_visible(true);
            imp.country_label.set_text(&metadata.country);
        }

        if !metadata.state.is_empty() {
            imp.location_group.set_visible(true);
            imp.state_row.set_visible(true);
            imp.state_label.set_text(&metadata.state);
        }

        // TODO: Re-enable map widget as soon as we have a formal OSM approval
        // https://operations.osmfoundation.org/policies/tiles/
        // let long: f64 = metadata.geo_long.unwrap_or(0.0).into();
        // let lat: f64 = metadata.geo_lat.unwrap_or(0.0).into();
        //
        // if long != 0.0 || lat != 0.0 {
        // imp.map_box.set_visible(true);
        // imp.marker.set_location(lat, long);
        // imp.map.center_on(lat, long);
        // }

        // Audio group
        if !metadata.codec.is_empty() {
            imp.codec_row.set_visible(true);
            imp.codec_label.set_text(&metadata.codec);
        }

        if metadata.bitrate != 0 {
            imp.bitrate_row.set_visible(true);
            let bitrate = i18n::i18n_f("{} kbit/s", &[&metadata.bitrate.to_string()]);
            imp.bitrate_label.set_text(&bitrate);
        }

        let url = if let Some(url_resolved) = metadata.url_resolved {
            url_resolved.to_string()
        } else {
            metadata.url.map(|x| x.to_string()).unwrap_or_default()
        };
        let url = url.replace('&', "&amp;");
        imp.stream_label
            .set_markup(&format!("<a href=\"{}\">{}</a>", &url, &url));
        imp.stream_label.set_tooltip_text(Some(&url));
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.scrolled_window.vadjustment().connect_value_notify(
            clone!(@weak self as this => move |adj|{
                let imp = this.imp();
                if adj.value() < 210.0 {
                    imp.headerbar.add_css_class("hidden");
                    imp.dialog_title.set_visible(false);
                }else {
                    imp.headerbar.remove_css_class("hidden");
                    imp.dialog_title.set_visible(true);
                }
            }),
        );

        imp.library_add_button
            .connect_clicked(clone!(@weak self as this => move|_|
                let imp = this.imp();
                let station = imp.station.get().unwrap().clone();
                SwApplication::default().library().add_stations(vec![station]);

                this.hide();
                this.close();
            ));

        imp.library_remove_button
            .connect_clicked(clone!(@weak self as this => move|_|
                let imp = this.imp();
                let station = imp.station.get().unwrap().clone();
                SwApplication::default().library().remove_stations(vec![station]);

                this.hide();
                this.close();
            ));

        imp.start_playback_button
            .connect_clicked(clone!(@weak self as this => move|_|
                let imp = this.imp();
                let station = imp.station.get().unwrap().clone();

                send!(imp.sender.get().unwrap(), Action::PlaybackSetStation(Box::new(station)));
                this.hide();
                this.close();
            ));

        imp.copy_stream_button
            .connect_clicked(clone!(@weak self as this => move|_|
                let metadata = this.imp().station.get().unwrap().clone().metadata();

                if let Some(url_resolved) = metadata.url_resolved {
                    let display = gdk::Display::default().unwrap();
                    let clipboard = display.clipboard();
                    clipboard.set_text(url_resolved.as_ref());
                }
            ));
    }
}
