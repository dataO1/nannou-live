use nannou::prelude::*;
use crate::audio::AudioFeatures;
use crate::sketch::Sketch;

pub struct Folds {
    name: String,
    params: [f32; 16],
    time: f32,
}

impl Folds {
    pub fn new() -> Box<Self> {
        let mut params = [0.5; 16];
        params[0] = 0.4;
        params[1] = 0.35;
        params[2] = 0.55;
        params[3] = 0.45;
        params[4] = 0.4;
        Box::new(Folds { name: "Folds".into(), params, time: 0.0 })
    }
}

impl Sketch for Folds {
    fn name(&self) -> &str { &self.name }

    fn init(&mut self, _app: &nannou::App, _window: window::Id) {
        log::info!("Folds init");
    }

    fn update(
        &mut self,
        _app: &nannou::App,
        _window: window::Id,
        t: &Update,
        _audio: &AudioFeatures,
        _params: &[f32; 16],
    ) {
        self.time += t.since_last.as_secs_f32();
    }

    fn view(&self, draw: &Draw, rect: Rect, audio: &AudioFeatures) {
        draw.background().color(rgb(0.02, 0.015, 0.01));

        let centre = rect.xy();
        let n = 36;
        for i in 0..n {
            let angle = i as f32 / n as f32 * 2.0 * PI + self.time * (audio.centroid + 0.2);
            let r = 100.0 + audio.rms * 200.0 + (angle * 0.5).sin() * 50.0 * audio.bass;
            let x = centre.x + angle.cos() * r;
            let y = centre.y + angle.sin() * r;
            draw.ellipse()
                .x_y(x, y)
                .radius(3.0 + audio.rms * 8.0)
                .color(rgba(0.9, 0.6, 0.2, 0.5));
        }

        draw.text(&format!("Folds — rms:{:.2}", audio.rms))
            .color(WHITE)
            .font_size(12)
            .xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
