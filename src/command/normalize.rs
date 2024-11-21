use super::convert;
use hound::WavReader;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn detect_peak_level(input: &PathBuf) -> Result<f32, Box<dyn std::error::Error>> {
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

pub fn normalize_files(
    input: &PathBuf,
    output_dir: Option<&PathBuf>,
    level: f32,
    input_format: &[String],
    recursive: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let peak_dbfs = detect_peak_level(input)?;
    println!("Detected peak level: {} dBFS", peak_dbfs);

    let gain = level - peak_dbfs;
    println!("Applying gain: {} dB", gain);

    convert::convert_files(
        input,
        output_dir,
        false,
        input_format,
        "wav",
        24,
        None,
        None,
        Some("_normalized"),
        recursive,
        force,
        None,
        Some(gain),
    );

    Ok(())
}
