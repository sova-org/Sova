pub const DEFAULT_C_TUNING : f64 = 261.6255653006;

pub struct Scale {
    pub tonic: f64,
    pub divisions: usize,
    pub deviation: f64,
    pub octave: f64,
    pub intervals: Vec<usize>
}

impl Default for Scale {
    fn default() -> Self {
        Self { 
            tonic: DEFAULT_C_TUNING, 
            divisions: 12, 
            deviation: 0.0, 
            octave: 2.0, 
            intervals: vec![1;12] 
        }
    }
}

impl Scale {

    pub fn note(&self, index: i64) -> f64 {
        todo!()
    }

}