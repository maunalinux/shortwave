// Shortwave - app.rs
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
use std::rc::Rc;
use std::str::FromStr;

use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::{clone, ObjectExt, ParamSpec, ParamSpecObject, Receiver, Sender, ToValue};
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::{Lazy, OnceCell};

use crate::api::{Client, SwStation};
use crate::audio::{GCastDevice, PlaybackState, Player, Song};
use crate::config;
use crate::database::SwLibrary;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key, SettingsWindow};
use crate::ui::{about_window, SwApplicationWindow, SwView};

#[derive(Debug, Clone)]
pub enum Action {
    // Audio Playback
    PlaybackConnectGCastDevice(GCastDevice),
    PlaybackDisconnectGCastDevice,
    PlaybackSetStation(Box<SwStation>),
    PlaybackSet(bool),
    PlaybackToggle,
    PlaybackSetVolume(f64),
    PlaybackSaveSong(Song),

    SettingsKeyChanged(Key),
}

mod imp {
    use super::*;

    pub struct SwApplication {
        pub sender: Sender<Action>,
        pub receiver: RefCell<Option<Receiver<Action>>>,

        pub window: OnceCell<WeakRef<SwApplicationWindow>>,
        pub player: Rc<Player>,
        pub library: SwLibrary,

        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwApplication {
        const NAME: &'static str = "SwApplication";
        type ParentType = adw::Application;
        type Type = super::SwApplication;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            let window = OnceCell::new();
            let player = Player::new(sender.clone());
            let library = SwLibrary::new(sender.clone());

            let settings = settings_manager::settings();

            Self {
                sender,
                receiver,
                window,
                player,
                library,
                settings,
            }
        }
    }

    // Implement GLib.Object for SwApplication
    impl ObjectImpl for SwApplication {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "library",
                    "",
                    "",
                    SwLibrary::static_type(),
                    glib::ParamFlags::READABLE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "library" => obj.library().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    // Implement Gio.Application for SwApplication
    impl ApplicationImpl for SwApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("gio::Application -> activate()");
            let app = app.downcast_ref::<super::SwApplication>().unwrap();

            // If the window already exists,
            // present it instead creating a new one again.
            if let Some(weak_window) = self.window.get() {
                weak_window.upgrade().unwrap().present();
                info!("Application window presented.");
                return;
            }

            // No window available -> we have to create one
            let window = app.create_window();
            let _ = self.window.set(window.downgrade());
            info!("Created application window.");

            // Setup app level GActions
            app.setup_gactions();

            // Setup action channel
            let receiver = self.receiver.borrow_mut().take().unwrap();
            receiver.attach(
                None,
                clone!(@strong app => move |action| app.process_action(action)),
            );

            // Retrieve station data
            app.refresh_data();

            // Setup settings signal (we get notified when a key gets changed)
            self.settings.connect_changed(
                None,
                clone!(@strong self.sender as sender => move |_, key_str| {
                    let key: Key = Key::from_str(key_str).unwrap();
                    send!(sender, Action::SettingsKeyChanged(key));
                }),
            );

            // Needs to be called after settings.connect_changed for it to trigger.
            app.update_color_scheme();

            // Small workaround to update every view to the correct sorting/order.
            send!(self.sender, Action::SettingsKeyChanged(Key::ViewSorting));
        }
    }

    // Implement Gtk.Application for SwApplication
    impl GtkApplicationImpl for SwApplication {}

    // Implement Adw.Application for SwApplication
    impl AdwApplicationImpl for SwApplication {}
}

// Wrap SwApplication into a usable gtk-rs object
glib::wrapper! {
    pub struct SwApplication(ObjectSubclass<imp::SwApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

// SwApplication implementation itself
impl SwApplication {
    pub fn run() {
        debug!(
            "{} ({}) ({}) - Version {} ({})",
            config::NAME,
            config::APP_ID,
            config::VCS_TAG,
            config::VERSION,
            config::PROFILE
        );
        info!("Isahc version: {}", isahc::version());

        // Create new GObject and downcast it into SwApplication
        let app = glib::Object::new::<SwApplication>(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("resource-base-path", &Some(config::PATH_ID)),
        ])
        .unwrap();

        // Start running gtk::Application
        app.run();
    }

    fn create_window(&self) -> SwApplicationWindow {
        let imp = self.imp();
        let window = SwApplicationWindow::new(imp.sender.clone(), self.clone(), imp.player.clone());

        // Set initial view
        window.set_view(SwView::Library);

        window.present();
        window
    }

    fn setup_gactions(&self) {
        let window = SwApplicationWindow::default();

        // app.show-preferences
        action!(
            self,
            "show-preferences",
            clone!(@weak window => move |_, _| {
                let settings_window = SettingsWindow::new(&window.upcast());
                settings_window.show();
            })
        );
        self.set_accels_for_action("app.show-preferences", &["<primary>comma"]);

        // app.quit
        action!(
            self,
            "quit",
            clone!(@weak window => move |_, _| {
                window.close();
            })
        );
        self.set_accels_for_action("app.quit", &["<primary>q"]);

        // app.about
        action!(
            self,
            "about",
            clone!(@weak window => move |_, _| {
                about_window::show(&window);
            })
        );

        self.set_accels_for_action("window.close", &["<primary>w"]);
    }

    pub fn library(&self) -> SwLibrary {
        self.imp().library.clone()
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let imp = self.imp();
        if self.active_window().is_none() {
            return glib::Continue(true);
        }

        let window = SwApplicationWindow::default();

        match action {
            Action::PlaybackConnectGCastDevice(device) => {
                imp.player.connect_to_gcast_device(device)
            }
            Action::PlaybackDisconnectGCastDevice => imp.player.disconnect_from_gcast_device(),
            Action::PlaybackSetStation(station) => {
                imp.player.set_station(*station);
                window.show_player_widget();
            }
            Action::PlaybackSet(true) => imp.player.set_playback(PlaybackState::Playing),
            Action::PlaybackSet(false) => imp.player.set_playback(PlaybackState::Stopped),
            Action::PlaybackToggle => imp.player.toggle_playback(),
            Action::PlaybackSetVolume(volume) => imp.player.set_volume(volume),
            Action::PlaybackSaveSong(song) => imp.player.save_song(song),
            Action::SettingsKeyChanged(key) => self.apply_settings_changes(key),
        }
        glib::Continue(true)
    }

    fn apply_settings_changes(&self, key: Key) {
        debug!("Settings key changed: {:?}", &key);
        match key {
            Key::ViewSorting | Key::ViewOrder => {
                let value = settings_manager::string(Key::ViewSorting);
                let sorting = SwSorting::from_str(&value).unwrap();
                let order = settings_manager::string(Key::ViewOrder);
                let descending = order == "Descending";

                self.imp()
                    .window
                    .get()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .set_sorting(sorting, descending);
            }
            Key::DarkMode => self.update_color_scheme(),
            _ => (),
        }
    }

    fn update_color_scheme(&self) {
        let manager = adw::StyleManager::default();
        if !manager.system_supports_color_schemes() {
            let color_scheme = if settings_manager::boolean(Key::DarkMode) {
                adw::ColorScheme::PreferDark
            } else {
                adw::ColorScheme::PreferLight
            };
            manager.set_color_scheme(color_scheme);
        }
    }

    pub fn refresh_data(&self) {
        let fut = clone!(@weak self as this => async move {
            let imp = this.imp();
            let window = SwApplicationWindow::default();

            if let Some(server) = Client::api_server().await{
                imp.library.refresh_data(Some(&server));
                window.refresh_data(&server);
                window.enable_offline_mode(false);
            }else{
                imp.library.refresh_data(None);
                window.enable_offline_mode(true);
            }
        });
        spawn!(fut);
    }
}

impl Default for SwApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
