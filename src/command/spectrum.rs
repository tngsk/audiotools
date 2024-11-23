use crate::utils::get_walker;
use hound::WavReader;
use plotters::prelude::*;
use plotters::style::RGBAColor;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
use std::path::PathBuf;

use crate::utils::detection::AutoStartDetection;
use crate::utils::time::{TimeRange, TimeSpecification};

// 定数定義
const FONT_FAMILY: &str = "Fira Code";
const BACKGROUND_COLOR: RGBColor = RGBColor(4, 20, 36);

pub fn parse_frequency_annotation(s: &str) -> Result<(f32, String), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("Annotation format should be 'frequency:label'".to_string());
    }

    let freq = parts[0]
        .parse::<f32>()
        .map_err(|_| "Invalid frequency value".to_string())?;

    Ok((freq, parts[1].to_string()))
}

pub fn create_spectrograms(
    input: &PathBuf,
    window_size: usize,
    overlap: f32,
    min_freq: f32,
    max_freq: f32,
    time_range: Option<TimeRange>,
    auto_start: Option<AutoStartDetection>,
    recursive: bool,
    annotations: Option<Vec<(f32, String)>>,
) {
    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            if ext.to_string_lossy().to_lowercase() == "wav" {
                let input_path = PathBuf::from(entry.path());
                let output_path = input_path.with_extension("png");

                match create_spectrogram(
                    &input_path,
                    &output_path,
                    window_size,
                    overlap,
                    min_freq,
                    max_freq,
                    time_range.clone(),
                    auto_start.clone(),
                    annotations.clone(),
                ) {
                    Ok(_) => println!(
                        "Created spectrogram: {} -> {}",
                        input_path.display(),
                        output_path.display()
                    ),
                    Err(e) => eprintln!("Error processing {}: {}", input_path.display(), e),
                }
            }
        }
    }
}

pub fn create_spectrogram(
    input: &PathBuf,
    output: &PathBuf,
    window_size: usize,
    overlap: f32,
    min_freq: f32,
    max_freq: f32,
    time_range: Option<TimeRange>,
    auto_start: Option<AutoStartDetection>,
    annotations: Option<Vec<(f32, String)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;

    // サンプルデータ取得
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.unwrap())
            .collect::<Vec<f32>>()
            .chunks(spec.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect(),
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let max_value = (1 << (bits - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap() as f32 / max_value)
                .collect::<Vec<f32>>()
                .chunks(spec.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        }
    };

    let total_duration = samples.len() as f32 / sample_rate;

    // 自動開始点検出
    let (start_time, end_time) = if let Some(auto_config) = auto_start {
        let detected_start = auto_config
            .detect_start_time(&samples, sample_rate)
            .ok_or("Failed to detect start time")?;

        let end_time = if let Some(range) = time_range {
            TimeRange {
                start: TimeSpecification::Seconds(detected_start),
                end: range.end,
            }
            .resolve(total_duration)
            .map_or(total_duration, |(_, end)| end)
        } else {
            total_duration
        };

        (detected_start, end_time)
    } else if let Some(range) = time_range {
        range.resolve(total_duration)?
    } else {
        (0.0, total_duration)
    };

    // サンプル範囲の計算
    let start_sample = (start_time * sample_rate) as usize;
    let end_sample = (end_time * sample_rate) as usize;
    let samples = samples[start_sample..end_sample].to_vec();

    // FFT処理
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    let hop_size = (window_size as f32 * (1.0 - overlap)) as usize;

    // ハニング窓
    let window: Vec<f32> = (0..window_size)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / window_size as f32).cos()))
        .collect();

    // スペクトログラム計算
    let mut spectrogram = Vec::new();
    let mut i = 0;
    while i + window_size <= samples.len() {
        let mut buffer: Vec<Complex<f32>> = samples[i..i + window_size]
            .iter()
            .zip(window.iter())
            .map(|(&s, &w)| Complex::new(s * w, 0.0))
            .collect();

        fft.process(&mut buffer);

        // 周波数ビンの計算を適切に行う
        let freq_resolution = sample_rate / window_size as f32;
        let spectrum: Vec<f32> = buffer[..window_size / 2]
            .iter()
            .enumerate()
            .map(|(bin, c)| {
                let amplitude = c.norm() / window_size as f32;
                let freq = bin as f32 * freq_resolution;
                if freq >= min_freq && freq <= max_freq {
                    20.0 * amplitude.log10()
                } else {
                    -128.0 // 表示範囲外の周波数は最小値に設定
                }
            })
            .collect();

        spectrogram.push(spectrum);
        i += hop_size;
    }

    // プロット作成
    let root = BitMapBackend::new(output.to_str().unwrap(), (1200, 600)).into_drawing_area();
    root.fill(&BACKGROUND_COLOR)?;

    let min_db = -128.0;
    let max_db = 0.0;

    let title = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Spectrogram");

    let total_time = samples.len() as f32 / sample_rate;
    let total_frames = spectrogram.len();
    let time_per_frame = total_time / total_frames as f32;

    // 対数スケールの目盛り位置計算
    let log_ticks: Vec<f32> = {
        let mut ticks = Vec::new();
        let mut freq = 10.0_f32;
        while freq <= max_freq {
            if freq >= min_freq {
                ticks.push(freq);
                if freq * 2.0 <= max_freq && freq * 2.0 >= min_freq {
                    ticks.push(freq * 2.0);
                }
                if freq * 5.0 <= max_freq && freq * 5.0 >= min_freq {
                    ticks.push(freq * 5.0);
                }
            }
            freq *= 10.0;
        }
        ticks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ticks
    };

    // グラフ設定
    let mut chart = ChartBuilder::on(&root)
        .margin(40)
        .caption(title, (FONT_FAMILY, 24).into_font().color(&WHITE))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0.0..total_time, min_freq..max_freq)?;

    chart
        .configure_mesh()
        .label_style((FONT_FAMILY, 14).into_font().color(&WHITE))
        .light_line_style(RGBAColor(255, 255, 255, 0.05))
        .bold_line_style(RGBAColor(255, 255, 255, 0.05))
        .axis_style(RGBAColor(255, 255, 255, 0.5))
        .x_labels(20)
        .x_label_formatter(&|x| format!("{:.1}", x))
        .y_desc("Frequency (Hz)")
        .x_desc("Time (s)")
        .y_labels(log_ticks.len())
        .y_label_formatter(&|y| format!("{:.0}", y))
        .draw()?;

    // スペクトログラムデータの描画
    let nyquist_freq = sample_rate / 2.0;
    let freq_bins = window_size / 2;

    for (frame, spectrum) in spectrogram.iter().enumerate() {
        let time = frame as f32 * time_per_frame;

        for (bin, &power) in spectrum.iter().enumerate() {
            let freq = (bin as f32 * nyquist_freq) / freq_bins as f32;

            if freq >= min_freq && freq <= max_freq {
                let normalized_power = ((power - min_db) / (max_db - min_db)).max(0.0).min(1.0);
                if normalized_power > 0.0 {
                    let color = {
                        let power = normalized_power.max(0.0).min(1.0);
                        &RGBColor(255, (power * 255.0) as u8, (power * power * 255.0) as u8)
                            .mix(power as f64)
                    };

                    chart.draw_series(std::iter::once(Circle::new(
                        (time, freq),
                        2.0,
                        color.filled(),
                    )))?;
                }
            }
        }
    }

    // アノテーションの描画
    if let Some(annotations) = annotations {
        for (freq, label) in annotations.iter() {
            if *freq >= min_freq && *freq <= max_freq {
                chart.draw_series(LineSeries::new(
                    vec![(0.0, *freq), (total_time, *freq)],
                    &GREEN,
                ))?;
                chart.draw_series(std::iter::once(Text::new(
                    label.to_string(),
                    (total_time - 0.1, *freq - 100.0),
                    (FONT_FAMILY, 16).into_font().color(&GREEN),
                )))?;
            }
        }
    }

    Ok(())
}
