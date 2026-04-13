pub fn calculate_rms(window: &[f32]) -> f32 {
    if window.is_empty() {
        return 0.0;
    }
    let sum_squares: f32 = window.iter().map(|&x| x * x).sum();
    (sum_squares / window.len() as f32).sqrt()
}
