// Shortwave - discover_page.rs
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

use adw::subclass::prelude::*;
use futures_util::FutureExt;
use glib::{subclass, Sender};
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::unsync::OnceCell;
use url::Url;

use crate::api::{Client, StationRequest};
use crate::app;
use crate::i18n::*;
use crate::ui::featured_carousel::Action;
use crate::ui::{SwApplicationWindow, SwFeaturedCarousel, SwStationFlowBox};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/discover_page.ui")]
    pub struct SwDiscoverPage {
        #[template_child]
        pub carousel: TemplateChild<SwFeaturedCarousel>,
        #[template_child]
        pub votes_flowbox: TemplateChild<SwStationFlowBox>,
        #[template_child]
        pub trending_flowbox: TemplateChild<SwStationFlowBox>,
        #[template_child]
        pub clicked_flowbox: TemplateChild<SwStationFlowBox>,

        pub sender: OnceCell<Sender<app::Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwDiscoverPage {
        const NAME: &'static str = "SwDiscoverPage";
        type ParentType = adw::Bin;
        type Type = super::SwDiscoverPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwDiscoverPage {}

    impl WidgetImpl for SwDiscoverPage {}

    impl BinImpl for SwDiscoverPage {}
}

glib::wrapper! {
    pub struct SwDiscoverPage(ObjectSubclass<imp::SwDiscoverPage>)
        @extends gtk::Widget, adw::Bin;
}

impl SwDiscoverPage {
    pub fn init(&self, sender: Sender<app::Action>) {
        let imp = self.imp();
        imp.sender.set(sender).unwrap();

        self.setup_widgets();
    }

    pub fn refresh_data(&self, server: &Url) {
        let imp = self.imp();

        // Most voted stations (stations with the most votes)
        let votes_request = StationRequest {
            order: Some("votes".to_string()),
            limit: Some(12),
            reverse: Some(true),
            ..Default::default()
        };
        self.fill_flowbox(server, &imp.votes_flowbox, votes_request);

        // Trending (stations with the highest clicktrend)
        let trending_request = StationRequest {
            order: Some("clicktrend".to_string()),
            limit: Some(12),
            ..Default::default()
        };
        self.fill_flowbox(server, &imp.trending_flowbox, trending_request);

        // Other users are listening to... (stations which got recently clicked)
        let clicked_request = StationRequest {
            order: Some("clicktimestamp".to_string()),
            limit: Some(12),
            ..Default::default()
        };
        self.fill_flowbox(server, &imp.clicked_flowbox, clicked_request);
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        // TODO: Implement show-server-stats action
        let _action = Action::new("win.show-server-stats", &i18n("Show statistics"));
        imp.carousel
            .add_page(&i18n("Browse over 30,000 stations"), "#1a5fb4", None);

        let action = Action::new("win.create-new-station", &i18n("Add new station"));
        imp.carousel.add_page(
            &i18n("Your favorite station is missing?"),
            "#e5a50a",
            Some(action),
        );

        let action = Action::new("win.open-radio-browser-info", &i18n("Open website"));
        imp.carousel.add_page(
            &i18n("Powered by radio-browser.info"),
            "#26a269",
            Some(action),
        );
    }

    fn fill_flowbox(&self, server: &Url, flowbox: &SwStationFlowBox, request: StationRequest) {
        let imp = self.imp();

        let client = Client::new(server.clone());
        let sender = imp.sender.get().unwrap().clone();

        let model = &*client.model;
        flowbox.init(model.clone(), sender);

        let fut = client.send_station_request(request).map(move |result| {
            if let Err(err) = result {
                warn!("Station data could not be received: {}", err.to_string());

                let text = i18n("Station data could not be received.");
                SwApplicationWindow::default().show_notification(&text);
            }
        });

        spawn!(fut);
    }
}
