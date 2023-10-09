// Shortwave - gcast_controller.rs
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

use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use gtk::glib;
use rust_cast::channels::connection::ConnectionResponse;
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::media::{GenericMediaMetadata, Image, Media, StreamType};
use rust_cast::channels::receiver::{Application, CastDeviceApp};
use rust_cast::{CastDevice, ChannelMessage};

use crate::api::{StationMetadata, SwStation};
use crate::app::Action;
use crate::audio::{Controller, GCastDevice, PlaybackState};

enum GCastAction {
    Connect,
    SetStation,
    HeartBeat,
    Disconnect,
}

pub struct GCastController {
    station_metadata: Arc<Mutex<Option<StationMetadata>>>,
    device_ip: Arc<Mutex<String>>,
    gcast_sender: Sender<GCastAction>,
    app_sender: glib::Sender<Action>,
}

// TODO: Re-structure this mess. Code cleanup is necessary.
// Even clippy starts to complain: "warning: the function has a cognitive
// complexity of (36/25)" Oops.
impl GCastController {
    pub fn new(app_sender: glib::Sender<Action>) -> Rc<Self> {
        let station_metadata = Arc::new(Mutex::new(None));
        let device_ip = Arc::new(Mutex::new("".to_string()));

        let (gcast_sender, gcast_receiver) = channel();
        let gcast_controller = Self {
            station_metadata,
            device_ip,
            gcast_sender,
            app_sender,
        };

        let gcc = Rc::new(gcast_controller);
        gcc.start_thread(gcast_receiver);
        gcc
    }

    fn start_thread(&self, receiver: Receiver<GCastAction>) {
        let station_metadata = self.station_metadata.clone();
        let device_ip = self.device_ip.clone();
        let gcast_sender = self.gcast_sender.clone();

        thread::spawn(move || {
            let mut device: Option<CastDevice> = None;
            let device_ip = device_ip.lock().unwrap().to_string();
            let mut app: Option<Application> = None;
            let mut connected = false;

            loop {
                if let Ok(action) = receiver.recv() {
                    match action {
                        GCastAction::Connect => {
                            debug!(
                                "Connect to gcast device with IP \"{}\"...",
                                device_ip.clone()
                            );
                            match CastDevice::connect_without_host_verification(
                                device_ip.clone(),
                                8009,
                            ) {
                                Ok(d) => {
                                    d.connection.connect("receiver-0".to_string()).unwrap();
                                    d.heartbeat.ping().unwrap();

                                    let app_to_launch = CastDeviceApp::from_str("default").unwrap();
                                    let a = d.receiver.launch_app(&app_to_launch).unwrap();
                                    d.connection.connect(a.transport_id.as_str()).unwrap();

                                    debug!("Connected to gcast device!");
                                    device = Some(d);
                                    app = Some(a);
                                    connected = true;

                                    // We need to set the station, since it already got set before.
                                    send!(gcast_sender, GCastAction::SetStation);
                                }
                                Err(_) => warn!("Could not connect to gcast device."),
                            }
                        }
                        GCastAction::SetStation => {
                            if !connected {
                                // We need to re-connect first
                                send!(gcast_sender, GCastAction::Connect);
                                continue;
                            }
                            if let Some(d) = device.as_ref() {
                                // TODO
                                let s = station_metadata.lock().unwrap().as_ref().unwrap().clone();
                                debug!("Transfer media information to gcast device...");

                                let image = Image {
                                    url: s.clone().favicon.unwrap().to_string(),
                                    dimensions: None,
                                };

                                let metadata = GenericMediaMetadata {
                                    title: Some(s.name.clone()),
                                    subtitle: None,
                                    images: vec![image],
                                    release_date: None,
                                };

                                d.media
                                    .load(
                                        app.as_ref().unwrap().transport_id.as_str(),
                                        app.as_ref().unwrap().session_id.as_str(),
                                        &Media {
                                            content_id: s.url.unwrap().to_string(),
                                            content_type: "".to_string(),
                                            stream_type: StreamType::Live,
                                            duration: None,
                                            metadata: Some(
                                                rust_cast::channels::media::Metadata::Generic(
                                                    metadata,
                                                ),
                                            ),
                                        },
                                    )
                                    .expect(
                                        "Could not transfer media information to gcast device.",
                                    );
                                send!(gcast_sender, GCastAction::HeartBeat);
                            };
                        }
                        GCastAction::HeartBeat => {
                            if !connected {
                                continue;
                            }
                            if let Some(d) = device.as_ref() {
                                match d.receive() {
                                    Ok(ChannelMessage::Heartbeat(response)) => {
                                        debug!("GCast [Heartbeat] {:?}", response);
                                        if let HeartbeatResponse::Ping = response {
                                            d.heartbeat.pong().unwrap();
                                        }
                                    }
                                    Ok(ChannelMessage::Connection(response)) => {
                                        debug!("GCast [Connection] {:?}", response);
                                        if let ConnectionResponse::Close = response {
                                            connected = false;
                                            warn!("GCast [Connection] Closed remotely");
                                        }
                                    }
                                    Ok(ChannelMessage::Media(response)) => {
                                        debug!("GCast [Media] {:?}", response);
                                    }
                                    Ok(ChannelMessage::Receiver(response)) => {
                                        debug!("GCast [Receiver] {:?}", response);
                                    }
                                    Ok(ChannelMessage::Raw(response)) => {
                                        debug!("GCast [Raw] {:?}", response);
                                    }
                                    Err(error) => {
                                        error!("Error occurred while receiving message {}", error)
                                    }
                                };
                                send!(gcast_sender, GCastAction::HeartBeat);
                            }
                        }
                        GCastAction::Disconnect => {
                            if let Some(d) = device.as_ref() {
                                debug!("Disconnect from gcast device...");
                                match d
                                    .receiver
                                    .stop_app(app.as_ref().unwrap().session_id.as_str())
                                {
                                    Ok(_) => connected = false,
                                    _ => warn!("Unable to disconnect from gcast device."),
                                }
                                send!(gcast_sender, GCastAction::HeartBeat);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn connect_to_device(&self, device: GCastDevice) {
        debug!("Called to connect to gcast device...");
        *self.device_ip.lock().unwrap() = device.ip.to_string();
        send!(self.gcast_sender, GCastAction::Connect);

        // Stop audio playback
        send!(self.app_sender, Action::PlaybackSet(false));
    }

    pub fn disconnect_from_device(&self) {
        debug!("Called to disconnect to gcast device...");
        *self.device_ip.lock().unwrap() = "".to_string();
        send!(self.gcast_sender, GCastAction::Disconnect);
    }
}

impl Controller for Rc<GCastController> {
    fn set_station(&self, station: SwStation) {
        *self.station_metadata.lock().unwrap() = Some(station.metadata());

        if !self.device_ip.lock().unwrap().is_empty() {
            debug!("Set new station on gcast device...");
            send!(self.gcast_sender, GCastAction::SetStation);

            // Stop audio playback
            send!(self.app_sender, Action::PlaybackSet(false));
        } else {
            debug!("No device ip available, don't set station. ")
        }
    }

    fn set_playback_state(&self, _playback_state: &PlaybackState) {
        // Ignore
    }

    fn set_volume(&self, _volume: f64) {
        // Ignore
    }

    fn set_song_title(&self, _title: &str) {
        // Ignore
    }
}
