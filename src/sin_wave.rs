
struct SinWave {
    hz: f64,
    t: u64,
}

impl SinWave {
    fn new(hz: f64) -> Self {
        SinWave { hz, t: 0 }
    }
}

impl Iterator for SinWave {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let amplitude = (2.0 * PI * self.hz * (self.t as f64) / 44000.0).sin();
        self.t += 1;
        Some(amplitude)
    }
}
