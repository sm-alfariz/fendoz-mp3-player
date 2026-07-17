use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

const EQ_FREQUENCIES: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0,
    1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

#[derive(Debug, Clone, Copy)]
pub struct BiquadFilter {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            a0: 1.0, a1: 0.0, a2: 0.0,
            b0: 1.0, b1: 0.0, b2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    /// Peaking EQ filter
    pub fn peaking_eq(gain_db: f32, frequency: f32, sample_rate: f32, q: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * frequency / sample_rate;
        let alpha = (w0 / (2.0 * q)).sin() * ((a + 1.0 / a) * (1.0 / (q * 2.0) - 1.0) + 2.0).sqrt();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        Self {
            a0, a1, a2, b0, b1, b2,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = (self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2) / self.a0;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

pub struct Equalizer {
    filters: [BiquadFilter; 10],
    gains: [f32; 10],
    sample_rate: f32,
}

impl Equalizer {
    pub fn new(sample_rate: f32) -> Self {
        let filters = std::array::from_fn(|i| {
            BiquadFilter::peaking_eq(0.0, EQ_FREQUENCIES[i], sample_rate, 1.0)
        });

        Self {
            filters,
            gains: [0.0; 10],
            sample_rate,
        }
    }

    pub fn set_gain(&mut self, band: usize, gain_db: f32) {
        if band < 10 {
            self.gains[band] = gain_db;
            self.filters[band] = BiquadFilter::peaking_eq(
                gain_db,
                EQ_FREQUENCIES[band],
                self.sample_rate,
                1.0,
            );
        }
    }

    pub fn get_gains(&self) -> [f32; 10] {
        self.gains
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let mut output = sample;
        for filter in self.filters.iter_mut() {
            output = filter.process(output);
        }
        output
    }

    pub fn reset(&mut self) {
        for filter in self.filters.iter_mut() {
            filter.reset();
        }
        self.gains = [0.0; 10];
    }
}

pub struct FFTProcessor {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    buffer_size: usize,
}

impl FFTProcessor {
    pub fn new(buffer_size: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(buffer_size);
        Self { fft, buffer_size }
    }

    pub fn process(&self, samples: &[f32]) -> Vec<f32> {
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        // Pad or truncate to buffer_size
        buffer.resize(self.buffer_size, Complex::new(0.0, 0.0));

        self.fft.process(&mut buffer);

        // Calculate magnitude spectrum
        let half_size = self.buffer_size / 2;
        buffer[..half_size]
            .iter()
            .map(|c| (c.norm() / self.buffer_size as f32))
            .collect()
    }
}

pub fn get_frequency_bands(fft_data: &[f32], sample_rate: f32) -> Vec<f32> {
    let bin_size = sample_rate / (fft_data.len() as f32 * 2.0);
    let mut bands = Vec::with_capacity(10);

    for &freq in EQ_FREQUENCIES.iter() {
        let bin = (freq / bin_size) as usize;
        let value = if bin < fft_data.len() {
            fft_data[bin]
        } else {
            0.0
        };
        bands.push(value);
    }

    bands
}
