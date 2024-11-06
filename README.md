# AudioTools

AudioTools is a command-line utility for audio file processing, providing functionality for format conversion, metadata extraction, and loudness analysis.

## Features

- Audio format conversion with customizable parameters
- Audio file information extraction
- EBU R128 loudness measurement
- Recursive directory processing
- Output file generation for analysis results
- JSON formatting utility for analysis results

## Project Scope

This tool was developed to address specific personal audio processing tasks and workflows, rather than serving as a general-purpose audio processing solution. While others may find it useful, the feature set and implementation are primarily focused on meeting particular requirements for audio post-production and mastering workflows.

## Prerequisites

- Rust (latest stable version)
- FFmpeg (must be installed and accessible in system PATH)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/_tngsk/audiotools.git
cd audiotools
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Audio Conversion

Convert audio files with various options:

```bash
# Convert to WAV (24-bit)
audiotools convert -i input_dir -I wav,flac,mp3 -O wav -b 24 -r

# Convert to FLAC with custom sample rate
audiotools convert -i input_dir -I wav -O flac -s 48000 -r

# Convert to MP3 with filename prefix/postfix
audiotools convert -i input_dir -I wav,flac -O mp3 --prefix "processed_" --postfix "_320k" -r
```

### Audio Information

Extract audio file metadata:

```bash
# Output to console
audiotools info -i input_dir -f duration,bitrate -r

# Save to file
audiotools info -i input_dir -f duration,bitrate -o info.txt -r
```

### Loudness Analysis

Perform EBU R128 loudness analysis:

```bash
# Output to console
audiotools loudness -i input_dir -r

# Save to file
audiotools loudness -i input_dir -o loudness.txt -r
```

### JSON Formatting

Format the analysis output to JSON:

```bash
# Format info output
fmtr -i info.txt -o info.json -t info

# Format loudness output
fmtr -i loudness.txt -o loudness.json -t loudness
```

## Supported Formats

Input/Output formats:
- WAV (16/24-bit)
- FLAC
- MP3
- AAC
- M4A
- OGG
- WMA
- AIFF
- ALAC
- OPUS

## Command Line Options

### Convert Command
- `-i, --input`: Input directory or file path
- `-I, --input-format`: Input formats to process (comma-separated)
- `-O, --output-format`: Target output format
- `-b, --bit-depth`: Output bit depth for WAV files
- `-s, --sample-rate`: Target sample rate
- `--prefix`: Prefix for output filenames
- `--postfix`: Postfix for output filenames
- `-r, --recursive`: Process directories recursively

### Info Command
- `-i, --input`: Input directory or file path
- `-o, --output`: Output file path
- `-f, --fields`: Fields to display (comma-separated)
- `-r, --recursive`: Process directories recursively

### Loudness Command
- `-i, --input`: Input directory or file path
- `-o, --output`: Output file path
- `-r, --recursive`: Process directories recursively

## Dependencies

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
walkdir = "2.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Note

This tool requires FFmpeg to be installed on your system. Please ensure FFmpeg is properly installed and available in your system PATH before using AudioTools.

To install FFmpeg:

- **Ubuntu/Debian**: `sudo apt-get install ffmpeg`
- **macOS**: `brew install ffmpeg`
- **Windows**: Download from [FFmpeg official website](https://ffmpeg.org/download.html)

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This project heavily relies on FFmpeg, an amazing open-source multimedia framework. Special thanks to the FFmpeg team and contributors for providing such a powerful and reliable tool that makes this project possible.

- FFmpeg: [https://ffmpeg.org/](https://ffmpeg.org/)
- FFmpeg Repository: [https://github.com/FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg)
