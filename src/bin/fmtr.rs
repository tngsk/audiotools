use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

// Command line interface configuration
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file (output file from audiotools)
    #[arg(short, long)]
    input: PathBuf,

    /// Output JSON file path
    #[arg(short, long)]
    output: PathBuf,

    /// Analysis type (info or loudness)
    #[arg(short, long)]
    type_: String,
}

// Structure for storing audio file information
#[derive(Serialize, Deserialize)]
struct AudioInfo {
    file_path: String,
    format: String,
    size: AudioSize,
    format_info: HashMap<String, String>,
    stream_info: HashMap<String, String>,
}

// Structure for file size information
#[derive(Serialize, Deserialize)]
struct AudioSize {
    formatted: String,
    bytes: u64,
}

// Structure for storing loudness analysis information
#[derive(Serialize, Deserialize)]
struct LoudnessInfo {
    file_path: String,
    format: String,
    size: AudioSize,
    loudness: HashMap<String, String>,
}

// Main function - handles command line arguments and dispatches processing
fn main() {
    let cli = Cli::parse();
    let file = File::open(&cli.input).expect("Failed to open input file");
    let reader = BufReader::new(file);

    match cli.type_.as_str() {
        "info" => process_info(reader, &cli.output),
        "loudness" => process_loudness(reader, &cli.output),
        _ => panic!("Invalid type. Use 'info' or 'loudness'"),
    }
}

// Process audio information and convert to JSON
fn process_info(reader: BufReader<File>, output_path: &PathBuf) {
    let mut audio_files: Vec<AudioInfo> = Vec::new();
    let mut current_file: Option<AudioInfo> = None;
    let mut in_format_section = false;
    let mut in_stream_section = false;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if line.starts_with("File: ") {
            if let Some(file_info) = current_file {
                audio_files.push(file_info);
            }
            current_file = Some(AudioInfo {
                file_path: line.trim_start_matches("File: ").to_string(),
                format: String::new(),
                size: AudioSize {
                    formatted: String::new(),
                    bytes: 0,
                },
                format_info: HashMap::new(),
                stream_info: HashMap::new(),
            });
        } else if line.starts_with("Format: ") && current_file.is_some() {
            if let Some(ref mut file) = current_file {
                file.format = line.trim_start_matches("Format: ").to_string();
            }
        } else if line.starts_with("Size: ") && current_file.is_some() {
            if let Some(ref mut file) = current_file {
                let size_str = line.trim_start_matches("Size: ");
                let parts: Vec<&str> = size_str.split(" (").collect();
                if parts.len() == 2 {
                    file.size.formatted = parts[0].to_string();
                    file.size.bytes = parts[1].trim_end_matches(" bytes)").parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("[FORMAT]") {
            in_format_section = true;
            in_stream_section = false;
        } else if line.starts_with("[/FORMAT]") {
            in_format_section = false;
        } else if line.starts_with("[STREAM]") {
            in_format_section = false;
            in_stream_section = true;
        } else if line.starts_with("[/STREAM]") {
            in_stream_section = false;
        } else if !line.is_empty() && !line.starts_with("[") && current_file.is_some() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();
                if let Some(ref mut file) = current_file {
                    if in_format_section {
                        file.format_info.insert(key.to_string(), value.to_string());
                    } else if in_stream_section {
                        file.stream_info.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    if let Some(file_info) = current_file {
        audio_files.push(file_info);
    }

    let json = serde_json::to_string_pretty(&audio_files).expect("Failed to serialize to JSON");
    std::fs::write(output_path, json).expect("Failed to write JSON file");
}

// Process loudness information and convert to JSON
fn process_loudness(reader: BufReader<File>, output_path: &PathBuf) {
    let mut audio_files: Vec<LoudnessInfo> = Vec::new();
    let mut current_file: Option<LoudnessInfo> = None;

    // Process input file line by line
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if line.starts_with("File: ") {
            // Start processing new file
            if let Some(file_info) = current_file {
                audio_files.push(file_info);
            }
            current_file = Some(LoudnessInfo {
                file_path: line.trim_start_matches("File: ").to_string(),
                format: String::new(),
                size: AudioSize {
                    formatted: String::new(),
                    bytes: 0,
                },
                loudness: HashMap::new(),
            });
        } else if line.starts_with("Format: ") && current_file.is_some() {
            // Extract format information
            if let Some(ref mut file) = current_file {
                file.format = line.trim_start_matches("Format: ").to_string();
            }
        } else if line.starts_with("Size: ") && current_file.is_some() {
            // Parse file size information
            if let Some(ref mut file) = current_file {
                let size_str = line.trim_start_matches("Size: ");
                let parts: Vec<&str> = size_str.split(" (").collect();
                if parts.len() == 2 {
                    file.size.formatted = parts[0].to_string();
                    file.size.bytes = parts[1].trim_end_matches(" bytes)").parse().unwrap_or(0);
                }
            }
        } else if line.contains("LUFS") || line.contains("LU") || line.contains("True Peak") {
            // Extract loudness measurements
            if let Some(ref mut file) = current_file {
                let parts: Vec<&str> = line.trim().split(':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    file.loudness.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    // Add last processed file
    if let Some(file_info) = current_file {
        audio_files.push(file_info);
    }

    // Write JSON output
    let json = serde_json::to_string_pretty(&audio_files).expect("Failed to serialize to JSON");
    std::fs::write(output_path, json).expect("Failed to write JSON file");
}
