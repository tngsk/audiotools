use super::convert;
use crate::utils::detection::detect_peak_level;
use crate::utils::get_walker;
use std::path::PathBuf;

pub fn normalize_files(
    input: &PathBuf,
    output_dir: Option<&PathBuf>,
    level: f32,
    input_format: &[String],
    recursive: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 入力フォーマットを小文字に変換
    let input_extensions: Vec<String> = input_format.iter().map(|f| f.to_lowercase()).collect();

    // フォルダ内のファイルを走査
    for entry in get_walker(input, recursive) {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if input_extensions.contains(&ext_str) {
                // 各ファイルのピークレベルを検出
                match detect_peak_level(&entry.path().to_path_buf()) {
                    Ok(peak_dbfs) => {
                        println!(
                            "Processing: {} (Peak level: {:.1} dBFS)",
                            entry.path().display(),
                            peak_dbfs
                        );

                        let gain = level - peak_dbfs;
                        println!("Applying gain: {:.1} dB", gain);

                        // 変換処理の実行
                        convert::convert_files(
                            &entry.path().to_path_buf(),
                            output_dir,
                            false,
                            &[ext_str],
                            "wav",
                            24,
                            None,
                            None,
                            Some(&format!("_normalized_{}dB", level)),
                            false,
                            force,
                            None,
                            Some(level),
                        );
                    }
                    Err(e) => {
                        println!("Error processing {}: {}", entry.path().display(), e);
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}
