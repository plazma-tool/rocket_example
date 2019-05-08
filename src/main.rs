#![feature(rustc_private)]

use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate glium;
extern crate notify;

extern crate rocket_sync;
extern crate rocket_client;

use glium::{glutin, Surface};
use glium::glutin::{Event, VirtualKeyCode, WindowEvent};

use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::channel;

use rocket_client::SyncClient;

pub mod state;
pub mod error;
pub mod utils;

use state::State;
use utils::file_to_string;

fn main() {
    std::env::set_var("RUST_LOG", "rocket_example,rocket_sync");
    env_logger::init();
    info!("main: started");

    // Setup glium

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        pos: [f32; 2],
        tex: [f32; 2],
    }

    implement_vertex!(Vertex, pos, tex);

    let quad = vec![
        Vertex { pos: [-1.0, -1.0], tex: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0], tex: [0.0, 1.0] },
        Vertex { pos: [ 1.0, -1.0], tex: [1.0, 0.0] },
        Vertex { pos: [ 1.0,  1.0], tex: [1.0, 1.0] },
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &quad).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    // path is relative to PWD, which is the project folder when running with 'cargo run'
    let vertex_shader_src = file_to_string(&PathBuf::from("./data/screen_quad.vert")).unwrap();
    let mut fragment_shader_src = file_to_string(&PathBuf::from("./data/shader.frag")).unwrap();

    let mut program = glium::Program::from_source(&display,
                                                  &vertex_shader_src,
                                                  &fragment_shader_src,
                                                  None).unwrap();

    // Rocket

    let mut rocket: Option<SyncClient> = None;

    let track_names = vec![
        "H_y".to_owned(),
        "i_y".to_owned(),
        "ground_y".to_owned(),
        "interlace".to_owned(),
        "white_noise".to_owned(),
    ];

   // State::new() will connect to a Rocket Editor or keep trying every 1 second

    let mut state: State = State::new(&mut rocket, track_names).unwrap();

    // Always start paused, we are not loading track data on our own and we need
    // Rocket to tell us what to do.
    state.set_is_paused(true);

    // watches every file in the data folder
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(PathBuf::from("./data/"), RecursiveMode::Recursive).unwrap();

    while state.get_is_running() {

        // 1. update time

        state.update_time();
        state.draw_anyway = false;

        // 2. get updates from Rocket

        match state.update_rocket(&mut rocket) {
            Ok(_) => {},
            Err(e) => error!("{:?}", e),
        }


        // 4. sync variable values

        let t = state.get_sync_device().time as f32 / 1000.0;

        let uniforms = uniform! {
            iGlobalTime: t,
            iResolution: state.window_resolution,
            H_y:         state.get_track_value(0),
            i_y:         state.get_track_value(1),
            ground_y:    state.get_track_value(2),
            interlace:   state.get_track_value(3),
            white_noise: state.get_track_value(4),
        };

        // 5. deal with events

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => state.set_is_running(false),

                    WindowEvent::KeyboardInput{ input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            state.set_is_running(false);
                        }
                    },

                    WindowEvent::Resized(wx, wy) => {
                        state.window_resolution = [wx as f32, wy as f32];
                        state.draw_anyway = true;
                    },
                    _ => (),
                },
                _ => (),
            }
        });

        // 6. recompile the shader on change

        match rx.try_recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(full_path) => {
                        info!("Change detected: {:?}", full_path);
                        let p: &str = full_path.to_str().unwrap();
                        if p.ends_with(".frag") {
                            info!("Recompiling the shader");
                            fragment_shader_src = file_to_string(&PathBuf::from("./data/shader.frag")).unwrap();
                            program = glium::Program::from_source(&display,
                                                                  &vertex_shader_src,
                                                                  &fragment_shader_src,
                                                                  None).unwrap();
                            state.draw_anyway = true;
                        }
                    },
                    _ => {},
                }
            },
            Err(_) => {},
        }

        // 7. draw if we are not paused or should draw anyway (e.g. window resized)

        let mut target = display.draw();

        if !state.get_is_paused() || state.draw_anyway {
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        }

        // ship the frame

        target.finish().unwrap();

        // 8. sleep if there is time left

        state.t_delta = state.t_frame_start.elapsed();

        if state.t_delta < state.t_frame_target {
            if let Some(t_sleep) = state.t_frame_target.checked_sub(state.t_delta)  {
                sleep(t_sleep);
            }
        }
    }
}
