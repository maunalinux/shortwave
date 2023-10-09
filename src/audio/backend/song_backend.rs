// Shortwave - song_backend.rs
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

use std::fs;
use std::rc::Rc;

use glib::Sender;
use gtk::glib;
use indexmap::IndexMap;

use crate::api::Error;
use crate::app::Action;
use crate::audio::Song;
use crate::i18n::i18n;
use crate::path;
use crate::settings::{settings_manager, Key};
use crate::ui::{SongListBox, SwApplicationWindow};

pub struct SongBackend {
    pub listbox: SongListBox,

    songs: IndexMap<String, Song>,
    save_count: usize,
}

// songs: IndexMap<String, Song>,
// 1 - Song E    <- Oldest song, next to remove
// 2 ...
// 3 ...
// 4 ...
// 5 - Song A    <- Newest song

impl SongBackend {
    pub fn new(sender: Sender<Action>, save_count: usize) -> Self {
        let listbox = SongListBox::new(sender);
        let songs = IndexMap::new();

        Self {
            listbox,
            songs,
            save_count,
        }
    }

    pub fn add_song(&mut self, song: Song) {
        // Check if song does not exist yet
        if self.songs.get(&song.title).is_none() {
            // Ensure max length
            if self.songs.len() >= self.save_count {
                // Get oldest song to remove
                let song = self.songs.get_index(0).unwrap().1.clone();
                self.remove_song(song);
            }

            // Add song to indexmap & listbox
            self.listbox.add_song(song.clone());
            self.songs.insert(song.title.to_string(), song);
        } else {
            warn!("Song \"{}\" is already recorded", song.title);
        }
    }

    fn remove_song(&mut self, song: Song) {
        // Remove song from indexmap
        self.songs.shift_remove(&song.title);

        // Remove recorded data from disk
        fs::remove_file(&song.path).expect("Could not delete old song from disk.");

        // Removes the last row in song listbox
        self.listbox.remove_last_row();
    }

    pub fn save_song(&self, song: Song) -> Result<(), Error> {
        debug!("Save song \"{}\"", &song.title);

        // For some unknown reasons some users don't have a xdg-music dir?
        // See: https://gitlab.gnome.org/World/Shortwave/-/issues/676
        let music_dir = glib::user_special_dir(glib::UserDirectory::Music);
        let mut dest_path = if let Some(path) = music_dir {
            path
        } else {
            // Fallback to xdg-home when xdg-music is not available
            let msg = i18n("Unable to access music directory. Saving song in home directory.");
            SwApplicationWindow::default().show_notification(&msg);

            warn!("Unable to access xdg-music directory, falling back to xdg-home");
            glib::home_dir()
        };
        dest_path.push(song.path.file_name().unwrap());

        let custom_path = settings_manager::string(Key::RecorderSongSavePath);
        if !custom_path.is_empty() {
            dest_path.push(custom_path);
            dest_path.push(song.path.file_name().unwrap());
        }

        fs::copy(song.path, dest_path).map_err(Rc::new)?;
        Ok(())
    }

    pub fn delete_songs(&self) {
        let mut path = path::CACHE.clone();
        path.push("recording");

        // Just delete the whole recording dir.
        // It gets recreated automatically
        if path.exists() {
            fs::remove_dir_all(path).expect("Could not delete recording dir.");
        }
    }
}
