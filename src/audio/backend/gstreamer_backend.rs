// Shortwave - gstreamer_backend.rs
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

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use glib::clone;
use gstreamer::prelude::*;
use gstreamer::{Bin, Element, MessageView, PadProbeReturn, PadProbeType, Pipeline, State};
use gstreamer_audio::{StreamVolume, StreamVolumeFormat};
use gtk::glib;
use gtk::glib::Sender;
use once_cell::unsync::OnceCell;

use crate::app::Action;
use crate::audio::PlaybackState;

#[rustfmt::skip]
////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                //
//  # Gstreamer Pipeline                                                                          //
//                                           -----     (   -------------   )                      //
//                                          |     | -> (  | recorderbin |  )                      //
//   --------------      --------------     |     |    (   -------------   )                      //
//  | uridecodebin | -> | audioconvert | -> | tee |                                               //
//   --------------      --------------     |     |     -------      ---------------------------  //
//                                          |     | -> | queue | -> | pulsesink | autoaudiosink | //
//                                           -----      -------      ---------------------------  //
//                                                                                                //
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum GstreamerMessage {
    SongTitleChanged(String),
    PlaybackStateChanged(PlaybackState),
}

#[derive(Default)]
struct BufferingState {
    buffering: bool,
    buffering_probe: Option<(gstreamer::Pad, gstreamer::PadProbeId)>,
    is_live: Option<bool>,
}

pub struct GstreamerBackend {
    pipeline: Pipeline,
    recorderbin: Arc<Mutex<Option<Bin>>>,
    current_title: Arc<Mutex<String>>,
    volume: Arc<Mutex<f64>>,
    volume_signal_id: Option<glib::signal::SignalHandlerId>,
    buffering_state: Arc<Mutex<BufferingState>>,
    bus_watch_guard: OnceCell<gstreamer::bus::BusWatchGuard>,
    sender: Sender<GstreamerMessage>,
}

impl GstreamerBackend {
    pub fn new(gst_sender: Sender<GstreamerMessage>, app_sender: Sender<Action>) -> Self {
        // Determine if env supports pulseaudio
        let audiosink = if Self::check_pulse_support() {
            "pulsesink"
        } else {
            // If not, use autoaudiosink as fallback
            warn!("Cannot find PulseAudio. Shortwave will only work with limited functions.");
            "autoaudiosink"
        };

        // create gstreamer pipeline
        let pipeline_launch = format!(
            "uridecodebin name=uridecodebin use-buffering=true buffer-duration=6000000000 ! audioconvert name=audioconvert ! tee name=tee ! queue ! {audiosink} name={audiosink}"
        );
        let pipeline = gstreamer::parse_launch(&pipeline_launch).unwrap();
        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().unwrap();
        pipeline.set_message_forward(true);

        // The recorderbin gets added / removed dynamically to the pipeline
        let recorderbin = Arc::new(Mutex::new(None));

        // Current song title
        // We need this variable to check if the title have changed.
        let current_title = Arc::new(Mutex::new(String::new()));

        // Playback volume
        let volume = Arc::new(Mutex::new(1.0));
        let volume_signal_id = None;

        // Buffering state
        let buffering_state = Arc::new(Mutex::new(BufferingState::default()));

        let mut gstreamer_backend = Self {
            pipeline,
            recorderbin,
            current_title,
            volume,
            volume_signal_id,
            buffering_state,
            bus_watch_guard: OnceCell::default(),
            sender: gst_sender,
        };

        gstreamer_backend.setup_signals(app_sender);
        gstreamer_backend
    }

    fn setup_signals(&mut self, app_sender: Sender<Action>) {
        // There's no volume support for non pulseaudio systems
        if let Some(pulsesink) = self.pipeline.by_name("pulsesink") {
            // We have to update the volume if we get changes from pulseaudio (pulsesink).
            // The user is able to control the volume from g-c-c.
            let (volume_sender, volume_receiver) = glib::MainContext::channel(glib::Priority::LOW);

            // We need to do message passing (sender/receiver) here, because gstreamer
            // messages are coming from a other thread (and app::Action enum is
            // not thread safe).
            volume_receiver.attach(
                None,
                clone!(@strong app_sender => move |volume| {
                    send!(app_sender, Action::PlaybackSetVolume(volume));
                    glib::ControlFlow::Continue
                }),
            );

            // Update volume coming from pulseaudio / pulsesink
            self.volume_signal_id = Some(pulsesink.connect_notify(
                Some("volume"),
                clone!(@weak self.volume as old_volume, @strong volume_sender => move |element, _| {
                    let pa_volume: f64 = element.property("volume");
                    let new_volume = StreamVolume::convert_volume(StreamVolumeFormat::Linear, StreamVolumeFormat::Cubic, pa_volume);

                    // We have to check if the values are the same. For some reason gstreamer sends us
                    // slightly different floats, so we round up here (only the the first two digits are
                    // important for use here).
                    let mut old_volume_locked = old_volume.lock().unwrap();
                    let new_val = format!("{new_volume:.2}");
                    let old_val = format!("{old_volume_locked:.2}");

                    if new_val != old_val {
                        send!(volume_sender, new_volume);
                        *old_volume_locked = new_volume;
                    }
                }),
            ));

            // It's possible to mute the audio (!= 0.0) from pulseaudio side, so we should
            // handle this too by setting the volume to 0.0
            pulsesink.connect_notify(
                Some("mute"),
                clone!(@weak self.volume as old_volume, @strong volume_sender => move |element, _| {
                    let mute: bool = element.property("mute");
                    let mut old_volume_locked = old_volume.lock().unwrap();
                    if mute && *old_volume_locked != 0.0 {
                        send!(volume_sender, 0.0);
                        *old_volume_locked = 0.0;
                    }
                }),
            );
        }

        // dynamically link uridecodebin element with audioconvert element
        let uridecodebin = self.pipeline.by_name("uridecodebin").unwrap();
        let audioconvert = self.pipeline.by_name("audioconvert").unwrap();
        uridecodebin.connect_pad_added(clone!(@weak audioconvert => move |_, src_pad| {
            let sink_pad = audioconvert.static_pad("sink").expect("Failed to get static sink pad from audioconvert");
            if sink_pad.is_linked() {
                return; // We are already linked. Ignoring.
            }

            let new_pad_caps = src_pad.current_caps().expect("Failed to get caps of new pad.");
            let new_pad_struct = new_pad_caps.structure(0).expect("Failed to get first structure of caps.");
            let new_pad_type = new_pad_struct.name();

            if new_pad_type.starts_with("audio/x-raw") {
                // check if new_pad is audio
                let _ = src_pad.link(&sink_pad);
            }
        }));

        // listen for new pipeline / bus messages
        let bus = self.pipeline.bus().expect("Unable to get pipeline bus");
        let guard = bus.add_watch_local(
            clone!(@weak self.pipeline as pipeline, @strong self.sender as gst_sender, @strong self.buffering_state as buffering_state, @weak self.current_title as current_title => @default-panic, move |_, message|{
                Self::parse_bus_message(pipeline, message, gst_sender.clone(), &buffering_state, current_title);
                glib::ControlFlow::Continue
            }),
        )
        .unwrap();
        self.bus_watch_guard.set(guard).unwrap();
    }

    pub fn set_state(&mut self, state: gstreamer::State) {
        debug!("Set playback state: {:?}", state);

        if state == gstreamer::State::Null {
            send!(
                self.sender,
                GstreamerMessage::PlaybackStateChanged(PlaybackState::Stopped)
            );
        }

        let res = self.pipeline.set_state(state);

        if state > gstreamer::State::Null && res.is_err() {
            warn!("Failed to set pipeline to playing");
            send!(
                self.sender,
                GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(String::from(
                    "Failed to set pipeline to playing"
                )))
            );
            let _ = self.pipeline.set_state(gstreamer::State::Null);
            return;
        }

        if state >= gstreamer::State::Paused {
            let mut buffering_state = self.buffering_state.lock().unwrap();
            if buffering_state.is_live.is_none() {
                let is_live = res == Ok(gstreamer::StateChangeSuccess::NoPreroll);
                debug!("Pipeline is live: {}", is_live);
                buffering_state.is_live = Some(is_live);
            }
        }
    }

    pub fn state(&self) -> PlaybackState {
        let state = self
            .pipeline
            .state(gstreamer::ClockTime::from_mseconds(250))
            .1;
        match state {
            gstreamer::State::Playing => PlaybackState::Playing,
            _ => PlaybackState::Stopped,
        }
    }

    pub fn set_volume(&self, volume: f64) {
        if let Some(pulsesink) = self.pipeline.by_name("pulsesink") {
            // We need to block the signal, otherwise we risk creating a endless loop
            glib::signal::signal_handler_block(&pulsesink, self.volume_signal_id.as_ref().unwrap());

            if volume != 0.0 {
                pulsesink.set_property("mute", false);
            }

            let pa_volume = StreamVolume::convert_volume(
                StreamVolumeFormat::Cubic,
                StreamVolumeFormat::Linear,
                volume,
            );
            pulsesink.set_property("volume", pa_volume);

            *self.volume.lock().unwrap() = volume;

            // Unblock the signal again
            glib::signal::signal_handler_unblock(
                &pulsesink,
                self.volume_signal_id.as_ref().unwrap(),
            );
        } else {
            warn!("PulseAudio is required for changing the volume.")
        }
    }

    pub fn new_source_uri(&mut self, source: &str) {
        debug!("Stop pipeline...");
        let _ = self.pipeline.set_state(State::Null);

        debug!("Set new source URI...");
        let uridecodebin = self.pipeline.by_name("uridecodebin").unwrap();
        uridecodebin.set_property("uri", source);

        debug!("Start pipeline...");
        let mut buffering_state = self.buffering_state.lock().unwrap();
        *buffering_state = BufferingState::default();
        let res = self.pipeline.set_state(State::Playing);

        if res.is_err() {
            warn!("Failed to set pipeline to playing");
            send!(
                self.sender,
                GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(String::from(
                    "Failed to set pipeline to playing"
                )))
            );
            let _ = self.pipeline.set_state(gstreamer::State::Null);
            return;
        }

        let is_live = res == Ok(gstreamer::StateChangeSuccess::NoPreroll);
        debug!("Pipeline is live: {}", is_live);
        buffering_state.is_live = Some(is_live);
    }

    pub fn start_recording(&mut self, path: PathBuf) {
        if self.is_recording() {
            warn!("Unable to start recording: Already recording");
            return;
        }
        debug!("Creating new recorderbin...");

        // Create actual recorderbin
        let description =
            "queue name=queue ! vorbisenc ! oggmux  ! filesink name=filesink async=false";
        let recorderbin = gstreamer::parse_bin_from_description(description, true)
            .expect("Unable to create recorderbin");
        recorderbin.set_property("message-forward", true);

        // We need to set an offset, otherwise the length of the recorded song would be
        // wrong. Get current clock time and calculate offset
        let offset = Self::calculate_pipeline_offset(&self.pipeline);
        let queue_srcpad = recorderbin
            .by_name("queue")
            .unwrap()
            .static_pad("src")
            .unwrap();
        queue_srcpad.set_offset(offset);

        // Set recording path
        let filesink = recorderbin.by_name("filesink").unwrap();
        filesink.set_property("location", path.to_str().unwrap());

        // First try setting the recording bin to playing: if this fails we know this
        // before it potentially interfered with the other part of the pipeline
        recorderbin
            .set_state(gstreamer::State::Playing)
            .expect("Failed to start recording");

        // Add new recorderbin to the pipeline
        self.pipeline
            .add(&recorderbin)
            .expect("Unable to add recorderbin to pipeline");

        // Get our tee element by name, request a new source pad from it and then link
        // that to our recording bin to actually start receiving data
        let tee = self.pipeline.by_name("tee").unwrap();
        let tee_srcpad = tee
            .request_pad_simple("src_%u")
            .expect("Failed to request new pad from tee");
        let sinkpad = recorderbin
            .static_pad("sink")
            .expect("Failed to get sink pad from recorderbin");

        // Link tee srcpad with the sinkpad of the recorderbin
        tee_srcpad
            .link(&sinkpad)
            .expect("Unable to link tee srcpad with recorderbin sinkpad");

        *self.recorderbin.lock().unwrap() = Some(recorderbin);
        debug!("Started recording to {:?}", path);
    }

    pub fn stop_recording(&mut self, discard_data: bool) {
        let recorderbin = self.recorderbin.lock().unwrap().take();
        let recorderbin = match recorderbin {
            None => {
                warn!("Unable to stop recording: No recording running");
                return;
            }
            Some(bin) => bin,
        };

        debug!(
            "Stop recording... (Discard recorded data: {:?})",
            &discard_data
        );

        // Get the source pad of the tee that is connected to the recorderbin
        let recorderbin_sinkpad = recorderbin
            .static_pad("sink")
            .expect("Failed to get sink pad from recorderbin");
        let tee_srcpad = match recorderbin_sinkpad.peer() {
            Some(peer) => peer,
            None => return,
        };

        // Once the tee source pad is idle and we wouldn't interfere with any data flow,
        // unlink the tee and the recording bin and finalize the recording bin
        // by sending it an end-of-stream event
        //
        // Once the end-of-stream event is handled by the whole recording bin, we get an
        // end-of-stream message from it in the message handler and the shut down the
        // recording bin and remove it from the pipeline
        tee_srcpad.add_probe(
            PadProbeType::IDLE,
            clone!(@weak self.pipeline as pipeline => @default-panic, move |tee_srcpad, _| {
                // Get the parent of the tee source pad, i.e. the tee itself
                let tee = tee_srcpad
                    .parent()
                    .and_then(|parent| parent.downcast::<Element>().ok())
                    .expect("Failed to get tee source pad parent");

                // Unlink the tee source pad and then release it
                let _ = tee_srcpad.unlink(&recorderbin_sinkpad);
                tee.release_request_pad(tee_srcpad);

                if !discard_data {
                    // Asynchronously send the end-of-stream event to the sinkpad as this might block for a
                    // while and our closure here might've been called from the main UI thread
                    let recorderbin_sinkpad = recorderbin_sinkpad.clone();
                    recorderbin.call_async(move |_| {
                        recorderbin_sinkpad.send_event(gstreamer::event::Eos::new());
                        debug!("Sent EOS event to recorderbin sinkpad");
                    });
                }else{
                    Self::destroy_recorderbin(pipeline, recorderbin.clone());
                    debug!("Stopped recording.");
                }

                // Don't block the pad but remove the probe to let everything
                // continue as normal
                PadProbeReturn::Remove
            }),
        );
    }

    pub fn is_recording(&self) -> bool {
        self.recorderbin.lock().unwrap().is_some()
    }

    pub fn current_recording_duration(&self) -> i64 {
        let recorderbin: &Option<Bin> = &self.recorderbin.lock().unwrap();
        if let Some(recorderbin) = recorderbin {
            let queue_srcpad = recorderbin
                .by_name("queue")
                .unwrap()
                .static_pad("src")
                .unwrap();
            let offset = queue_srcpad.offset() / 1_000_000_000;

            let pipeline_time = self
                .pipeline
                .clock()
                .expect("Could not get pipeline clock")
                .time()
                .unwrap()
                .nseconds() as i64
                / 1_000_000_000;
            let result = pipeline_time + offset + 1;

            // Workaround to avoid crash as described in issue #540
            // https://gitlab.gnome.org/World/Shortwave/-/issues/540
            // TODO: Find out actual root cause for this nonsense
            if !(0..=86_400).contains(&result) {
                error!(
                    "Unable to determine correct recording value: {} seconds",
                    result
                );
                return 0;
            }

            return result;
        }

        warn!("No recording active, unable to get recording duration.");
        0
    }

    fn calculate_pipeline_offset(pipeline: &Pipeline) -> i64 {
        let clock_time = pipeline
            .clock()
            .expect("Could not get pipeline clock")
            .time()
            .unwrap()
            .nseconds() as i64;
        let base_time = pipeline
            .base_time()
            .expect("Could not get pipeline base time")
            .nseconds() as i64;

        -(clock_time - base_time)
    }

    fn destroy_recorderbin(pipeline: Pipeline, recorderbin: Bin) {
        // Ignore if the bin was not in the pipeline anymore for whatever
        // reason. It's not a problem
        let _ = pipeline.remove(&recorderbin);

        if let Err(err) = recorderbin.set_state(gstreamer::State::Null) {
            warn!("Failed to stop recording: {}", err);
        }
        debug!("Destroyed recorderbin.");
    }

    fn check_pulse_support() -> bool {
        let pulsesink = gstreamer::ElementFactory::make("pulsesink").build();
        pulsesink.is_ok()
    }

    fn parse_bus_message(
        pipeline: Pipeline,
        message: &gstreamer::Message,
        sender: Sender<GstreamerMessage>,
        buffering_state: &Arc<Mutex<BufferingState>>,
        current_title: Arc<Mutex<String>>,
    ) {
        match message.view() {
            MessageView::Tag(tag) => {
                if let Some(t) = tag.tags().get::<gstreamer::tags::Title>() {
                    let new_title = t.get().to_string();

                    // only send message if song title really have changed.
                    let mut current_title_locked = current_title.lock().unwrap();
                    if *current_title_locked != new_title {
                        *current_title_locked = new_title.clone();
                        send!(sender, GstreamerMessage::SongTitleChanged(new_title));
                    }
                }
            }
            MessageView::StateChanged(sc) => {
                // Only report the state change once the pipeline itself changed a state,
                // not whenever any of the internal elements does that.
                // https://gitlab.gnome.org/World/Shortwave/-/issues/528
                if message.src() == Some(pipeline.upcast_ref::<gstreamer::Object>()) {
                    let playback_state = match sc.current() {
                        gstreamer::State::Playing => PlaybackState::Playing,
                        gstreamer::State::Paused => PlaybackState::Playing,
                        gstreamer::State::Ready => PlaybackState::Loading,
                        _ => PlaybackState::Stopped,
                    };

                    send!(
                        sender,
                        GstreamerMessage::PlaybackStateChanged(playback_state)
                    );
                }
            }
            MessageView::Buffering(buffering) => {
                let percent = buffering.percent();
                debug!("Buffering ({}%)", percent);

                // Wait until buffering is complete before start/resume playing
                let mut buffering_state = buffering_state.lock().unwrap();
                if percent < 100 {
                    if !buffering_state.buffering {
                        buffering_state.buffering = true;
                        send!(
                            sender,
                            GstreamerMessage::PlaybackStateChanged(PlaybackState::Loading)
                        );

                        if buffering_state.is_live == Some(false) {
                            debug!("Pausing pipeline because buffering started");
                            let tee = pipeline.by_name("tee").unwrap();
                            let sinkpad = tee.static_pad("sink").unwrap();
                            let probe_id = sinkpad
                                .add_probe(
                                    gstreamer::PadProbeType::BLOCK
                                        | gstreamer::PadProbeType::BUFFER
                                        | gstreamer::PadProbeType::BUFFER_LIST,
                                    |_pad, _info| {
                                        debug!("Pipeline blocked because of buffering");
                                        gstreamer::PadProbeReturn::Ok
                                    },
                                )
                                .unwrap();

                            buffering_state.buffering_probe = Some((sinkpad, probe_id));
                            let _ = pipeline.set_state(State::Paused);
                        }
                    }
                } else if buffering_state.buffering {
                    buffering_state.buffering = false;
                    send!(
                        sender,
                        GstreamerMessage::PlaybackStateChanged(PlaybackState::Playing)
                    );

                    if buffering_state.is_live == Some(false) {
                        debug!("Resuming pipeline because buffering finished");
                        let _ = pipeline.set_state(State::Playing);
                        if let Some((pad, probe_id)) = buffering_state.buffering_probe.take() {
                            pad.remove_probe(probe_id);
                        }
                    }
                }
            }
            MessageView::Element(element) => {
                // Catch the end-of-stream messages from the filesink
                let structure = element.structure().unwrap();
                if structure.name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap();
                    if let MessageView::Eos(_) = &message.view() {
                        // Get recorderbin from message
                        let recorderbin = match message
                            .src()
                            .and_then(|src| src.clone().downcast::<Bin>().ok())
                        {
                            Some(src) => src,
                            None => return,
                        };

                        // And then asynchronously remove it and set its state to Null
                        pipeline.call_async(move |pipeline| {
                            Self::destroy_recorderbin(pipeline.clone(), recorderbin);
                            debug!("Stopped recording.");
                        });
                    }
                }
            }
            MessageView::Error(err) => {
                let msg = err.error().to_string();
                if let Some(debug) = err.debug() {
                    warn!("Gstreamer Error: {} (debug {})", msg, debug);
                } else {
                    warn!("Gstreamer Error: {}", msg);
                }
                send!(
                    sender,
                    GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(msg))
                );
            }
            _ => (),
        };
    }
}
