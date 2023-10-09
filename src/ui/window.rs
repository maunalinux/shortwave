// Shortwave - window.rs
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

use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, Sender};
use gtk::{gdk, gio, glib, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::app::{Action, SwApplication};
use crate::audio::Player;
use crate::config;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key};
use crate::ui::pages::*;
use crate::ui::SwCreateStationDialog;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/window.ui")]
    pub struct SwApplicationWindow {
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,

        #[template_child]
        pub library_page: TemplateChild<SwLibraryPage>,
        #[template_child]
        pub discover_page: TemplateChild<SwDiscoverPage>,
        #[template_child]
        pub search_page: TemplateChild<SwSearchPage>,

        #[template_child]
        pub mini_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        pub window_animation_x: OnceCell<adw::TimedAnimation>,
        pub window_animation_y: OnceCell<adw::TimedAnimation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwApplicationWindow {
        const NAME: &'static str = "SwApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::SwApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let app = SwApplication::default();
            let sender = app.imp().sender.clone();
            let player = app.imp().player.clone();

            self.obj().setup_widgets(sender.clone(), player);
            self.obj().setup_gactions(sender);
        }
    }

    impl WidgetImpl for SwApplicationWindow {}

    impl WindowImpl for SwApplicationWindow {
        fn close_request(&self) -> glib::Propagation {
            debug!("Saving window geometry.");
            let width = self.obj().default_size().0;
            let height = self.obj().default_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            glib::Propagation::Proceed
        }
    }

    impl ApplicationWindowImpl for SwApplicationWindow {}

    impl AdwApplicationWindowImpl for SwApplicationWindow {}

    impl SwApplicationWindow {}
}

glib::wrapper! {
    pub struct SwApplicationWindow(
        ObjectSubclass<imp::SwApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SwApplicationWindow {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn setup_widgets(&self, sender: Sender<Action>, player: Rc<Player>) {
        let imp = self.imp();

        // Init pages
        imp.library_page.init(sender.clone());
        imp.discover_page.init(sender.clone());
        imp.search_page.init(sender);

        // Wire everything up
        imp.mini_controller_box
            .append(&player.mini_controller_widget);
        imp.toolbar_controller_box
            .append(&player.toolbar_controller_widget);
        imp.split_view.set_sidebar(Some(&player.widget));

        // Animations for smooth mini player transitions
        let x_callback =
            adw::CallbackAnimationTarget::new(clone!(@weak self as this => move |val|{
                this.set_default_width(val as i32);
            }));
        let x_animation = adw::TimedAnimation::new(self, 0.0, 0.0, 500, x_callback);
        x_animation.set_easing(adw::Easing::EaseOutCubic);
        imp.window_animation_x.set(x_animation).unwrap();

        let y_callback =
            adw::CallbackAnimationTarget::new(clone!(@weak self as this => move |val|{
                this.set_default_height(val as i32);
            }));
        let y_animation = adw::TimedAnimation::new(self, 0.0, 0.0, 500, y_callback);
        y_animation.set_easing(adw::Easing::EaseOutCubic);
        imp.window_animation_y.set(y_animation).unwrap();

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            self.add_css_class("devel");
        }

        // Restore window geometry
        let width = settings_manager::integer(Key::WindowWidth);
        let height = settings_manager::integer(Key::WindowHeight);
        self.set_default_size(width, height);
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        let app = SwApplication::default();

        self.add_action_entries([
            // win.open-radio-browser-info
            gio::ActionEntry::builder("open-radio-browser-info")
                .activate(|_, _, _| {
                    gtk::show_uri(
                        Some(&SwApplicationWindow::default()),
                        "https://www.radio-browser.info/",
                        gdk::CURRENT_TIME,
                    );
                })
                .build(),
            // win.create-new-station
            gio::ActionEntry::builder("create-new-station")
                .activate(clone!(@strong sender => move |_, _, _| {
                    let dialog = SwCreateStationDialog::new(sender.clone());
                    dialog.show();
                }))
                .build(),
            // win.show-player
            gio::ActionEntry::builder("show-player")
                .activate(clone!(@weak self as this => move |_, _, _| {
                    this.imp().split_view.set_show_sidebar(true);
                }))
                .build(),
            // win.hide-player
            gio::ActionEntry::builder("hide-player")
                .activate(clone!(@weak self as this => move |_, _, _| {
                    this.imp().split_view.set_show_sidebar(false);
                }))
                .build(),
            // win.toggle-playback
            gio::ActionEntry::builder("toggle-playback")
                .activate(clone!(@strong sender => move |_, _, _| {
                    send!(sender, Action::PlaybackToggle);
                }))
                .build(),
            // win.disable-mini-player
            gio::ActionEntry::builder("disable-mini-player")
                .activate(clone!(@weak self as this => move |_, _, _| {
                    this.enable_mini_player(false);
                }))
                .build(),
            // win.enable-mini-player
            gio::ActionEntry::builder("enable-mini-player")
                .activate(clone!(@weak self as this => move |_, _, _| {
                    this.enable_mini_player(true);
                }))
                .build(),
        ]);
        app.set_accels_for_action("win.toggle-playback", &["<primary>space"]);

        // Sort / Order menu
        let sorting_action = settings_manager::create_action(Key::ViewSorting);
        self.add_action(&sorting_action);

        let order_action = settings_manager::create_action(Key::ViewOrder);
        self.add_action(&order_action);
    }

    pub fn show_notification(&self, text: &str) {
        let toast = adw::Toast::new(text);
        self.imp().toast_overlay.add_toast(toast);
    }

    pub fn set_sorting(&self, sorting: SwSorting, descending: bool) {
        self.imp()
            .library_page
            .get()
            .set_sorting(sorting, descending);
    }

    pub fn enable_mini_player(&self, enable: bool) {
        debug!("Enable mini player: {:?}", enable);

        if self.is_maximized() && enable {
            self.unmaximize();
        }

        let mut previous_width = settings_manager::integer(Key::WindowPreviousWidth) as f64;
        let mut previous_height = settings_manager::integer(Key::WindowPreviousHeight) as f64;

        // Save current window size as previous size, so you can restore it
        // if you switch between mini player / normal window mode.
        let current_width = self.default_size().0;
        let current_height = self.default_size().1;
        settings_manager::set_integer(Key::WindowPreviousWidth, current_width);
        settings_manager::set_integer(Key::WindowPreviousHeight, current_height);

        let x_animation = self.imp().window_animation_x.get().unwrap();
        let y_animation = self.imp().window_animation_y.get().unwrap();

        x_animation.reset();
        x_animation.set_value_from(self.width() as f64);
        y_animation.reset();
        y_animation.set_value_from(self.height() as f64);

        if enable {
            if previous_height > 175.0 {
                previous_width = 450.0;
                previous_height = 105.0;
            }

            x_animation.set_value_to(previous_width);
            y_animation.set_value_to(previous_height);
        } else {
            if previous_height < 175.0 {
                previous_width = 950.0;
                previous_height = 650.0;
            }

            x_animation.set_value_to(previous_width);
            y_animation.set_value_to(previous_height);
        }

        x_animation.play();
        y_animation.play();
    }
}

impl Default for SwApplicationWindow {
    fn default() -> Self {
        SwApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}
