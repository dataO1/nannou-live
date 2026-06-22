// Minimal audio capture — raw RMS only, no FFT.
// Guaranteed stable: no spectral analysis, no beat detection.
// Just loudness and basic energy.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

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
    _stream: cpal::Stream,
    buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: f32,
}

impl AudioEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No audio input device");

        let config = device
            .default_input_config()
            .expect("No default input config");
        let sample_rate = config.sample_rate().0 as f32;

        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
        let buf = buffer.clone();

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut b) = buf.lock() {
                        for &s in data { b.push_back(s); }
                    }
                },
                |err| log::error!("Audio: {err}"),
                None,
            )
            .expect("Failed to build input stream");

        stream.play().expect("Failed to start");
        log::info!("Audio capture: {sample_rate} Hz");

        AudioEngine {
            features: AudioFeatures::default(),
            _stream: stream,
            buffer,
            sample_rate,
        }
    }

    pub fn update(&mut self) {
        // Read ~5ms worth of samples
        let n = (self.sample_rate * 0.005) as usize;
        let mut sum = 0.0f32;
        let mut count = 0usize;

        if let Ok(mut buf) = self.buffer.lock() {
            for _ in 0..n {
                if let Some(s) = buf.pop_front() {
                    sum += s * s;
                    count += 1;
                }
            }
        }

        let rms = if count > 0 { (sum / count as f32).sqrt() } else { 0.0 };

        // Smooth RMS
        let s = 0.12;
        self.features.rms = self.features.rms * (1.0 - s) + rms * s;

        // Derive everything from RMS for stable visual
        let r = self.features.rms;
        self.features.sub_bass = r * 0.7;
        self.features.bass = r * 0.8;
        self.features.low_mid = r * 0.6;
        self.features.mid = r * 0.5;
        self.features.upper_mid = r * 0.4;
        self.features.presence = r * 0.3;
        self.features.brilliance = r * 0.2;
        self.features.centroid = 0.3 + r * 0.2;
        self.features.beat = 0.0;
        self.features.onset = 0.0;
        self.features.flux = 0.0;
        self.features.flatness = 0.5;
        self.features.beat_phase = 0.0;
        self.features.bpm = 120.0 / 300.0;
    }

    pub fn features(&self) -> &AudioFeatures {
        &self.features
    }
}
