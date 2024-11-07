use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

// Define supported audio formats that can be processed
const AUDIO_EXTENSIONS: &[&str] = &[
    "wav", "flac", "mp3", "aac", "m4a", "ogg", "wma", "aiff", "alac", "opus",
];

// Define CLI application structure using clap
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Define available subcommands and their arguments
#[derive(Subcommand)]
enum Commands {
    /// Convert audio files between formats
    Convert {
        /// Input directory or file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory path
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Flatten output directory structure (ignore source directory hierarchy)
        #[arg(short = 'f', long)]
        flatten: bool,

        /// Input formats to process (e.g., wav,flac,mp3)
        #[arg(short = 'I', long, value_delimiter = ',', default_value = "wav")]
        input_format: Vec<String>,

        /// Target output format
        #[arg(short = 'O', long, default_value = "wav")]
        output_format: String,

        /// Output bit depth for WAV files
        #[arg(short, long, default_value = "16")]
        bit_depth: u8,

        /// Target sample rate for conversion
        #[arg(short, long)]
        sample_rate: Option<u32>,

        /// Prefix to add to output filenames
        #[arg(long)]
        prefix: Option<String>,

        /// Postfix to add to output filenames
        #[arg(long)]
        postfix: Option<String>,

        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Display audio file information
    Info {
        /// Input directory or file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file for information
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Fields to display in output
        #[arg(short, long, value_delimiter = ',')]
        fields: Vec<String>,

        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Measure audio loudness using EBU R128
    Loudness {
        /// Input directory or file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file for measurements
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,
    },
}

// Create an iterator for walking through files
fn get_walker(input: &PathBuf, recursive: bool) -> impl Iterator<Item = walkdir::DirEntry> {
    let walker = if recursive {
        WalkDir::new(input)
    } else {
        WalkDir::new(input).max_depth(1)
    };
    walker.into_iter().filter_map(|e| e.ok())
}

// Main function: Parse CLI arguments and dispatch to appropriate handler
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output_dir,
            flatten,
            input_format,
            output_format,
            bit_depth,
            sample_rate,
            prefix,
            postfix,
            recursive,
        } => {
            convert_files(
                &input,
                output_dir.as_ref(),
                flatten,
                &input_format,
                &output_format,
                bit_depth,
                sample_rate,
                prefix.as_deref(),
                postfix.as_deref(),
                recursive,
            );
        }
        Commands::Info {
            input,
            output,
            fields,
            recursive,
        } => {
            get_audio_info(&input, output.as_ref(), &fields, recursive);
        }
        Commands::Loudness {
            input,
            output,
            recursive,
        } => {
            measure_loudness(&input, output.as_ref(), recursive);
        }
    }
}

// Convert audio files between formats using ffmpeg
fn convert_files(
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

// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return format!("0 {}", UNITS[0]);
    }

    let base = 1024_f64;
    let exp = (bytes as f64).ln() / base.ln();
    let unit_index = exp.floor() as usize;

    if unit_index >= UNITS.len() {
        return format!("{} {}", bytes, UNITS[0]);
    }

    let size = bytes as f64 / base.powi(unit_index as i32);
    format!("{:.2} {} ({} bytes)", size, UNITS[unit_index], bytes)
}

// Check if file extension matches supported audio formats
fn is_audio_file(ext: &str) -> bool {
    AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

// Get detailed information about audio files using ffprobe
fn get_audio_info(input: &PathBuf, output: Option<&PathBuf>, fields: &[String], recursive: bool) {
    // Prepare output file if specified
    let mut output_file =
        output.map(|path| File::create(path).expect("Failed to create output file"));

    println!("Supported formats: {}", AUDIO_EXTENSIONS.join(", "));

    // Process each file in the input directory
    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            if is_audio_file(&ext_str) {
                // Get file size information
                let file_size = fs::metadata(entry.path())
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "Unknown size".to_string());

                // Get detailed format information using ffprobe
                let probe_output = Command::new("ffprobe")
                    .arg("-v")
                    .arg("quiet")
                    .arg("-print_format")
                    .arg("json")
                    .arg("-show_format")
                    .arg("-show_streams")
                    .arg(entry.path())
                    .output();

                match probe_output {
                    Ok(output) => {
                        let _probe_info = String::from_utf8_lossy(&output.stdout);

                        // Get additional format-specific information
                        let format_info = Command::new("ffprobe")
                            .arg("-v")
                            .arg("quiet")
                            .arg("-show_entries")
                            .arg(format!("format={}", fields.join(",")))
                            .arg("-show_entries")
                            .arg("stream=codec_name,sample_rate,channels,bit_rate")
                            .arg(entry.path())
                            .output()
                            .ok()
                            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                            .unwrap_or_else(|| "Format information unavailable".to_string());

                        // Format and output information
                        let info = format!(
                            "File: {}\nFormat: {}\nSize: {}\n{}\n",
                            entry.path().display(),
                            ext_str.to_uppercase(),
                            file_size,
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

// Measure audio loudness according to EBU R128 standard
fn measure_loudness(input: &PathBuf, output: Option<&PathBuf>, recursive: bool) {
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
