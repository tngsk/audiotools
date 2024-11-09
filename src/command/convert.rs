use crate::utils::get_walker;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Convert audio files between formats using ffmpeg
pub fn convert_files(
    input: &PathBuf,
    output_dir: Option<&PathBuf>,
    flatten: bool,
    input_format: &[String],
    output_format: &str,
    bit_depth: u8,
    sample_rate: Option<u32>,
    prefix: Option<&str>,
    postfix: Option<&str>,
    recursive: bool,
) {
    // Determine codec and extension based on output format
    let (codec, out_ext) = match output_format.to_lowercase().as_str() {
        "wav" => (
            match bit_depth {
                16 => "pcm_s16le",
                24 => "pcm_s24le",
                _ => panic!("Unsupported bit depth for WAV"),
            },
            "wav",
        ),
        "flac" => ("flac", "flac"),
        "mp3" => ("libmp3lame", "mp3"),
        _ => panic!("Unsupported output format"),
    };

    // Convert input formats to lowercase for comparison
    let input_extensions: Vec<String> = input_format.iter().map(|f| f.to_lowercase()).collect();

    // Process each file in the input directory
    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if input_extensions.contains(&ext_str) {
                // Generate output filename with optional prefix/postfix
                let stem = entry.path().file_stem().unwrap().to_string_lossy();
                let filename = format!(
                    "{}{}{}.{}",
                    prefix.unwrap_or(""),
                    stem,
                    postfix.unwrap_or(""),
                    out_ext
                );

                // Create output path based on options
                let output = if let Some(out_dir) = output_dir {
                    if flatten {
                        // すべてのファイルを出力ディレクトリの直下に配置
                        out_dir.join(&filename)
                    } else {
                        // 入力ディレクトリからの相対パスを維持
                        let relative_path = entry
                            .path()
                            .strip_prefix(input)
                            .unwrap_or(entry.path())
                            .parent()
                            .unwrap_or_else(|| std::path::Path::new(""));
                        let full_output_dir = out_dir.join(relative_path);
                        // 出力ディレクトリが存在しない場合は作成
                        fs::create_dir_all(&full_output_dir)
                            .expect("Failed to create output directory");
                        full_output_dir.join(&filename)
                    }
                } else {
                    // 出力ディレクトリが指定されていない場合は元のファイルと同じ場所に出力
                    entry.path().with_file_name(filename)
                };

                // Build ffmpeg command with conversion parameters
                let mut cmd = Command::new("ffmpeg");
                cmd.arg("-i").arg(entry.path());

                // Add sample rate conversion if specified
                if let Some(rate) = sample_rate {
                    cmd.arg("-ar").arg(rate.to_string());
                }

                // Add format-specific encoding options
                match output_format {
                    "mp3" => {
                        cmd.args(&["-b:a", "320k"]);
                    }
                    "flac" => {
                        cmd.args(&["-compression_level", "8"]);
                    }
                    _ => {}
                }

                // Execute conversion
                cmd.args(&["-acodec", codec])
                    .arg(&output)
                    .output()
                    .expect("Failed to execute ffmpeg");

                println!(
                    "Converted: {} -> {}",
                    entry.path().display(),
                    output.display()
                );
            }
        }
    }
}
