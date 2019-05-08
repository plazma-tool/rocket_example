use std::error::Error;
use std::time::{Duration, Instant};

use rocket_sync::{SyncDevice, SyncTrack};
use rocket_client::SyncClient;

use crate::error::ToolError;

pub struct State {
    pub t_rocket_last_connection_attempt: Instant,
    pub t_frame_start: Instant,
    pub t_delta: Duration,
    pub t_frame_target: Duration,

    pub is_running: bool,
    pub draw_anyway: bool,

    pub window_resolution: [f32; 2],

    track_names: Vec<String>,
    device: SyncDevice,
}

impl State {

    pub fn new(rocket: &mut Option<SyncClient>, track_names: Vec<String>) -> Result<State, Box<Error>> {
        let mut state = State {
            t_rocket_last_connection_attempt: Instant::now(),
            t_frame_start: Instant::now(),
            t_delta: Duration::new(0, 0),
            t_frame_target: Duration::from_millis(16),
            is_running: true,
            draw_anyway: false,
            window_resolution: [1024.0_f32, 768.0_f32],
            track_names: track_names,
            device: SyncDevice::new(125.0, 8),
        };

        *rocket = match SyncClient::new("localhost:1338") {
            Ok(x) => Some(x),
            Err(_) => None,
        };

        // If Rocket is on, send the track names.
        if let &mut Some(ref mut r) = rocket {
            r.send_track_names(&state.track_names).unwrap();
        }

        state.device.is_paused = true;

        // add empty tracks
        for _ in state.track_names.iter() {
            state.device.tracks.push(SyncTrack::new());
        }

        Ok(state)
    }

    pub fn update_time(&mut self) {
        self.t_frame_start = Instant::now();

        if !self.get_is_paused() {
            let mut d = self.get_sync_device_mut();
            d.time += 16;// 1s / 60 frames
            d.set_row_from_time();
        }
    }

    pub fn update_rocket(&mut self, rocket: &mut Option<SyncClient>) -> Result<(), Box<Error>> {
        let mut do_rocket_none = false;
        if let &mut Some(ref mut r) = rocket {
            match r.update(self.get_sync_device_mut()) {
                Ok(a) => self.draw_anyway = a,
                Err(err) => {
                    do_rocket_none = true;
                    // It's a Box<Error>, so we can't restore the original type.
                    // Let's parse the debug string for now.
                    let msg: &str = &format!("{:?}", err);
                    if msg.contains("kind: UnexpectedEof") {
                        warn!("Rocket disconnected");
                    } else {
                        error!("{}", msg);
                    }
                },
            }
        }

        if do_rocket_none {
            *rocket = None;
        }

        // Try to re-connect to Rocket. Good in the case when the Rocket Editor
        // was started after the tool.
        if rocket.is_none() && self.t_rocket_last_connection_attempt.elapsed() > Duration::from_secs(1) {
            *rocket = match SyncClient::new("localhost:1338") {
                Ok(r) => Some(r),
                Err(_) => None,
            };

            // If Rocket is on, send the track names.
            if let &mut Some(ref mut r) = rocket {
                r.send_track_names(self.get_track_names()).unwrap();
            }

            if rocket.is_some() {
                self.set_is_paused(true);
            }

            self.t_rocket_last_connection_attempt = Instant::now();
        }

        if !self.get_is_paused() {
            if let &mut Some(ref mut r) = rocket {
                match r.send_row(self.get_sync_device_mut()) {
                    Ok(_) => {},
                    Err(e) => warn!("{:?}", e),
                }
            }
        }

        Ok(())
    }

    pub fn get_is_running(&self) -> bool {
        self.is_running
    }

    pub fn set_is_running(&mut self, value: bool) {
        self.is_running = value
    }

    pub fn get_is_paused(&self) -> bool {
        self.device.is_paused
    }

    pub fn set_is_paused(&mut self, value: bool) {
        self.device.is_paused = value;
    }

    pub fn get_sync_device(&self) -> &SyncDevice {
        &self.device
    }

    pub fn get_sync_device_mut(&mut self) -> &mut SyncDevice {
        &mut self.device
    }

    pub fn get_track_names(&self) -> &Vec<String> {
        &self.track_names
    }

    pub fn get_track_value(&self, track_id: usize) -> f32 {
        match self.device.get_track_value(track_id) {
            Ok(x) => x as f32,
            Err(e) => {
                error!("{:?}", ToolError::Sync(e));
                0.0
            }
        }
    }
}
