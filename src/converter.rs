use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Converts all .mp4 videos in the input folder into SBS VR format
/// and writes them into the output folder using either CPU or GPU.
pub fn convert_all_to_vr(input_dir: &str, output_dir: &str, mode: &str) {
    fs::create_dir_all(output_dir).expect("Failed to create VR output directory");

    let entries = fs::read_dir(input_dir).expect("Failed to read input directory");
    let mut conversion_count = 0;
    let mut total_files = 0;

    // Count total MP4 files first
    for entry in fs::read_dir(input_dir).expect("Failed to read input directory") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
                total_files += 1;
            }
        }
    }

    if total_files == 0 {
        println!("🔍 No MP4 files found in the input directory.");
        return;
    }

    println!("\n🎬 Starting VR conversion process...");
    println!("📁 Found {} MP4 file(s) to convert", total_files);
    println!("⚙️  Mode: {}", mode.to_uppercase());
    println!("═══════════════════════════════════════════════════════════════\n");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
                conversion_count += 1;
                let input_path = path.to_string_lossy();
                let file_stem = path.file_stem().unwrap().to_string_lossy();
                let output_path = format!("{}/{}_VR.mp4", output_dir, file_stem);

                if Path::new(&output_path).exists() {
                    println!(
                        "🔁 [{}/{}] Skipping (already converted): {}",
                        conversion_count, total_files, file_stem
                    );
                    continue;
                }

                println!(
                    "🎥 [{}/{}] Converting: {}",
                    conversion_count, total_files, file_stem
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

                let mut child = Command::new("ffmpeg")
                    .args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to start ffmpeg");

                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    let mut line_count = 0;
                    let mut last_shown_frame = 0;

                    for line in reader.lines() {
                        if let Ok(line) = line {
                            line_count += 1;

                            // Look for lines with frame= and speed= (progress lines)
                            if line.contains("frame=") && line.contains("speed=") {
                                // Extract frame number for progress tracking
                                if let Some(frame_str) = line
                                    .split("frame=")
                                    .nth(1)
                                    .and_then(|s| s.split_whitespace().next())
                                {
                                    if let Ok(current_frame) = frame_str.parse::<i32>() {
                                        // Show progress every 1000 frames to avoid spam
                                        if current_frame >= last_shown_frame + 1000 {
                                            // Extract speed if available
                                            let speed = line
                                                .split("speed=")
                                                .nth(1)
                                                .and_then(|s| s.split_whitespace().next())
                                                .unwrap_or("?");

                                            println!(
                                                "⚡ Processing: frame {} at {} speed",
                                                current_frame, speed
                                            );
                                            last_shown_frame = current_frame;
                                        }
                                    }
                                }
                            }
                            // Show progress every 100 lines as fallback if frame parsing fails
                            else if line_count % 100 == 0 && line.contains("time=") {
                                println!("⚡ Processing...");
                            }
                        }
                    }
                }

                let status = child.wait().expect("Failed to wait for ffmpeg");

                if !status.success() {
                    println!("❌ Conversion failed");
                } else {
                    println!("✅ Conversion completed: {}_VR.mp4", file_stem);
                }
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            }
        }
    }

    println!("🎉 VR conversion process completed!");
    println!("📊 Processed {}/{} files", conversion_count, total_files);
}
