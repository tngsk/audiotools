pub mod detection;
pub mod ffprobe;
pub mod time;
pub mod wave_header;

use crate::AUDIO_EXTENSIONS;

use std::path::PathBuf;
use walkdir::WalkDir;

pub fn get_walker(input: &PathBuf, recursive: bool) -> impl Iterator<Item = walkdir::DirEntry> {
    let walker = if recursive {
        WalkDir::new(input)
    } else {
        WalkDir::new(input).max_depth(1)
    };
    walker.into_iter().filter_map(|e| e.ok())
}

// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
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
pub fn is_audio_file(ext: &str) -> bool {
    AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}
