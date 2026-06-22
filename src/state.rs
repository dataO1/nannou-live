use nannou::prelude::*;
use crate::audio::AudioEngine;
use crate::sketch::{SketchManager, SketchHandle};

pub struct LiveState {
    pub audio: AudioEngine,
    pub sketches: SketchManager,
    pub active: SketchHandle,
    pub window: window::Id,

    // OSC placeholder — receives messages from DJ software
    #[cfg(feature = "osc")]
    pub osc_rx: Option<std::sync::mpsc::Receiver<crate::osc::OscCommand>>,
}

impl LiveState {
    pub fn model(app: &nannou::App) -> Self {
        let window = app
            .new_window()
            .size(1280, 720)
            .title("nannou-live")
            .view(view)
            .key_pressed(key_pressed)
            .build()
            .unwrap();

        let audio = AudioEngine::new();
        let mut sketches = SketchManager::new();
        let active = sketches.load_all(app, window);

        #[cfg(feature = "osc")]
        let osc_rx = crate::osc::spawn_listener();

        LiveState {
            audio,
            sketches,
            active,
            window,
            #[cfg(feature = "osc")]
            osc_rx,
        }
    }

    pub fn update(app: &nannou::App, model: &mut Self, update: Update) {
        // Process audio → features
        model.audio.update();
        let features = model.audio.features().clone();

        #[cfg(feature = "osc")]
        {
            // Drain OSC commands from DJ software
            if let Some(ref rx) = model.osc_rx {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        crate::osc::OscCommand::Scene(name) => {
                            log::info!("OSC: switch scene → {name}");
                            model.sketches.switch_to(&name, &mut model.active);
                        }
                        crate::osc::OscCommand::Param { index, value } => {
                            model.sketches.set_param(model.active, index, value);
                        }
                        crate::osc::OscCommand::Beat => {
                            // Inject beat into audio features
                        }
                    }
                }
            }
        }

        // Update active sketch
        model.sketches.update(app, model.active, &update, &features);
    }
}

fn view(app: &nannou::App, model: &LiveState, frame: Frame) {
    model.sketches.view_frame(model.active, &frame);
    // Fallback: draw-based sketches still work
    let draw = app.draw();
    let features = model.audio.features();
    model.sketches.view(model.active, &draw, frame.rect(), features);
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(app: &nannou::App, model: &mut LiveState, key: Key) {
    match key {
        Key::Key1 => model.sketches.switch_to_index(0, &mut model.active),
        Key::Key2 => model.sketches.switch_to_index(1, &mut model.active),
        Key::Key3 => model.sketches.switch_to_index(2, &mut model.active),
        Key::F => {
            let window = app.window(model.window).unwrap();
            let is_fs = window.is_fullscreen();
            window.set_fullscreen(!is_fs);
        }
        _ => {}
    }
}
