// Shortwave - station_row.rs
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

use futures_util::future::FutureExt;
use glib::{clone, Sender};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use inflector::Inflector;
use once_cell::unsync::OnceCell;

use crate::api::{FaviconDownloader, SwStation};
use crate::app::Action;
use crate::ui::{FaviconSize, StationFavicon};
use crate::SwApplication;

mod imp {
    use glib::subclass;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_row.ui")]
    pub struct SwStationRow {
        #[template_child]
        pub station_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub favicon_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub local_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub orphaned_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,

        pub station: OnceCell<SwStation>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationRow {
        const NAME: &'static str = "SwStationRow";
        type ParentType = gtk::FlowBoxChild;
        type Type = super::SwStationRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwStationRow {}

    impl WidgetImpl for SwStationRow {}

    impl FlowBoxChildImpl for SwStationRow {}
}

glib::wrapper! {
    pub struct SwStationRow(ObjectSubclass<imp::SwStationRow>)
        @extends gtk::Widget, gtk::FlowBoxChild;
}

impl SwStationRow {
    pub fn new(sender: Sender<Action>, station: SwStation) -> Self {
        let row = glib::Object::new::<Self>();

        let imp = row.imp();
        imp.sender.set(sender).unwrap();
        imp.station.set(station).unwrap();

        row.setup_widgets();
        row.setup_signals();

        row
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        // play_button
        imp.play_button.connect_clicked(
            clone!(@strong imp.sender as sender, @strong imp.station as station => move |_| {
                SwApplication::default().imp().player.set_station(station.get().unwrap().clone());
            }),
        );
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        // Set row information
        let station = imp.station.get().unwrap();
        imp.station_label.set_text(&station.metadata().name);

        // Set subtitle
        let metadata = station.metadata();
        let mut subtitle = metadata.country.to_title_case();

        if subtitle.is_empty() {
            subtitle = metadata.tags;
        } else if !metadata.tags.is_empty() {
            subtitle = format!("{} · {}", subtitle, metadata.formatted_tags());
        }

        imp.subtitle_label.set_text(&subtitle);
        imp.subtitle_label.set_visible(!subtitle.is_empty());

        imp.local_image.set_visible(station.is_local());
        imp.orphaned_image.set_visible(station.is_orphaned());

        // Download & set station favicon
        let station_favicon = StationFavicon::new(FaviconSize::Small);
        imp.favicon_box.append(&station_favicon.widget);

        if let Some(pixbuf) = station.favicon() {
            station_favicon.set_pixbuf(&pixbuf);
        } else if let Some(favicon) = station.metadata().favicon.as_ref() {
            let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Small as i32).map(
                move |pixbuf| {
                    if let Ok(pixbuf) = pixbuf {
                        station_favicon.set_pixbuf(&pixbuf)
                    }
                },
            );
            spawn!(fut);
        }
    }

    pub fn station(&self) -> SwStation {
        self.imp().station.get().unwrap().clone()
    }
}
