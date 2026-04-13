use audiotools::utils::detection::AutoStartDetection;
use std::time::Instant;

fn main() {
    let detector = AutoStartDetection {
        threshold: 0.1,
        window_size: 512,
        min_duration: 0.01,
    };

    let sample_rate = 44100.0;
    let num_samples = 1_000_000;
    let mut samples = vec![0.0f32; num_samples];

    // Put a trigger at the very end to maximize the O(N*W) work
    // We want it to NOT trigger until near the end.
    for i in (num_samples - 1000)..num_samples {
        samples[i] = 0.5 * (i % 2) as f32; // Alternating to ensure some RMS value
    }

    println!("Starting benchmark with {} samples, window size {}...", num_samples, detector.window_size);
    let start = Instant::now();
    let result = detector.detect_start_time(&samples, sample_rate);
    let duration = start.elapsed();

    println!("Detection result: {:?}", result);
    println!("Time elapsed: {:?}", duration);
}
