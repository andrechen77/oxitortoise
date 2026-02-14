use std::time::Instant;

use rand::Rng;

pub fn check_finite(num: f64) -> bool {
    num.is_finite()
}

pub fn check_finite_weird(num: f64) -> bool {
    let x = num * 0.0;
    x == x
}

fn main() {
    let n = 50_000_000; // size per test
    let mut rng = rand::thread_rng();

    // Generate vectors
    let mut finite_data = Vec::with_capacity(n);
    let mut nan_inf_data = Vec::with_capacity(n);

    for _ in 0..n {
        // finite numbers between -1e308..1e308
        finite_data.push(rng.gen_range(-1e307..1e307));
        // mostly NaN/∞
        let choice: u8 = rng.gen_range(0..3);
        let val = match choice {
            0 => f64::INFINITY,
            1 => f64::NEG_INFINITY,
            _ => f64::NAN,
        };
        nan_inf_data.push(val);
    }

    // Helper function to run a benchmark
    fn run_benchmark(name: &str, data: &Vec<f64>, func: fn(f64) -> bool) {
        let start = Instant::now();
        let sum: u64 = data.iter().map(|&x| func(x) as u64).sum();
        let duration = start.elapsed();
        println!("{:<35} sum={:<10} took {:?}", name, sum, duration);
    }

    // Run four trials
    run_benchmark("std is_finite on NaN/∞", &nan_inf_data, check_finite);
    run_benchmark("std is_finite on finite", &finite_data, check_finite);
    run_benchmark("weird multiply-zero on NaN/∞", &nan_inf_data, check_finite_weird);
    run_benchmark("weird multiply-zero on finite", &finite_data, check_finite_weird);
}
