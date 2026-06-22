// Real audio capture from default microphone via CPAL.
// Replaces the synthetic test signal with live FFT analysis.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex as FftComplex;
use rustfft::FftPlanner as FftPlan;
use std::collections::VecDeque;
use rustfft::{FftPlanner, num_complex::Complex};
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
    fft_buffer: Vec<rustfft::num_complex::Complex<f32>>,
    fft_scratch: Vec<rustfft::num_complex::Complex<f32>>,
    fft_output: Vec<rustfft::num_complex::Complex<f32>>,
    prev_spectrum: Vec<f32>,
    sample_rate: f32,
    fft_size: usize,
}

impl AudioEngine {
    pub fn new() -> Self {
        let fft_size = 512;
        let ring_len = fft_size * 4;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No audio input device found");

        let config = device
            .default_input_config()
            .expect("No default input config");
        let sample_rate = config.sample_rate().0 as f32;

        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(fft_size * 4)));
        let buf_clone = buffer.clone();

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buf_clone.lock() {
                        for &sample in data {
                            buf.push_back(sample);
                        }
                    }
                },
                |err| log::error!("Audio error: {err}"),
                None,
            )
            .expect("Failed to build input stream");

        stream.play().expect("Failed to start audio stream");
        log::info!("Audio capture started: {sample_rate} Hz, FFT size {fft_size}");

        AudioEngine {
            features: AudioFeatures::default(),
            _stream: stream,
            buffer,
            fft_buffer: vec![FftComplex::new(0.0, 0.0); fft_size],
            fft_scratch: vec![FftComplex::new(0.0, 0.0); fft_size],
            fft_output: vec![FftComplex::new(0.0, 0.0); fft_size],
            prev_spectrum: vec![0.0; fft_size / 2],
            sample_rate,
            fft_size,
        }
    }

    pub fn update(&mut self) {
        let half = self.fft_size / 2;

        // Drain buffer into FFT input
        if let Ok(mut buf) = self.buffer.lock() {
            for i in 0..self.fft_size {
                let sample = buf.pop_front().unwrap_or(0.0);
                self.fft_buffer[i] = FftComplex::new(sample, 0.0);
            }
        }

        // FFT
        let mut planner = FftPlan::new();
        let fft = planner.plan_fft_forward(self.fft_size);
        self.fft_scratch.copy_from_slice(&self.fft_buffer);
        fft.process_with_scratch(&mut self.fft_scratch, &mut self.fft_output);

        // Magnitude spectrum
        let mut spectrum = vec![0.0f32; half];
        let mut max_mag = 0.0f32;
        for i in 0..half {
            let mag = (self.fft_output[i].re.powi(2) + self.fft_output[i].im.powi(2)).sqrt();
            spectrum[i] = mag;
            if mag > max_mag { max_mag = mag; }
        }
        if max_mag > 0.0 {
            for s in &mut spectrum { *s /= max_mag; }
        }

        // Frequency bands (Phosphor-compatible 7-band scheme)
        let band = |lo: f32, hi: f32| -> f32 {
            let lo_bin = (lo / self.sample_rate * self.fft_size as f32) as usize;
            let hi_bin = (hi / self.sample_rate * self.fft_size as f32) as usize;
            let lo_bin = lo_bin.min(half - 1);
            let hi_bin = hi_bin.min(half - 1);
            if hi_bin <= lo_bin { return 0.0; }
            let sum: f32 = spectrum[lo_bin..=hi_bin].iter().sum();
            let count = (hi_bin - lo_bin + 1) as f32;
            sum / count
        };

        // Apply exponential smoothing to all features
        let s = 0.25; // smoothing factor (higher = smoother)
        let sm = |old: &mut f32, new: f32| { *old = *old * (1.0 - s) + new * s; *old };

        sm(&mut self.features.sub_bass, band(20.0, 60.0));
        sm(&mut self.features.bass, band(60.0, 250.0));
        sm(&mut self.features.low_mid, band(250.0, 500.0));
        sm(&mut self.features.mid, band(500.0, 2000.0));
        sm(&mut self.features.upper_mid, band(2000.0, 4000.0));
        sm(&mut self.features.presence, band(4000.0, 6000.0));
        sm(&mut self.features.brilliance, band(6000.0, 20000.0));

        let raw_rms = spectrum.iter().sum::<f32>() / half as f32;
        sm(&mut self.features.rms, raw_rms);

        // Spectral centroid
        let mut weighted = 0.0;
        let mut total = 0.0;
        for i in 0..half {
            weighted += spectrum[i] * i as f32;
            total += spectrum[i];
        }
        let raw_centroid = if total > 0.0 { weighted / total / half as f32 } else { 0.0 };
        sm(&mut self.features.centroid, raw_centroid);

        // Spectral flux (beat/onset detection)
        let mut flux = 0.0;
        for i in 0..half {
            let diff = spectrum[i] - self.prev_spectrum[i];
            flux += diff.max(0.0);
        }
        sm(&mut self.features.flux, flux.min(1.0));
        let raw_onset = if flux > 0.4 { flux } else { 0.0 };
        sm(&mut self.features.onset, raw_onset);
        let raw_beat = if flux > 0.55 { 1.0 } else { 0.0 };
        // Stronger smoothing on beat to prevent rapid re-triggering
        self.features.beat = self.features.beat * 0.7 + raw_beat * 0.3;
        if self.features.beat < 0.1 { self.features.beat = 0.0; }
        self.features.beat_phase = 0.0;
        self.features.bpm = 120.0 / 300.0;
        self.features.flatness = 0.5;
        self.prev_spectrum.copy_from_slice(&spectrum);
    }

    pub fn features(&self) -> &AudioFeatures {
        &self.features
    }
}
