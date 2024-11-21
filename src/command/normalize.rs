use super::convert;
use std::path::PathBuf;
use std::process::Command;

pub fn normalize_files(
    input: &PathBuf,
    output_dir: Option<&PathBuf>,
    level: f32,
    input_format: &[String],
    recursive: bool,
    force: bool,
) {
    // プラットフォーム依存のnullデバイスを選択
    let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };

    let output = Command::new("ffmpeg")
        .args(&[
            "-i",
            input.to_str().unwrap(),
            "-af",
            "volumedetect",
            "-vn", // 映像ストリームを無視
            "-sn", // 字幕ストリームを無視
            "-dn", // データストリームを無視
            "-f",
            "null",
            null_device,
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    // stderrからmax_volumeを抽出
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("FFmpeg volume detection output:\n{}", stderr); // デバッグ用出力

    let max_volume = stderr
        .lines()
        .find(|line| line.contains("max_volume:"))
        .and_then(|line| {
            line.split(':')
                .nth(1)
                .and_then(|v| v.trim().trim_end_matches(" dB").parse::<f32>().ok())
        })
        .unwrap_or_else(|| panic!("Failed to detect max volume. FFmpeg output:\n{}", stderr));

    println!("Detected max volume: {} dB", max_volume); // デバッグ用出力

    // ゲイン値を計算
    let gain = level - max_volume;
    println!("Applying gain: {} dB", gain); // デバッグ用出力

    // 正規化を実行
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
}
