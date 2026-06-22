use nannou::prelude::*;
use crate::audio::AudioFeatures;
use crate::sketch::Sketch;

pub struct Chemreact {
    name: String,
    params: [f32; 16],
    // TODO: WGSL pipeline, feedback texture, uniforms
}

impl Chemreact {
    pub fn new() -> Self {
        let mut params = [0.5; 16];
        params[0] = 0.45; // feed_rate
        params[1] = 0.35; // kill_rate
        params[2] = 0.6;  // diff_a
        params[3] = 0.5;  // diff_b
        params[4] = 0.4;  // inject_str
        Chemreact {
            name: "Chemreact".into(),
            params,
        }
    }
}

impl Sketch for Chemreact {
    fn name(&self) -> &str { &self.name }

    fn update(&mut self, _update: Update, _audio: &AudioFeatures, _params: &[f32; 16]) {
        // TODO: update uniforms, run compute pass
    }

    fn view(&self, draw: &Draw, rect: Rect, _audio: &AudioFeatures, _params: &[f32; 16]) {
        // TODO: render WGSL output to fullscreen quad
        // Placeholder: dark background
        draw.background().color(rgb(0.02, 0.015, 0.01));
        draw.text("Chemreact — WIP")
            .color(WHITE)
            .font_size(24)
            .xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
