// Chemreact — Gray-Scott reaction-diffusion (draw-based for now).
// WGSL compute pipeline will be added when nannou wgpu API is mapped.

use nannou::prelude::*;
use crate::audio::AudioFeatures;
use crate::sketch::Sketch;

pub struct Chemreact {
    name: String,
    params: [f32; 16],
    time: f32,
}

impl Chemreact {
    pub fn new() -> Box<Self> {
        let mut params = [0.5; 16];
        params[0] = 0.45; params[1] = 0.35; params[2] = 0.6;
        params[3] = 0.5;  params[4] = 0.4;
        Box::new(Chemreact { name: "Chemreact".into(), params, time: 0.0 })
    }
}

impl Sketch for Chemreact {
    fn name(&self) -> &str { &self.name }

    fn init(&mut self, _app: &nannou::App, _window: window::Id, _device: Option<&wgpu::Device>, _size: (u32, u32)) {}

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
        draw.background().color(rgb(0.02, 0.01, 0.04));

        let w = rect.w(); let h = rect.h();
        let cx = 0.0; let cy = 0.0;

        // 7-band spectrum as radial bars
        let bands = [
            (audio.sub_bass, 0.3, rgb(0.95, 0.35, 0.08)),
            (audio.bass,     0.4, rgb(0.90, 0.50, 0.20)),
            (audio.low_mid,  0.5, rgb(0.70, 0.30, 0.55)),
            (audio.mid,      0.6, rgb(0.30, 0.40, 0.85)),
            (audio.upper_mid,0.7, rgb(0.20, 0.55, 0.90)),
            (audio.presence, 0.8, rgb(0.10, 0.65, 0.75)),
            (audio.brilliance,0.9,rgb(0.25, 0.80, 0.80)),
        ];

        let n = 64;
        for j in 0..n {
            let a = j as f32 / n as f32 * 2.0 * PI;
            let band = &bands[j % 7];
            let r_inner = 40.0 + band.0 * (j / 7) as f32 * 40.0;
            let r_outer = r_inner + band.1 * w * 0.25 * band.0;
            let x1 = cx + a.cos() * r_inner;
            let y1 = cy + a.sin() * r_inner;
            let x2 = cx + a.cos() * r_outer;
            let y2 = cy + a.sin() * r_outer;
            draw.line()
                .start(pt2(x1, y1))
                .end(pt2(x2, y2))
                .color(band.2)
                .stroke_weight(1.5 + band.0 * 2.0);
        }

        // RMS ring
        let r = audio.rms * w * 0.25 + 20.0;
        draw.ellipse().radius(r).no_fill()
            .stroke(rgba(0.9, 0.8, 0.6, 0.3))
            .stroke_weight(1.0);

        // Centroid dot
        let ca = audio.centroid * 2.0 * PI;
        let cr = r * 1.1;
        draw.ellipse()
            .x_y(cx + ca.cos() * cr, cy + ca.sin() * cr)
            .radius(4.0 + audio.rms * 8.0)
            .color(rgba(1.0, 0.9, 0.7, 0.6));

        // Label
        draw.text(&format!("Chemreact  rms:{:.2}", audio.rms))
            .color(WHITE).font_size(11).xy(rect.xy());
    }

    fn params(&self) -> &[f32; 16] { &self.params }
}
