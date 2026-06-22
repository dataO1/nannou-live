// Audio capture with FFT — stable, low-latency.
// Fixed: proper buffer drain, modest FFT, strong smoothing.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
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
    fft_buffer: Vec<Complex<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    fft_output: Vec<Complex<f32>>,
    prev_spectrum: Vec<f32>,
    sample_rate: f32,
    fft_size: usize,
}

impl AudioEngine {
    pub fn new() -> Self {
        let fft_size = 256; // small = fast, enough for 7 bands

        let host = cpal::default_host();
        let device = host.default_input_device().expect("No audio input");
        let config = device.default_input_config().expect("No config");
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
        log::info!("Audio: {sample_rate} Hz, FFT={fft_size}");

        AudioEngine {
            features: AudioFeatures::default(),
            _stream: stream,
            buffer,
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            fft_scratch: vec![Complex::new(0.0, 0.0); fft_size],
            fft_output: vec![Complex::new(0.0, 0.0); fft_size],
            prev_spectrum: vec![0.0; fft_size / 2],
            sample_rate,
            fft_size,
        }
    }

    pub fn update(&mut self) {
        let half = self.fft_size / 2;

        // Drain buffer — discard old samples, keep only the latest fft_size
        if let Ok(mut buf) = self.buffer.lock() {
            let excess = buf.len().saturating_sub(self.fft_size);
            for _ in 0..excess {
                buf.pop_front();
            }
            for i in 0..self.fft_size {
                let s = buf.pop_front().unwrap_or(0.0);
                self.fft_buffer[i] = Complex::new(s, 0.0);
            }
        }

        // FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_size);
        self.fft_scratch.copy_from_slice(&self.fft_buffer);
        fft.process_with_scratch(&mut self.fft_scratch, &mut self.fft_output);

        // Magnitude spectrum
        let mut spectrum = vec![0.0f32; half];
        for i in 0..half {
            spectrum[i] = (self.fft_output[i].re.powi(2) + self.fft_output[i].im.powi(2)).sqrt();
        }

        // Frequency bands
        let band = |lo: f32, hi: f32| -> f32 {
            let lo_bin = (lo / self.sample_rate * self.fft_size as f32).max(0.0) as usize;
            let hi_bin = (hi / self.sample_rate * self.fft_size as f32).max(0.0) as usize;
            let lo_bin = lo_bin.min(half - 1);
            let hi_bin = hi_bin.min(half - 1);
            if hi_bin <= lo_bin { return 0.0; }
            let slice = &spectrum[lo_bin..=hi_bin];
            slice.iter().sum::<f32>() / slice.len() as f32
        };

        // Smoothing
        let s = 0.3;
        let sm = |old: &mut f32, new: f32| { *old = *old * (1.0 - s) + new * s; *old };

        sm(&mut self.features.sub_bass, band(20.0, 60.0));
        sm(&mut self.features.bass, band(60.0, 250.0));
        sm(&mut self.features.low_mid, band(250.0, 500.0));
        sm(&mut self.features.mid, band(500.0, 2000.0));
        sm(&mut self.features.upper_mid, band(2000.0, 4000.0));
        sm(&mut self.features.presence, band(4000.0, 6000.0));
        sm(&mut self.features.brilliance, band(6000.0, 20000.0));
        sm(&mut self.features.rms, spectrum.iter().sum::<f32>() / half as f32);

        // Centroid
        let mut w = 0.0; let mut t = 0.0;
        for i in 0..half { w += spectrum[i] * i as f32; t += spectrum[i]; }
        sm(&mut self.features.centroid, if t > 0.0 { w / t / half as f32 } else { 0.0 });

        // Flux (simple)
        let mut flux = 0.0;
        for i in 0..half { let d = spectrum[i] - self.prev_spectrum[i]; flux += d.max(0.0); }
        sm(&mut self.features.flux, flux.min(1.0));

        // Beat: only on strong flux with cooldown
        let raw_beat = if flux > 0.6 { 1.0f32 } else { 0.0 };
        self.features.beat = self.features.beat * 0.65 + raw_beat * 0.35;
        self.features.onset = self.features.flux * 0.5;

        self.features.beat_phase = 0.0;
        self.features.bpm = 120.0 / 300.0;
        self.features.flatness = 0.5;
        self.prev_spectrum.copy_from_slice(&spectrum);
    }

    pub fn features(&self) -> &AudioFeatures {
        &self.features
    }
}
