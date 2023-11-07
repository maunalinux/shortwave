// Shortwave - song_row.rs
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
use chrono::NaiveTime;
use glib::{clone, subclass, Sender};
use gtk::{gdk, glib, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::app::Action;
use crate::audio::Song;
use crate::ui::SwApplicationWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/song_row.ui")]
    pub struct SwSongRow {
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub open_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub button_stack: TemplateChild<gtk::Stack>,

        pub song: OnceCell<Song>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwSongRow {
        const NAME: &'static str = "SwSongRow";
        type ParentType = adw::ActionRow;
        type Type = super::SwSongRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwSongRow {}

    impl WidgetImpl for SwSongRow {}

    impl ListBoxRowImpl for SwSongRow {}

    impl PreferencesRowImpl for SwSongRow {}

    impl ActionRowImpl for SwSongRow {}
}

glib::wrapper! {
    pub struct SwSongRow(ObjectSubclass<imp::SwSongRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl SwSongRow {
    pub fn new(sender: Sender<Action>, song: Song) -> Self {
        let row = glib::Object::new::<Self>(&[]).unwrap();

        // Set information
        let duration = Self::format_duration(song.duration.as_secs());
        row.set_title(&song.title);
        row.set_tooltip_text(Some(&song.title));
        row.set_subtitle(&duration);

        let imp = row.imp();
        imp.sender.set(sender).unwrap();
        imp.song.set(song).unwrap();

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.save_button
            .connect_clicked(clone!(@weak self as this => move |_| {
                let imp = this.imp();

                // Save the song
                let sender = imp.sender.get().unwrap();
                let song = imp.song.get().unwrap();
                send!(sender, Action::PlaybackSaveSong(song.clone()));

                // Display play button instead of save button
                imp.button_stack.set_visible_child_name("open");
                this.set_activatable_widget(Some(&imp.open_button.get()));
            }));

        imp.open_button
            .connect_clicked(clone!(@strong self as this => move |_| {
                let song = this.imp().song.get().unwrap();
                let path = format!("file://{}", song.path.as_os_str().to_str().unwrap());

                gtk::show_uri(
                    Some(&SwApplicationWindow::default()),
                    &path,
                    gdk::CURRENT_TIME,
                );
            }));
    }

    // stolen from gnome-podcasts
    // https://gitlab.gnome.org/haecker-felix/podcasts/blob/2f8a6a91f87d7fa335a954bbaf2f70694f32f6dd/podcasts-gtk/src/widgets/player.rs#L168
    fn format_duration(seconds: u64) -> String {
        debug!("Format duration (seconds): {}", &seconds);
        let time = NaiveTime::from_num_seconds_from_midnight(seconds as u32, 0);

        if seconds >= 3600 {
            time.format("%T").to_string()
        } else {
            time.format("%M∶%S").to_string()
        }
    }
}
