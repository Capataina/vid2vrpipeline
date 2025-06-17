use std::fs;
use std::path::Path;
use std::process::Command;

/// Converts all .mp4 videos in the input folder into SBS VR format
/// and writes them into the output folder using either CPU or GPU.
pub fn convert_all_to_vr(input_dir: &str, output_dir: &str, mode: &str) {
    fs::create_dir_all(output_dir).expect("Failed to create VR output directory");

    let entries = fs::read_dir(input_dir).expect("Failed to read input directory");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
                let input_path = path.to_string_lossy();
                let file_stem = path.file_stem().unwrap().to_string_lossy();
                let output_path = format!("{}/{}_VR.mp4", output_dir, file_stem);

                if Path::new(&output_path).exists() {
                    println!("🔁 Skipping (already converted): {}", file_stem);
                    continue;
                }

                println!(
                    "🎬 Converting to VR ({}): {}",
                    mode.to_uppercase(),
                    file_stem
                );

                let mut args = vec![
                    "-i",
                    &input_path,
                    "-filter_complex",
                    "[0:v]scale=iw/2:ih[left];[0:v]scale=iw/2:ih[right];[left][right]hstack",
                ];

                // Encoder choice based on mode
                match mode.to_lowercase().as_str() {
                    "gpu" => {
                        args.extend(["-c:v", "h264_nvenc", "-cq", "18", "-preset", "slow"]);
                    }
                    "cpu" => {
                        args.extend(["-c:v", "libx264", "-crf", "18", "-preset", "slow"]);
                    }
                    other => {
                        eprintln!("❌ Unknown mode '{}'. Use 'cpu' or 'gpu'.", other);
                        continue;
                    }
                }

                args.push(&output_path);

                let status = Command::new("ffmpeg")
                    .args(&args)
                    .status()
                    .expect("Failed to run ffmpeg");

                if !status.success() {
                    eprintln!("❌ ffmpeg failed on: {}", input_path);
                }
            }
        }
    }
}
