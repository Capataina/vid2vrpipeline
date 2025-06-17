use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
pub struct DownloadLinks(pub Vec<String>);

/// Reads links from the given JSON file and downloads all videos using yt-dlp.
/// To use properly, make sure you fill in the json file.
pub fn download_all(link_file: &str, output_dir: &str) {
    // Create the output folder if it doesn't exist
    // We have a second check for this, but it's still good to keep it there.
    fs::create_dir_all(output_dir).expect("Failed to create download directory");

    // Read and parse the JSON file
    let file_content = fs::read_to_string(link_file).expect("Failed to read the links file");

    let links: Vec<String> = serde_json::from_str(&file_content)
        .expect("Invalid JSON format: expected an array of strings");

    for (index, link) in links.iter().enumerate() {
        println!("\n📁 [{}/{}] Starting download:", index + 1, links.len());
        println!("🔗 {}", link);

        let mut child = Command::new("yt-dlp")
            .args([
                "-f",
                "bv[height<=720][ext=mp4]+ba[ext=m4a]/best[ext=mp4]",
                "--merge-output-format",
                "mp4",
                "--check-formats",
                "--no-part",
                "--no-overwrites",
                "--progress",
                "--newline",
                "-o",
                &format!("{}/%(title)s.%(ext)s", output_dir),
                &link,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start yt-dlp");

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut last_shown_progress = 0;

            for line in reader.lines() {
                if let Ok(line) = line {
                    // Look for download progress lines like "[download] 45.2% of 12.5MB"
                    if line.contains("[download]") && line.contains("%") && !line.contains("100%") {
                        // Try to extract percentage - more robust parsing
                        for word in line.split_whitespace() {
                            if word.ends_with('%') {
                                if let Ok(progress) = word.trim_end_matches('%').parse::<f32>() {
                                    let progress_int = progress as i32;
                                    // Show every 20% increment
                                    if progress_int >= last_shown_progress + 20
                                        && progress_int < 100
                                    {
                                        println!("📥 Download progress: {}%", progress_int);
                                        last_shown_progress = progress_int;
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    // Handle completion
                    else if line.contains("[download] 100%") {
                        println!("📥 Download progress: 100%");
                    }
                }
            }
        }

        let status = child.wait().expect("Failed to wait for yt-dlp");

        if !status.success() {
            println!("❌ Download failed");
        } else {
            println!("✅ Download completed successfully!");
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}
