use nannou::prelude::*;
use crate::audio::AudioFeatures;
use crate::sketch::Sketch;

pub struct Chemreact {
    name: String,
    params: [f32; 16],
}

impl Chemreact {
    pub fn new() -> Box<Self> {
        let mut params = [0.5; 16];
        params[0] = 0.45;
        params[1] = 0.35;
        params[2] = 0.6;
        params[3] = 0.5;
        params[4] = 0.4;
        Box::new(Chemreact { name: "Chemreact".into(), params })
    }
}

impl Sketch for Chemreact {
    fn name(&self) -> &str { &self.name }

    fn init(&mut self, _app: &nannou::App, _window: window::Id) {
        // TODO: Create compute pipeline from WGSL, storage textures,
        // ping-pong for Gray-Scott state, bind group
        log::info!("Chemreact init");
    }

    fn update(
        &mut self,
        _app: &nannou::App,
        _window: window::Id,
        _t: &Update,
        _audio: &AudioFeatures,
        _params: &[f32; 16],
    ) {
        // TODO: Update uniforms, dispatch compute, ping-pong swap
    }

    fn view(&self, draw: &Draw, rect: Rect) {
        draw.background().color(rgb(0.02, 0.015, 0.01));
        draw.text("Chemreact — audio capture active")
            .color(WHITE)
            .font_size(14)
            .xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
