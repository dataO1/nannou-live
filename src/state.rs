use nannou::prelude::*;
use crate::audio::AudioEngine;
use crate::sketch::{SketchManager, SketchHandle};

pub struct LiveState {
    pub audio: AudioEngine,
    pub sketches: SketchManager,
    pub active: SketchHandle,
    pub window: window::Id,
}

impl LiveState {
    pub fn model(app: &nannou::App) -> Self {
        let window = app
            .new_window()
            .size(1280, 720)
            .title("nannou-live")
            .view(view)
            .build()
            .unwrap();

        let audio = AudioEngine::new();
        let mut sketches = SketchManager::new();
        let active = sketches.load_all();

        LiveState { audio, sketches, active, window }
    }

    pub fn update(app: &nannou::App, model: &mut Self, update: Update) {
        // Process audio
        model.audio.update();

        // Update active sketch
        let audio_features = model.audio.features();
        model.sketches.update(model.active, update, audio_features);

        // Scene switching: 1-9 keys
        // Handled in view() via app.keys
    }
}

fn view(app: &nannou::App, model: &App, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    model.sketches.view(model.active, &draw, frame.rect());

    draw.to_frame(app, &frame).unwrap();
}
