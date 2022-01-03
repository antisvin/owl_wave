use std::cmp::min;

use wavetable::{WavData, Wavetable, WtReader};

pub struct Grid {
    rows: usize,
    cols: usize,
    samples: usize,
    wavetable: Wavetable,
}

impl Grid {
    pub fn new(rows: usize, cols: usize, samples: usize) -> Self {
        let mut grid = Grid {
            rows,
            cols,
            samples,
            wavetable: Wavetable::new(rows * cols, 1, samples),
        };
        for i in 0..rows {
            for j in 0..cols {
                let wave = grid.wavetable.get_wave_mut(i * rows + j);
                Wavetable::add_sine_wave(wave, 1.0, 1.0, 0.0);
            }
        }
        grid
    }
    /*
    pub fn get_rows(&self) -> usize{
        self.rows
    }
    pub fn get_cols(&self)-> usize {
        self.cols
    }
     */
    pub fn get_waves(&self) -> usize {
        self.rows * self.cols
    }
    /*
    pub fn get_wave(&self, row: usize, col: usize) -> &Vec<f64>{
        self.wavetable.get_wave(row * self.cols + col)
    }
    pub fn get_wave_mut(&mut self, row: usize, col: usize) -> &mut Vec<f64>{
        self.wavetable.get_wave_mut(row * self.cols + col)
    }
    */
    pub fn get_wave_by_id(&self, i: usize) -> &Vec<f64> {
        self.wavetable.get_wave(i)
    }
    /*
    pub fn get_wave_by_idmut(&mut self, i: usize) -> &mut Vec<f64>{
        self.wavetable.get_wave_mut(i)
    }
    */
    pub fn get_samples(&self) -> usize {
        self.samples
    }
    pub fn load_waves(&mut self, wav_data: &WavData) -> Result<usize, ()> {
        let result = WtReader::create_wavetable(wav_data, Some(self.samples));
        if let Ok(wt_ref) = result {
            let num_tables = min(self.get_waves(), wt_ref.num_tables);
            for i in 0..num_tables {
                let wave = self.wavetable.get_wave_mut(i);
                let new_wave = wt_ref.get_wave(i);
                for (j, &el) in new_wave.iter().enumerate() {
                    wave[j] = el;
                }

                //wave.clear();
                // = wt_ref.get_wave(i);
            }
            for i in num_tables..self.get_waves() {
                self.wavetable.get_wave_mut(i).clear()
            }
            Ok(num_tables)
        } else {
            Ok(0)
        }
    }
}
