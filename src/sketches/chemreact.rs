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
        log::info!("Chemreact init");
    }

    fn update(
        &mut self,
        _app: &nannou::App,
        _window: window::Id,
        _t: &Update,
        _audio: &AudioFeatures,
        _params: &[f32; 16],
    ) {}

    fn view(&self, draw: &Draw, rect: Rect, audio: &AudioFeatures) {
        draw.background().color(rgb(0.02, 0.01, 0.04));

        let w = rect.w();
        let h = rect.h();
        let bands = [
            (audio.sub_bass, rgb(0.9, 0.3, 0.1)),
            (audio.bass,     rgb(0.9, 0.5, 0.2)),
            (audio.low_mid,  rgb(0.7, 0.3, 0.6)),
            (audio.mid,      rgb(0.3, 0.4, 0.9)),
            (audio.upper_mid,rgb(0.2, 0.6, 0.9)),
            (audio.presence, rgb(0.1, 0.7, 0.8)),
            (audio.brilliance,rgb(0.3, 0.8, 0.7)),
        ];

        let bar_w = w / bands.len() as f32;
        for (i, &(amp, col)) in bands.iter().enumerate() {
            let bar_h = amp * h * 0.8;
            let x = i as f32 * bar_w - w / 2.0 + bar_w / 2.0;
            let y = bar_h / 2.0 - h / 2.0;
            draw.rect()
                .x_y(x, y)
                .w_h(bar_w * 0.8, bar_h.max(2.0))
                .color(col);
        }

        // RMS ring in centre
        let r = audio.rms * w * 0.3;
        draw.ellipse()
            .radius(r)
            .no_fill()
            .stroke(rgb(0.9, 0.8, 0.6))
            .stroke_weight(2.0);

        // Beat flash
        if audio.beat > 0.5 {
            draw.rect()
                .wh(rect.wh())
                .color(rgba(1.0, 0.9, 0.7, 0.05));
        }

        // FPS label
        draw.text(&format!("Chemreact — rms:{:.2} beat:{}", audio.rms, audio.beat as u8))
            .color(WHITE)
            .font_size(12)
            .xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
