# AudioTools

AudioTools is a command-line utility for audio file processing and visualization, designed to assist audio engineers and producers in their workflow.

## Features

### Core Features
- Waveform visualization with amplitude/dB scale
- Audio format conversion with customizable parameters
- Audio level normalization and peak analysis
- Spectrogram generation with frequency annotations
- Auto start/silence detection
- Time range selection for analysis
- Recursive directory processing

### Additional Features
- Audio file metadata extraction
- EBU R128 loudness measurement
- Analysis results export

## Project Scope

This tool was developed to address specific audio visualization and processing needs in music production and sound design workflows. The feature set focuses on practical tools for audio analysis and batch processing, rather than serving as a comprehensive audio workstation.

## Prerequisites

- Rust (latest stable version)
- FFmpeg (must be installed and accessible in system PATH)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/tngsk/audiotools.git
cd audiotools
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Audio Conversion and Normalization

Convert and normalize audio files:

```bash
# Convert to 24-bit WAV with normalization
audiotools convert -i input_dir -O wav -b 24 --level -1.0

# Normalize levels while preserving format
audiotools normalize -i input_dir --level -1.0

# Convert to mono/stereo
audiotools convert -i input.wav --channels 1
```

The `-o, --output-dir` option specifies the destination directory for converted files. By default, the tool preserves the source directory structure and skips existing files. Use the `-f, --flatten` flag to output all files directly to the specified output directory, and `--force` to overwrite existing files.

### Waveform Visualization

Generate detailed waveform visualizations with options:

```bash
# Basic waveform display
audiotools waveform -i input.wav

# Waveform with dB scale and RMS envelope
audiotools waveform -i input.wav --scale decibel --show-rms

# Add time annotations
audiotools waveform -i input.wav --annotate "1.5:start,4.2:end"

# Process with auto start detection
audiotools waveform -i input.wav --auto-start --threshold 0.01
```

### Spectrogram Analysis

Generate spectrograms with customizable parameters:

```bash
# Basic spectrogram
audiotools spectrum -i input.wav

# Detailed frequency analysis
audiotools spectrum -i input.wav --window-size 4096 --overlap 0.85

# Add frequency annotations
audiotools spectrum -i input.wav --annotate "440:A4,880:A5"
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

### Convert/Normalize Commands
- `-i, --input`: Input path
- `-o, --output-dir`: Output directory
- `--level`: Target normalization level (dBFS)
- `--channels`: Output channel count (1=mono, 2=stereo)
- `-b, --bit-depth`: Bit depth for WAV output
- `--force`: Overwrite existing files

### Waveform Command
- `-i, --input`: Input audio file
- `--scale`: Display scale (amplitude/decibel)
- `--show-rms`: Show RMS envelope
- `--start/--end`: Time range selection
- `--auto-start`: Enable automatic start detection
- `--annotate`: Time-based annotations (format: "time:label")

### Spectrum Command
- `-i, --input`: Input audio file
- `--window-size`: FFT window size
- `--overlap`: Window overlap ratio
- `--min/max-freq`: Frequency range
- `--annotate`: Frequency annotations

## Dependencies

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
hound = "3.5"
plotters = "0.3"
rustfft = "6.1"
walkdir = "2.3"
```

## Note

This tool requires FFmpeg to be installed on your system. Please ensure FFmpeg is properly installed and available in your system PATH before using AudioTools.

To install FFmpeg:
- Ubuntu/Debian: `sudo apt-get install ffmpeg`
- macOS: `brew install ffmpeg`
- Windows: Download from [FFmpeg official website](https://ffmpeg.org/download.html)

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.


## Acknowledgments

This project heavily relies on FFmpeg, an amazing open-source multimedia framework. Special thanks to the FFmpeg team and contributors for providing such a powerful and reliable tool that makes this project possible.

- FFmpeg: [https://ffmpeg.org/](https://ffmpeg.org/)
- FFmpeg Repository: [https://github.com/FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg)
