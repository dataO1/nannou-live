use nannou::prelude::*;
use nannou::window;
use nannou::wgpu;
use crate::audio::AudioFeatures;

/// A live visual sketch — pluggable scene with WGSL compute + render.
pub trait Sketch: Send + Sync {
    fn name(&self) -> &str;

    /// Called once when sketch is loaded. Set up compute pipelines,
    /// storage textures, bind groups here.
    fn init(&mut self, _app: &nannou::App, _window: window::Id, _device: Option<&wgpu::Device>, _size: (u32, u32)) {}

    /// Called each frame. Update uniforms, dispatch compute passes.
    fn update(
        &mut self,
        _app: &nannou::App,
        _window: window::Id,
        _t: &Update,
        _audio: &AudioFeatures,
        _params: &[f32; 16],
    ) {}

    /// Render the sketch output to the frame.
    /// Render using raw wgpu frame (for compute shader effects).
    /// Default: fall back to draw-based view.
    fn view_frame(&self, _frame: &nannou::Frame) {}

    /// Simple draw-based view (for nannou Draw API effects).
    fn view(&self, _draw: &Draw, _rect: Rect, _audio: &AudioFeatures) {}

    /// Default parameter values (0.0–1.0 range).
    fn params(&self) -> &[f32; 16] { &[0.5; 16] }
}

pub type SketchHandle = usize;

pub struct SketchManager {
    sketches: Vec<Box<dyn Sketch>>,
    params: Vec<[f32; 16]>,
    names: Vec<String>,
}

impl SketchManager {
    pub fn new() -> Self {
        SketchManager {
            sketches: Vec::new(),
            params: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn load_all(&mut self, app: &nannou::App, window: window::Id) -> SketchHandle {
        use crate::sketches;

        // Register sketches here
        self.add(sketches::chemreact::Chemreact::new(), app, window);
        self.add(sketches::folds::Folds::new(), app, window);

        0 // return handle to first sketch
    }

    fn add(&mut self, mut sketch: Box<dyn Sketch>, app: &nannou::App, window: window::Id) {
        let w = app.window(window);
        let device = w.as_ref().map(|w| w.device());
        let size = w.as_ref().map(|w| w.inner_size_pixels()).unwrap_or((1280, 720));
        sketch.init(app, window, device.as_deref(), size);
        self.params.push(*sketch.params());
        self.names.push(sketch.name().to_string());
        self.sketches.push(sketch);
    }

    pub fn switch_to(&mut self, name: &str, active: &mut SketchHandle) {
        if let Some(i) = self.names.iter().position(|n| n == name) {
            *active = i;
            log::info!("Switched to sketch: {name}");
        }
    }

    pub fn switch_to_index(&mut self, index: usize, active: &mut SketchHandle) {
        if index < self.sketches.len() {
            *active = index;
            log::info!("Switched to sketch: {}", self.names[index]);
        }
    }

    pub fn set_param(&mut self, handle: SketchHandle, index: usize, value: f32) {
        if index < 16 {
            self.params[handle][index] = value;
        }
    }

    pub fn update(
        &mut self,
        app: &nannou::App,
        handle: SketchHandle,
        t: &Update,
        audio: &AudioFeatures,
    ) {
        if let Some(sketch) = self.sketches.get_mut(handle) {
            sketch.update(app, window::Id::from(0u64), t, audio, &self.params[handle]);
        }
    }

    pub fn view_frame(&self, handle: SketchHandle, frame: &nannou::Frame) {
        if let Some(sketch) = self.sketches.get(handle) {
            sketch.view_frame(frame);
        }
    }

    pub fn view(&self, handle: SketchHandle, draw: &Draw, rect: Rect, audio: &AudioFeatures) {
        if let Some(sketch) = self.sketches.get(handle) {
            sketch.view(draw, rect, audio);
        }
    }
}
