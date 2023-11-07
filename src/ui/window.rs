// Shortwave - window.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass, Enum, ParamFlags, ParamSpec, ParamSpecEnum, Sender, ToValue};
use gtk::{gdk, gio, glib, CompositeTemplate};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use url::Url;

use crate::app::{Action, SwApplication};
use crate::audio::Player;
use crate::config;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key};
use crate::ui::pages::*;
use crate::ui::SwCreateStationDialog;

#[derive(Display, Copy, Debug, Clone, EnumString, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SwView")]
pub enum SwView {
    Library,
    Discover,
    Search,
    Player,
}

impl Default for SwView {
    fn default() -> Self {
        SwView::Library
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/window.ui")]
    pub struct SwApplicationWindow {
        #[template_child]
        pub library_page: TemplateChild<SwLibraryPage>,
        #[template_child]
        pub discover_page: TemplateChild<SwDiscoverPage>,
        #[template_child]
        pub search_page: TemplateChild<SwSearchPage>,

        #[template_child]
        pub connection_infobar: TemplateChild<gtk::InfoBar>,
        #[template_child]
        pub mini_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub window_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub window_flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub appmenu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub default_menu: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub library_menu: TemplateChild<gio::MenuModel>,

        pub window_animation_x: OnceCell<adw::TimedAnimation>,
        pub window_animation_y: OnceCell<adw::TimedAnimation>,
        pub view: RefCell<SwView>,
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
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecEnum::new(
                    "view",
                    "View",
                    "View",
                    SwView::static_type(),
                    SwView::default() as i32,
                    ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "view" => obj.view().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "view" => obj.set_view(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    // Implement Gtk.Widget for SwApplicationWindow
    impl WidgetImpl for SwApplicationWindow {}

    // Implement Gtk.Window for SwApplicationWindow
    impl WindowImpl for SwApplicationWindow {}

    // Implement Gtk.ApplicationWindow for SwApplicationWindow
    impl ApplicationWindowImpl for SwApplicationWindow {}

    // Implement Adw.ApplicationWindow for SwApplicationWindow
    impl AdwApplicationWindowImpl for SwApplicationWindow {}
}

// Wrap imp::SwApplicationWindow into a usable gtk-rs object
glib::wrapper! {
    pub struct SwApplicationWindow(
        ObjectSubclass<imp::SwApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

// SwApplicationWindow implementation itself
impl SwApplicationWindow {
    pub fn new(sender: Sender<Action>, app: SwApplication, player: Rc<Player>) -> Self {
        // Create new GObject and downcast it into SwApplicationWindow
        let window = glib::Object::new::<Self>(&[]).unwrap();
        app.add_window(&window);

        window.setup_widgets(sender.clone(), player);
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);

        // Library is the default page
        window.set_view(SwView::Library);

        window
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
        imp.window_flap.set_flap(Some(&player.widget));

        // Animations for smooth mini player transitions
        let x_callback =
            adw::CallbackAnimationTarget::new(clone!(@weak self as this => move |val|{
                this.set_default_width(val as i32);
            }));
        let x_animation = adw::TimedAnimation::new(self, 0.0, 0.0, 500, &x_callback);
        x_animation.set_easing(adw::Easing::EaseOutCubic);
        imp.window_animation_x.set(x_animation).unwrap();

        let y_callback =
            adw::CallbackAnimationTarget::new(clone!(@weak self as this => move |val|{
                this.set_default_height(val as i32);
            }));
        let y_animation = adw::TimedAnimation::new(self, 0.0, 0.0, 500, &y_callback);
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

    fn setup_signals(&self, _sender: Sender<Action>) {
        let imp = self.imp();

        // flap
        imp.window_flap
            .get()
            .connect_folded_notify(clone!(@strong self as this => move |_| {
                this.update_visible_view();
            }));
        imp.window_flap
            .get()
            .connect_reveal_flap_notify(clone!(@strong self as this => move |_| {
                this.update_visible_view();
            }));

        // search_button
        imp.search_button
            .connect_toggled(clone!(@strong self as this => move |search_button| {
                if search_button.is_active(){
                    this.set_view(SwView::Search);
                }else if *this.imp().view.borrow() != SwView::Player {
                    this.set_view(SwView::Discover);
                }
            }));

        // window gets closed
        self.connect_close_request(move |window| {
            debug!("Saving window geometry.");
            let width = window.default_size().0;
            let height = window.default_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            glib::signal::Inhibit(false)
        });
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        let imp = self.imp();
        let app = self.application().unwrap();

        // win.open-radio-browser-info
        action!(self, "open-radio-browser-info", |_, _| {
            gtk::show_uri(
                Some(&SwApplicationWindow::default()),
                "https://www.radio-browser.info/",
                gdk::CURRENT_TIME,
            );
        });

        // win.create-new-station
        action!(
            self,
            "create-new-station",
            clone!(@strong sender => move |_, _| {
                let dialog = SwCreateStationDialog::new(sender.clone());
                dialog.show();
            })
        );

        // win.go-back
        action!(
            self,
            "go-back",
            clone!(@weak self as this => move |_, _| {
                this.go_back();
            })
        );
        app.set_accels_for_action("win.go-back", &["Escape"]);

        // win.show-discover
        action!(
            self,
            "show-discover",
            clone!(@weak self as this => move |_, _| {
                this.set_view(SwView::Discover);
            })
        );
        app.set_accels_for_action("win.show-discover", &["<primary>d"]);

        // win.show-search
        action!(
            self,
            "show-search",
            clone!(@weak self as this => move |_, _| {
                this.set_view(SwView::Search);
            })
        );
        app.set_accels_for_action("win.show-search", &["<primary>f"]);

        // win.show-library
        action!(
            self,
            "show-library",
            clone!(@weak self as this => move |_, _| {
                this.set_view(SwView::Library);
            })
        );
        app.set_accels_for_action("win.show-library", &["<primary>l"]);

        // win.show-appmenu
        action!(
            self,
            "show-appmenu",
            clone!(@strong imp.appmenu_button as appmenu_button => move |_, _| {
                appmenu_button.popup();
            })
        );
        app.set_accels_for_action("win.show-appmenu", &["F10"]);

        // win.toggle-playback
        action!(
            self,
            "toggle-playback",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::PlaybackToggle);
            })
        );
        app.set_accels_for_action("win.toggle-playback", &["<primary>space"]);

        // win.disable-mini-player
        action!(
            self,
            "disable-mini-player",
            clone!(@weak self as this => move |_, _| {
                this.enable_mini_player(false);
            })
        );

        // win.enable-mini-player
        action!(
            self,
            "enable-mini-player",
            clone!(@weak self as this => move |_, _| {
                this.enable_mini_player(true);
            })
        );

        // win.refresh-data
        action!(self, "refresh-data", |_, _| {
            SwApplication::default().refresh_data();
        });
        app.set_accels_for_action("win.refresh-data", &["<primary>r"]);

        // Sort / Order menu
        let sorting_action = settings_manager::create_action(Key::ViewSorting);
        self.add_action(&sorting_action);

        let order_action = settings_manager::create_action(Key::ViewOrder);
        self.add_action(&order_action);
    }

    pub fn refresh_data(&self, server: &Url) {
        let imp = self.imp();

        imp.discover_page.refresh_data(server);
        imp.search_page.refresh_data(server);
    }

    pub fn show_player_widget(&self) {
        let imp = self.imp();

        imp.toolbar_controller_revealer.set_visible(true);
        imp.window_flap.set_locked(false);

        self.update_visible_view();
    }

    pub fn show_notification(&self, text: &str) {
        let toast = adw::Toast::new(text);
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub fn set_sorting(&self, sorting: SwSorting, descending: bool) {
        self.imp()
            .library_page
            .get()
            .set_sorting(sorting, descending);
    }

    pub fn view(&self) -> SwView {
        *self.imp().view.borrow()
    }

    pub fn set_view(&self, view: SwView) {
        *self.imp().view.borrow_mut() = view;

        // Delay updating the view, otherwise it could invalidate widgets if it gets
        // called during an allocation and cause glitches (eg. short flickering)
        glib::idle_add_local(
            clone!(@weak self as this => @default-return glib::Continue(false), move||{
                this.update_view(); glib::Continue(false)
            }),
        );
    }

    pub fn enable_offline_mode(&self, enable: bool) {
        self.imp().connection_infobar.set_revealed(enable);

        if enable {
            self.set_view(SwView::Library);
        }

        // Disable discover/search since those are useless
        // if there's no connectivity to radio-browser.info
        let action: gio::SimpleAction = self
            .lookup_action("show-discover")
            .unwrap()
            .downcast()
            .unwrap();
        action.set_enabled(!enable);
        let action: gio::SimpleAction = self
            .lookup_action("show-search")
            .unwrap()
            .downcast()
            .unwrap();
        action.set_enabled(!enable);
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

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let imp = self.imp();

        if *imp.view.borrow() == SwView::Player {
            imp.window_flap.set_reveal_flap(false);
        } else {
            imp.window_leaflet.navigate(adw::NavigationDirection::Back);
        }

        self.update_visible_view();
    }

    fn update_visible_view(&self) {
        let imp = self.imp();

        let view = if imp.window_flap.is_folded() && imp.window_flap.reveals_flap() {
            SwView::Player
        } else {
            let leaflet_child = imp.window_leaflet.visible_child().unwrap();
            if leaflet_child == imp.library_page.get() {
                SwView::Library
            } else if leaflet_child == imp.discover_page.get() {
                SwView::Discover
            } else if leaflet_child == imp.search_page.get() {
                SwView::Search
            } else {
                panic!("Unknown leaflet child")
            }
        };

        debug!("Update visible view to {:?}", view);
        self.set_view(view);
    }

    fn update_view(&self) {
        let imp = self.imp();
        let view = *imp.view.borrow();
        debug!("Set view to {:?}", view);

        // Not enough place to display player sidebar and content side by side (eg.
        // mobile phones)
        let slim_mode = imp.window_flap.is_folded();
        // Whether the player widgets (sidebar / bottom toolbar) should get display or
        // not.
        let player_activated = !imp.window_flap.is_locked();

        if player_activated {
            if slim_mode && view == SwView::Player {
                imp.window_flap.set_reveal_flap(true);
                imp.toolbar_controller_revealer.set_reveal_child(false);
            } else if slim_mode {
                imp.window_flap.set_reveal_flap(false);
                imp.toolbar_controller_revealer.set_reveal_child(true);
            } else {
                imp.window_flap.set_reveal_flap(true);
                imp.toolbar_controller_revealer.set_reveal_child(false);
            }
        }

        // Show requested view / page
        match view {
            SwView::Library => {
                imp.window_leaflet
                    .set_visible_child(&imp.library_page.get());
                imp.appmenu_button
                    .set_menu_model(Some(&imp.library_menu.get()));
                imp.search_revealer.set_reveal_child(false);
                imp.add_button.set_visible(true);
                imp.back_button.set_visible(false);
            }
            SwView::Discover => {
                imp.window_leaflet
                    .set_visible_child(&imp.discover_page.get());
                imp.appmenu_button
                    .set_menu_model(Some(&imp.default_menu.get()));
                imp.search_button.set_active(false);
                imp.search_revealer.set_reveal_child(true);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
            SwView::Search => {
                imp.window_leaflet.set_visible_child(&imp.search_page.get());
                imp.appmenu_button
                    .set_menu_model(Some(&imp.default_menu.get()));
                imp.search_button.set_active(true);
                imp.search_revealer.set_reveal_child(true);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
            SwView::Player => {
                imp.window_flap.set_reveal_flap(true);
                imp.search_button.set_active(false);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
        }
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
