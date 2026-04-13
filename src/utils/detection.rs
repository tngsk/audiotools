use hound::WavReader;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AutoStartDetection {
    pub threshold: f32,     // 振幅のスレッショルド値
    pub window_size: usize, // 検出用の移動平均ウィンドウサイズ
    pub min_duration: f32,  // 最小持続時間（秒）
}

impl Default for AutoStartDetection {
    fn default() -> Self {
        Self {
            threshold: 0.01,    // デフォルトのスレッショルド値（-40dB相当）
            window_size: 512,   // デフォルトのウィンドウサイズ
            min_duration: 0.01, // デフォルトの最小持続時間（10ms）
        }
    }
}

impl AutoStartDetection {
    // ゼロクロッシングを検出する関数
    fn is_zero_crossing(a: f32, b: f32) -> bool {
        (a < 0.0 && b >= 0.0) || (a >= 0.0 && b < 0.0)
    }

    pub fn detect_start_time(&self, samples: &[f32], sample_rate: f32) -> Option<f32> {
        if samples.len() < self.window_size {
            return None;
        }

        let min_samples = (self.min_duration * sample_rate) as usize;
        let mut triggered = false;
        let mut potential_start = 0;

        // Calculate initial window sum of squares
        let mut sum_squares: f64 = samples[..self.window_size]
            .iter()
            .map(|&x| (x as f64) * (x as f64))
            .sum();

        let threshold_sq = (self.threshold as f64) * (self.threshold as f64);
        let window_size_f64 = self.window_size as f64;

        for i in 0..samples.len().saturating_sub(self.window_size) {
            if !triggered && sum_squares / window_size_f64 > threshold_sq {
                triggered = true;
                potential_start = i;
            } else if triggered {
                if i - potential_start >= min_samples {
                    for j in potential_start..i {
                        if j + 1 < samples.len()
                            && Self::is_zero_crossing(samples[j], samples[j + 1])
                        {
                            return Some(j as f32 / sample_rate);
                        }
                    }
                    return Some(potential_start as f32 / sample_rate);
                }
            }

            // Update sliding window
            if i + self.window_size < samples.len() {
                let out_sample = samples[i] as f64;
                let in_sample = samples[i + self.window_size] as f64;
                sum_squares -= out_sample * out_sample;
                sum_squares += in_sample * in_sample;
                // Avoid cumulative floating point errors
                if sum_squares < 0.0 {
                    sum_squares = 0.0;
                }
            }
        }

        None
    }
}

pub fn create_auto_start_config(
    enabled: bool,
    threshold: f32,
    window_size: usize,
    min_duration: f32,
) -> Option<AutoStartDetection> {
    if enabled {
        Some(AutoStartDetection {
            threshold,
            window_size,
            min_duration,
        })
    } else {
        None
    }
}

pub fn detect_peak_level(input: &PathBuf) -> Result<f32, Box<dyn std::error::Error>> {
    let mut max_peak = 0.0f32;

    if let Ok(reader) = WavReader::open(input) {
        // WAVファイルの場合
        let spec = reader.spec();
        match spec.sample_format {
            hound::SampleFormat::Float => {
                for sample in reader.into_samples::<f32>() {
                    if let Ok(sample) = sample {
                        max_peak = max_peak.max(sample.abs());
                    }
                }
            }
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                let max_value = (1 << (bits - 1)) as f32;

                for sample in reader.into_samples::<i32>() {
                    if let Ok(sample) = sample {
                        let normalized = sample as f32 / max_value;
                        max_peak = max_peak.max(normalized.abs());
                    }
                }
            }
        }
    } else {
        // WAV以外のフォーマットの場合（mp3, flac等）
        let file = File::open(input)?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader)?;

        // i16サンプルをf32に正規化(-1.0から1.0の範囲に)
        for sample in decoder {
            let normalized = sample as f32 / 32768.0; // i16の最大値で正規化
            max_peak = max_peak.max(normalized.abs());
        }
    }

    // ピーク値をdBFSに変換
    let peak_dbfs = 20.0 * max_peak.max(1e-20).log10();
    Ok(peak_dbfs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_start_time_triggers() {
        let detector = AutoStartDetection {
            threshold: 0.1,
            window_size: 10,
            min_duration: 0.01,
        };
        let sample_rate = 1000.0; // 1 sample = 1ms
        let mut samples = vec![0.0; 100];
        // Trigger at index 50
        for i in 50..60 {
            samples[i] = 0.5;
        }
        // Zero crossing at 51 (0.5 to -0.5 is not possible here but let's make it cross)
        samples[51] = -0.5;

        let result = detector.detect_start_time(&samples, sample_rate);
        assert!(result.is_some());
        // Since min_duration is 10ms (10 samples), and it triggers at 50,
        // it checks for zero crossings between 50 and 60.
        // There is a zero crossing between 50 and 51.
        assert_eq!(result.unwrap(), 50.0 / sample_rate);
    }

    #[test]
    fn test_detect_start_time_no_trigger() {
        let detector = AutoStartDetection {
            threshold: 0.1,
            window_size: 10,
            min_duration: 0.01,
        };
        let sample_rate = 1000.0;
        let samples = vec![0.05; 100];

        let result = detector.detect_start_time(&samples, sample_rate);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_start_time_trigger_at_end() {
        let detector = AutoStartDetection {
            threshold: 0.1,
            window_size: 10,
            min_duration: 0.01,
        };
        let sample_rate = 1000.0;
        let mut samples = vec![0.0; 100];
        for i in 90..100 {
            samples[i] = 0.5;
        }

        let result = detector.detect_start_time(&samples, sample_rate);
        // It might not trigger because the loop goes to samples.len() - window_size
        // 100 - 10 = 90. Loop ends at 89.
        // At 89, window is 89..99. 90..99 are 0.5. RMS will be > 0.1.
        // i = 89, triggered = true, potential_start = 89.
        // But then loop ends. triggered is true but min_duration not reached in loop.
        assert!(result.is_none());
    }

    #[test]
    fn test_is_zero_crossing() {
        // Basic crossing tests
        assert!(AutoStartDetection::is_zero_crossing(0.1, -0.1));
        assert!(AutoStartDetection::is_zero_crossing(-0.1, 0.1));

        // Zero crossing to exactly 0.0
        assert!(AutoStartDetection::is_zero_crossing(-0.1, 0.0));
        assert!(AutoStartDetection::is_zero_crossing(0.0, -0.1));

        // No crossing
        assert!(!AutoStartDetection::is_zero_crossing(0.1, 0.1));
        assert!(!AutoStartDetection::is_zero_crossing(-0.1, -0.1));
        assert!(!AutoStartDetection::is_zero_crossing(0.0, 0.0));
        assert!(!AutoStartDetection::is_zero_crossing(0.0, 0.1));

        // -0.0 edge cases
        // -0.0 behaves like 0.0 in comparison operations (e.g. -0.0 == 0.0, -0.0 >= 0.0 is true)
        assert!(!AutoStartDetection::is_zero_crossing(-0.0, 0.1));
        assert!(AutoStartDetection::is_zero_crossing(-0.0, -0.1));

        // Large values
        assert!(AutoStartDetection::is_zero_crossing(100.0, -100.0));
        assert!(AutoStartDetection::is_zero_crossing(-100.0, 100.0));

        // Subnormal values
        let subnormal = f32::from_bits(1);
        assert!(AutoStartDetection::is_zero_crossing(subnormal, -subnormal));
        assert!(AutoStartDetection::is_zero_crossing(-subnormal, subnormal));
    }
}
