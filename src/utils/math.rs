pub fn calculate_rms(window: &[f32]) -> f32 {
    if window.is_empty() {
        return 0.0;
    }
    let sum_squares: f64 = window.iter().map(|&x| (x as f64) * (x as f64)).sum();
    (sum_squares / window.len() as f64).sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rms() {
        assert_eq!(calculate_rms(&[]), 0.0);
        assert_eq!(calculate_rms(&[0.0, 0.0, 0.0]), 0.0);
        assert_eq!(calculate_rms(&[1.0, 1.0, 1.0]), 1.0);
        assert_eq!(calculate_rms(&[-1.0, -1.0, -1.0]), 1.0);
        assert_eq!(calculate_rms(&[1.0, -1.0, 1.0, -1.0]), 1.0);

        // 3^2 + 4^2 = 9 + 16 = 25
        // 25 / 2 = 12.5
        // sqrt(12.5) ≈ 3.5355339
        let rms = calculate_rms(&[3.0, 4.0]);
        assert!((rms - 3.5355339).abs() < 1e-6);
    }
}
