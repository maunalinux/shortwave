// Shortwave - discover_page.rs
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

use adw::subclass::prelude::*;
use glib::{closure, subclass, Sender};
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::api::{Error, StationRequest, SwClient};
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

        pub client1: SwClient,
        pub client2: SwClient,
        pub client3: SwClient,
        pub sender: OnceCell<Sender<app::Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwDiscoverPage {
        const NAME: &'static str = "SwDiscoverPage";
        type ParentType = adw::NavigationPage;
        type Type = super::SwDiscoverPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwDiscoverPage {
        fn constructed(&self) {}
    }

    impl WidgetImpl for SwDiscoverPage {}

    impl NavigationPageImpl for SwDiscoverPage {
        fn shown(&self) {
            // TODO: THIS IS HORRIBLE
            // burn it with fire, and rewrite it from scratch

            // Most voted stations (stations with the most votes)
            let votes_request = StationRequest {
                order: Some("votes".to_string()),
                limit: Some(12),
                reverse: Some(true),
                ..Default::default()
            };
            self.fill_flowbox(&self.client1, &self.votes_flowbox, votes_request);

            // Trending (stations with the highest clicktrend)
            let trending_request = StationRequest {
                order: Some("clicktrend".to_string()),
                limit: Some(12),
                ..Default::default()
            };
            self.fill_flowbox(&self.client2, &self.trending_flowbox, trending_request);

            // Other users are listening to... (stations which got recently clicked)
            let clicked_request = StationRequest {
                order: Some("clicktimestamp".to_string()),
                limit: Some(12),
                ..Default::default()
            };
            self.fill_flowbox(&self.client3, &self.clicked_flowbox, clicked_request);
        }
    }

    impl SwDiscoverPage {
        fn fill_flowbox(
            &self,
            client: &SwClient,
            flowbox: &SwStationFlowBox,
            request: StationRequest,
        ) {
            let sender = self.sender.get().unwrap().clone();
            flowbox.init(client.model(), sender);
            client.send_station_request(request);
        }
    }
}

glib::wrapper! {
    pub struct SwDiscoverPage(ObjectSubclass<imp::SwDiscoverPage>)
        @extends gtk::Widget, adw::NavigationPage;
}

impl SwDiscoverPage {
    pub fn init(&self, sender: Sender<app::Action>) {
        let imp = self.imp();
        imp.sender.set(sender).unwrap();

        self.setup_widgets();
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

        imp.client1.connect_closure(
            "error",
            false,
            closure!(|_: SwClient, err: Error| {
                warn!("Station data could not be received: {}", err.to_string());

                let text = i18n("Station data could not be received.");
                SwApplicationWindow::default().show_notification(&text);
            }),
        );
    }
}
