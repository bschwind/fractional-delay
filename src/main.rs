fn main() {
    let mut tone_generator = ToneGenerator::new(600, 48_000);
    let mut tone_generator_2 = ToneGenerator::new(1200, 48_000);
    let mut tone_generator_3 = ToneGenerator::new(2562, 48_000);

    let delay = 7.816;

    let mut fractional_delay = FractionalDelay::new(delay);

    for i in 0..=100 {
        let next_sample = tone_generator.next();
        let next_2 = tone_generator_2.next();
        let next_3 = tone_generator_3.next();

        let next_sample = next_sample + next_2 + next_3;

        let delayed_sample = fractional_delay.process(next_sample);
        println!("{i}, {next_sample}, {delayed_sample}");
    }
}

struct ToneGenerator {
    delta: f64,
    current: f64,
}

impl ToneGenerator {
    pub fn new(frequency_hz: u32, sample_rate: u32) -> Self {
        let delta = (frequency_hz as f64 * core::f64::consts::TAU) / sample_rate as f64;

        Self {
            delta,
            current: 0.0,
        }
    }

    pub fn next(&mut self) -> f64 {
        let val = self.current.sin();
        self.current += self.delta;

        if self.current > core::f64::consts::TAU {
            self.current -= core::f64::consts::TAU;
        }

        val
    }
}

struct FractionalDelay {
    fir_filter: FirFilter,
}

impl FractionalDelay {
    fn new(delay: f64) -> Self {
        // Reference implementation:
        // http://www.labbookpages.co.uk/audio/beamforming/fractionalDelay.html
        use std::f64::consts::PI;

        let integer_delay = delay.floor() as usize;
        let fractional_delay = delay - delay.floor();

        let filter_length = integer_delay * 2 + 1;
        let center_tap = filter_length / 2;

        let fir_filter_weights = (0..filter_length)
            .map(|t| {
                let x = t as f64 - fractional_delay;

                let sinc = if (x - center_tap as f64).abs() <= 0.00000001 {
                    1.0
                } else {
                    f64::sin(PI * (x - center_tap as f64)) / (PI * (x - center_tap as f64))
                };

                let window = 0.54 - 0.46 * f64::cos(2.0 * PI * (x + 0.5) / filter_length as f64);

                window * sinc
            })
            .collect();

        let fir_filter = FirFilter::new(fir_filter_weights);

        Self { fir_filter }
    }

    fn process(&mut self, input: f64) -> f64 {
        self.fir_filter.process(input)
    }
}

struct FirFilter {
    filter_weights: Vec<f64>,
    buffer: Vec<f64>,
    buf_index: usize,
    output: f64,
}

impl FirFilter {
    fn new(filter_weights: Vec<f64>) -> Self {
        let length = filter_weights.len();

        Self {
            filter_weights,
            buffer: vec![0.0; length],
            buf_index: 0,
            output: 0.0,
        }
    }

    fn process(&mut self, input: f64) -> f64 {
        // Reference implementation:
        // "FIR Filter Design and Software Implementation - Phil's Lab #17"
        // https://www.youtube.com/watch?v=uNNNj9AZisM&t=402s
        self.buffer[self.buf_index] = input;

        self.buf_index += 1;

        if self.buf_index >= self.buffer.len() {
            self.buf_index = 0;
        }

        self.output = 0.0;
        let mut sum_index = self.buf_index;

        for n in 0..self.buffer.len() {
            if sum_index > 0 {
                sum_index -= 1;
            } else {
                sum_index = self.buffer.len() - 1;
            }

            self.output += self.filter_weights[n] * self.buffer[sum_index];
        }

        self.output
    }
}
