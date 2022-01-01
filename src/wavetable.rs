use basic_dsp::*;
use num::complex::*;


struct WaveTable {
    len: usize,
    time_domain: Vec<f32>,
    freq_domain: Vec<Complex32> 
}

impl WaveTable {
    pub fn new(len: usize) -> Self {
        WaveTable {
            len,
            time_domain: vec![0.0; len],
            freq_domain: vec![Complex32::default(); len as usize];}
    }

    pub fn make_time_domain(&mut self) -> Self{
        time_domain.fft(time_domain);
        self
    }

    pub fn make_freq_domain(&mut self) -> Self {
        freq_domain.ifft(time_domain);
        self
    }
}

/*
impl WaveTable {
    fn run_fft(&mut self) {
        // make a planner
        let mut real_planner = RealFftPlanner::<f64>::new();

        // create a FFT
        let r2c = real_planner.plan_fft_forward(self.data.size());
        // make input and output vectors
        let mut indata = r2c.make_input_vec();
        let mut spectrum = r2c.make_output_vec();

        // Forward transform the input data
        r2c.process(&mut indata, &mut spectrum).unwrap();

    }
    fn run_ifft(&mut self) {
        // create an iFFT and an output vector
        let c2r = real_planner.plan_fft_inverse(length);
        let mut outdata = c2r.make_output_vec();

        c2r.process(&mut spectrum, &mut outdata).unwrap();        
    }

}
 */
