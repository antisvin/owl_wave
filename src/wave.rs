use wavetable::Harmonic;

// Magnitude + phase
pub type Polar = (f64, f64);

struct Wave {
    time_domain: Vec<f64>,
    freq_domain: Vec<Harmonic>,
    polar: Vec<Polar>
}

impl Wave {
    pub fn new(time_domain: Vec<f64>, freq_domain: Vec<Harmonic>) -> Self {
        let polar = freq_domain
            .iter()
            .map(|h| {h.to_polar()})
            .collect();
        Wave {time_domain, freq_domain, polar}
    }
}
