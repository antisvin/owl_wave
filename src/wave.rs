use rustfft::num_complex::Complex;
use rustfft::{num_traits::Zero, FftPlanner};
use wavetable::{Float, Harmonic};
//use crate::effects::Effect;

// Magnitude + phase
pub type Polar = (f64, f64);

pub struct WaveState {
    time_domain: Vec<Float>,
    freq_domain: Vec<Harmonic>,
    polar: Vec<Polar>,
}

impl WaveState {
    /*
    pub fn new(size: usize) -> Self {
        WaveState {
            time_domain: Vec::<Float>::with_capacity(size),
            freq_domain: Vec::<Harmonic>::with_capacity(size),
            polar: Vec::<Polar>::with_capacity(size)}
    }
    */
}
/*
pub struct Wave {
    pre_fx: WaveState,
//    fx: Vec<Effect>,
    post_fx: WaveState
}

impl Wave {
    pub fn new(size: usize) -> Self {
        Wave { pre_fx: WaveState::new(size), post_fx: WaveState::new(size) }
//        Wave { pre_fx: WaveState::new(size), fx: vec!(Effect, 0), post_fx: WaveState::new(size) }
    }
}

*/

pub struct TimeDomain;
pub struct FreqDomain;

/*
pub enum DomainState {
    Time(TimeDomain),
    Freq(FreqDomain)
}
*/

pub trait DomainData<Domain, Data> {
    fn get_domain_data(&self) -> &Vec<Data>;
    fn get_domain_data_mut(&mut self) -> &mut Vec<Data>;
}

impl DomainData<TimeDomain, f64> for WaveState {
    fn get_domain_data(&self) -> &Vec<f64> {
        &self.time_domain
    }
    fn get_domain_data_mut(&mut self) -> &mut Vec<f64> {
        &mut self.time_domain
    }
}

impl DomainData<FreqDomain, Harmonic> for WaveState {
    fn get_domain_data(&self) -> &Vec<Harmonic> {
        &self.freq_domain
    }
    fn get_domain_data_mut(&mut self) -> &mut Vec<Harmonic> {
        &mut self.freq_domain
    }
}

pub trait DomainConversion {
    fn convert(wave: &mut WaveState);
}

impl DomainConversion for FreqDomain {
    fn convert(wave: &mut WaveState) {
        let num_samples = wave.time_domain.len();
        let fft_len = num_samples;

        // Prepare FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_len);

        // Copy wave to buffer.
        for (i, sample) in wave.time_domain.iter().take(num_samples).enumerate() {
            wave.freq_domain[i].re = *sample;
            wave.freq_domain[i].im = 0.0;
        }

        // Process buffer
        fft.process(wave.freq_domain.as_mut_slice());
        for (i, sample) in wave.freq_domain.iter().take(num_samples).enumerate() {
            wave.polar[i] = sample.to_polar();
        }
    }
}

impl DomainConversion for TimeDomain {
    fn convert(wave: &mut WaveState) {
        let num_samples = wave.freq_domain.len();
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(num_samples);
        let mut buffer: Vec<Complex<Float>> = vec![Complex::zero(); num_samples];
        fft.process(&mut buffer);
        for (i, item) in buffer.iter().enumerate().take(num_samples) {
            wave.time_domain[i] = item.re;
        }
    }
}
