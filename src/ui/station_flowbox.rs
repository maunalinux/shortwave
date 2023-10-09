// Shortwave - station_flowbox.rs
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
use glib::{clone, subclass, Properties, Sender};
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::api::SwStation;
use crate::app::Action;
use crate::model::{SwSorting, SwStationModel, SwStationSorter};
use crate::ui::{SwStationDialog, SwStationRow};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Properties)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui")]
    #[properties(wrapper_type = super::SwStationFlowBox)]
    pub struct SwStationFlowBox {
        #[property(get)]
        pub model: gtk::SortListModel,

        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,

        pub sorter: SwStationSorter,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationFlowBox {
        const NAME: &'static str = "SwStationFlowBox";
        type ParentType = adw::Bin;
        type Type = super::SwStationFlowBox;

        fn new() -> Self {
            let sorter = SwStationSorter::new();
            let model = gtk::SortListModel::new(None::<SwStationModel>, Some(sorter.clone()));

            Self {
                flowbox: TemplateChild::default(),
                sorter,
                model,
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SwStationFlowBox {}

    impl WidgetImpl for SwStationFlowBox {}

    impl BinImpl for SwStationFlowBox {}
}

glib::wrapper! {
    pub struct SwStationFlowBox(ObjectSubclass<imp::SwStationFlowBox>)
        @extends gtk::Widget, adw::Bin;
}

impl SwStationFlowBox {
    pub fn init(&self, model: SwStationModel, sender: Sender<Action>) {
        let imp = self.imp();
        imp.model.set_model(Some(&model));

        self.setup_signals(sender);
    }

    pub fn set_sorting(&self, sorting: SwSorting, descending: bool) {
        let imp = self.imp();
        imp.sorter.set_sorting(sorting);
        imp.sorter.set_descending(descending);
    }

    fn setup_signals(&self, sender: Sender<Action>) {
        let imp = self.imp();

        imp.flowbox.get().bind_model(
            Some(&self.imp().model),
            clone!(@strong sender => move |station|{
                let station = station.downcast_ref::<SwStation>().unwrap();
                let row = SwStationRow::new(sender.clone(), station.clone());
                row.upcast()
            }),
        );

        // Show StationDialog when row gets clicked
        imp.flowbox
            .connect_child_activated(clone!(@strong sender => move |_, child| {
                let row = child.clone().downcast::<SwStationRow>().unwrap();
                let station = row.station();

                let station_dialog = SwStationDialog::new(&station);
                station_dialog.show();
            }));
    }
}
