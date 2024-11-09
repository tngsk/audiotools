use crate::utils::{format_size, get_walker, is_audio_file};
use crate::AUDIO_EXTENSIONS;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

// Measure audio loudness according to EBU R128 standard
pub fn measure_loudness(input: &PathBuf, output: Option<&PathBuf>, recursive: bool) {
    let mut output_file =
        output.map(|path| File::create(path).expect("Failed to create output file"));

    println!("Supported formats: {}", AUDIO_EXTENSIONS.join(", "));

    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            // 対応フォーマットのチェック
            if is_audio_file(&ext_str) {
                // ファイルサイズの取得と変換
                let file_size = fs::metadata(entry.path())
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "Unknown size".to_string());

                // ffmpegコマンドの実行
                let loudness_output = Command::new("ffmpeg")
                    .arg("-i")
                    .arg(entry.path())
                    .arg("-filter_complex")
                    .arg("ebur128=peak=true")
                    .arg("-f")
                    .arg("null")
                    .arg("-")
                    .output();

                match loudness_output {
                    Ok(output) => {
                        // 結果の出力
                        let info = String::from_utf8_lossy(&output.stderr);
                        let formatted_output = format!(
                            "File: {}\nFormat: {}\nSize: {}\nLoudness Analysis:\n{}\n",
                            entry.path().display(),
                            ext_str.to_uppercase(),
                            file_size,
                            // EBU R128の関連する行のみを抽出
                            info.lines()
                                .filter(|line| {
                                    line.contains("LUFS")
                                        || line.contains("LU")
                                        || line.contains("Summary")
                                        || line.contains("Integrated")
                                        || line.contains("Loudness")
                                        || line.contains("Range")
                                        || line.contains("True Peak")
                                })
                                .collect::<Vec<&str>>()
                                .join("\n")
                        );

                        if let Some(file) = &mut output_file {
                            writeln!(file, "{}", formatted_output)
                                .expect("Failed to write to output file");
                        } else {
                            println!("{}", formatted_output);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "File: {}\nError: Failed to measure loudness: {}\n",
                            entry.path().display(),
                            e
                        );
                        if let Some(file) = &mut output_file {
                            writeln!(file, "{}", error_msg)
                                .expect("Failed to write to output file");
                        } else {
                            eprintln!("{}", error_msg);
                        }
                    }
                }
            }
        }
    }
}
