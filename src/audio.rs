/// Simplified audio features for visual reactivity.
/// Currently uses placeholder values — wire up nannou_audio/cpal
/// when the rest of the pipeline is working.

#[derive(Debug, Clone, Default)]
pub struct AudioFeatures {
    pub rms: f32,
    pub centroid: f32,
    pub sub_bass: f32,
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
    time: f32,
}

impl AudioEngine {
    pub fn new() -> Self {
        AudioEngine {
            features: AudioFeatures::default(),
            time: 0.0,
        }
    }

    pub fn update(&mut self, _dt: f32) {
        self.time += _dt;

        // Placeholder: generate synthetic test signal
        let pulse = (self.time * 2.0).sin().max(0.0);
        let low = (self.time * 0.5).sin().abs();
        let high = (self.time * 8.0).sin().abs() * 0.3;

        self.features.rms = pulse * 0.3 + 0.1;
        self.features.sub_bass = low * 0.8;
        self.features.bass = low * 0.7;
        self.features.low_mid = low * 0.4 + pulse * 0.2;
        self.features.mid = pulse * 0.5 + high * 0.2;
        self.features.upper_mid = high * 0.4 + pulse * 0.3;
        self.features.presence = high * 0.6;
        self.features.brilliance = high * 0.3;
        self.features.centroid = 0.3 + high * 0.5;
        self.features.flux = (self.time * 4.0).sin().abs() * 0.5;
        self.features.beat = if (self.time % 0.5) < 0.05 { 1.0 } else { 0.0 };
        self.features.onset = self.features.beat * 0.8;
        self.features.beat_phase = (self.time % 0.5) / 0.5;
        self.features.bpm = 120.0 / 300.0;
        self.features.flatness = 0.4;
    }

    pub fn features(&self) -> &AudioFeatures {
        &self.features
    }
}
