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

use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use adw::prelude::*;
use glib::{clone, subclass, Receiver, Sender};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::app::Action;
use crate::audio::{GCastDiscoverer, GCastDiscovererMessage};
use crate::ui::SwApplicationWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui")]
    pub struct SwStreamingDialog {
        #[template_child]
        pub row_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub devices_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,

        pub gcd: OnceCell<Rc<GCastDiscoverer>>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStreamingDialog {
        const NAME: &'static str = "SwStreamingDialog";
        type ParentType = gtk::Dialog;
        type Type = super::SwStreamingDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwStreamingDialog {}

    impl WidgetImpl for SwStreamingDialog {}

    impl WindowImpl for SwStreamingDialog {}

    impl DialogImpl for SwStreamingDialog {}
}

glib::wrapper! {
    pub struct SwStreamingDialog(ObjectSubclass<imp::SwStreamingDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

#[gtk::template_callbacks]
impl SwStreamingDialog {
    pub fn new(sender: Sender<Action>) -> Self {
        let dialog = glib::Object::builder::<Self>()
            .property("use-header-bar", 1)
            .build();

        // Setup Google Cast discoverer
        let gcd_t = GCastDiscoverer::new();
        let gcd = Rc::new(gcd_t.0);
        gcd.start_discover();
        let gcd_receiver = gcd_t.1;

        let imp = dialog.imp();
        imp.sender.set(sender).unwrap();
        imp.gcd.set(gcd).unwrap();

        dialog.setup_signals(gcd_receiver);
        dialog
    }

    fn setup_signals(&self, gcd_receiver: Receiver<GCastDiscovererMessage>) {
        gcd_receiver.attach(
            None,
            clone!(@weak self as this => @default-panic, move |message| {
                let imp = this.imp();

                match message {
                    GCastDiscovererMessage::DiscoverStarted => {
                        while let Some(child) = imp.devices_listbox.first_child() {
                            imp.devices_listbox.remove(&child);
                        }
                        imp.devices_listbox.set_visible(false);
                        imp.row_stack.set_visible_child_name("loading");
                        imp.spinner.set_spinning(true);
                    }
                    GCastDiscovererMessage::DiscoverEnded => {
                        if imp.devices_listbox.last_child().is_none() {
                            imp.row_stack.set_visible_child_name("no-devices");
                        } else {
                            imp.row_stack.set_visible_child_name("ready");
                        }
                        imp.spinner.set_spinning(false);
                    }
                    GCastDiscovererMessage::FoundDevice(device) => {
                        imp.row_stack.set_visible_child_name("ready");

                        let row = adw::ActionRow::new();
                        row.set_title(&device.name);
                        row.set_subtitle(&device.ip.to_string());
                        row.set_activatable(true);

                        imp.devices_listbox.append(&row);
                        imp.devices_listbox.set_visible(true);
                        imp.spinner.set_spinning(false);
                    }
                }

                glib::ControlFlow::Continue
            }),
        );

        self.imp().devices_listbox.connect_row_activated(
            clone!(@weak self as this => move |_, row|{
                let imp = this.imp();
                let row: adw::ActionRow = row.clone().downcast().unwrap();
                let ip_addr: IpAddr = IpAddr::from_str(row.subtitle().unwrap().as_str()).unwrap();

                // Get GCastDevice
                let device = imp.gcd.get().unwrap().device_by_ip_addr(ip_addr).unwrap();
                send!(imp.sender.get().unwrap(), Action::PlaybackConnectGCastDevice(device));
                this.hide();
            }),
        );

        self.connect_show(clone!(@weak self as this => move |_|{
            let window = SwApplicationWindow::default();
            this.set_transient_for(Some(&window));
            this.set_modal(true);
        }));
    }

    #[template_callback]
    fn search_again(&self) {
        self.imp().gcd.get().unwrap().start_discover();
    }
}
