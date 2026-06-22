use nannou::prelude::*;
use crate::audio::AudioFeatures;
use crate::sketch::Sketch;

pub struct Folds {
    name: String,
    params: [f32; 16],
    // TODO: WGSL pipeline, feedback texture
}

impl Folds {
    pub fn new() -> Self {
        let mut params = [0.5; 16];
        params[0] = 0.4;  // folds
        params[1] = 0.35; // rotation
        params[2] = 0.55; // zoom
        params[3] = 0.45; // complexity
        params[4] = 0.4;  // distortion
        Folds {
            name: "Folds".into(),
            params,
        }
    }
}

impl Sketch for Folds {
    fn name(&self) -> &str { &self.name }

    fn update(&mut self, _update: Update, _audio: &AudioFeatures, _params: &[f32; 16]) {
        // TODO: update uniforms, run compute pass
    }

    fn view(&self, draw: &Draw, rect: Rect, _audio: &AudioFeatures, _params: &[f32; 16]) {
        draw.background().color(rgb(0.02, 0.015, 0.01));
        draw.text("Folds — WIP")
            .color(WHITE)
            .font_size(24)
            .xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
