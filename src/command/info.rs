use crate::audio::wav::WavHeader;
use crate::utils::ffprobe::run_ffprobe;
use crate::utils::{format_size, get_walker, is_audio_file};
use crate::AUDIO_EXTENSIONS;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn get_audio_info(
    input: &PathBuf,
    output: Option<&PathBuf>,
    fields: &[String],
    recursive: bool,
) {
    let mut output_file =
        output.map(|path| File::create(path).expect("Failed to create output file"));

    println!("Supported formats: {}", AUDIO_EXTENSIONS.join(", "));

    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            if is_audio_file(&ext_str) {
                let file_size = fs::metadata(entry.path())
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "Unknown size".to_string());

                // WAVEファイルの場合は詳細なヘッダ情報を読み取る
                let mut additional_info = String::new();
                if ext_str == "wav" {
                    if let Ok(mut file) = File::open(entry.path()) {
                        match WavHeader::read_from_file(&mut file) {
                            Ok(header) => {
                                additional_info = header.format_info();
                            }
                            Err(e) => {
                                additional_info = format!("Error reading WAV header: {}", e);
                            }
                        }
                    }
                }

                // ffprobeによる情報取得
                let probe_result = run_ffprobe(
                    entry.path(),
                    &["-print_format", "json", "-show_format", "-show_streams"],
                );

                match probe_result {
                    Ok(_json_output) => {
                        let format_info = run_ffprobe(
                            entry.path(),
                            &[
                                "-show_entries",
                                &format!("format={}", fields.join(",")),
                                "-show_entries",
                                "stream=codec_name,sample_rate,channels,bit_rate",
                            ],
                        )
                        .unwrap_or_else(|_| "Format information unavailable".to_string());

                        let info = format!(
                            "File: {}\nFormat: {}\nSize: {}\n{}\n{}\n",
                            entry.path().display(),
                            ext_str.to_uppercase(),
                            file_size,
                            additional_info,
                            format_info,
                        );

                        if let Some(file) = &mut output_file {
                            writeln!(file, "{}", info).expect("Failed to write to output file");
                        } else {
                            println!("{}", info);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "File: {}\nError: Failed to get audio info: {}\n",
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
