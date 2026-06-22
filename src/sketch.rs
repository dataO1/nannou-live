use nannou::prelude::*;
use crate::audio::AudioFeatures;

/// A live visual sketch — like a Phosphor effect or Xtal sketch.
pub trait Sketch: Send + Sync {
    fn name(&self) -> &str;
    fn update(&mut self, _update: Update, _audio: &AudioFeatures, _params: &[f32; 16]) {}
    fn view(&self, draw: &Draw, rect: Rect, audio: &AudioFeatures, params: &[f32; 16]);
    fn params(&self) -> &[f32; 16] { &[0.5; 16] }
}

pub type SketchHandle = usize;

pub struct SketchManager {
    sketches: Vec<Box<dyn Sketch>>,
    params: Vec<[f32; 16]>,
}

impl SketchManager {
    pub fn new() -> Self {
        SketchManager {
            sketches: Vec::new(),
            params: Vec::new(),
        }
    }

    pub fn load_all(&mut self) -> SketchHandle {
        use crate::sketches;
        let mut idx = self.sketches.len();

        // Register sketches — add new ones here
        self.sketches.push(Box::new(sketches::chemreact::Chemreact::new()));
        self.params.push(*self.sketches.last().unwrap().params());

        self.sketches.push(Box::new(sketches::folds::Folds::new()));
        self.params.push(*self.sketches.last().unwrap().params());

        // Return handle to first sketch
        idx = 0;
        idx
    }

    pub fn update(&mut self, handle: SketchHandle, update: Update, audio: &AudioFeatures) {
        if let Some(sketch) = self.sketches.get_mut(handle) {
            sketch.update(update, audio, &self.params[handle]);
        }
    }

    pub fn view(&self, handle: SketchHandle, draw: &Draw, rect: Rect) {
        if let Some(sketch) = self.sketches.get(handle) {
            sketch.view(draw, rect, &AudioFeatures::default(), &self.params[handle]);
        }
    }
}
