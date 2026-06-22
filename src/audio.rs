use nannou::prelude::*;

/// Audio features extracted each frame, matching Phosphor's uniform set.
#[derive(Debug, Clone, Default)]
pub struct AudioFeatures {
    pub rms: f32,
    pub centroid: f32,
    pub bass: f32,
    pub low_mid: f32,
    pub mid: f32,
    pub upper_mid: f32,
    pub presence: f32,
    pub brilliance: f32,
    pub beat: f32,
    pub onset: f32,
    pub flux: f32,
    pub flatness: f32,
    pub beat_phase: f32,
    pub bpm: f32,
}

pub struct AudioEngine {
    pub features: AudioFeatures,
    // TODO: FFT buffer, beat detector, etc.
}

impl AudioEngine {
    pub fn new() -> Self {
        AudioEngine {
            features: AudioFeatures::default(),
        }
    }

    pub fn update(&mut self) {
        // TODO: read from audio input, compute FFT, detect beats
        // For now, placeholder — will wire up nannou_audio in next step
    }

    pub fn features(&self) -> &AudioFeatures {
        &self.features
    }
}
