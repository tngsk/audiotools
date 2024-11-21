use clap::{Parser, Subcommand};
use std::path::PathBuf;

use audiotools::command::{convert, info, loudness};

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

        /// Force overwrite of existing files
        #[arg(long)]
        force: bool,

        /// Number of output channels (1=mono, 2=stereo)
        #[arg(long, value_name = "CHANNELS")]
        channels: Option<u8>,
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
            force,
            channels,
        } => {
            convert::convert_files(
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
                force,
                channels,
            );
        }
        Commands::Info {
            input,
            output,
            fields,
            recursive,
        } => {
            info::get_audio_info(&input, output.as_ref(), &fields, recursive);
        }
        Commands::Loudness {
            input,
            output,
            recursive,
        } => {
            loudness::measure_loudness(&input, output.as_ref(), recursive);
        }
    }
}
