use serde::Deserialize;
use std::fs;
use std::process::Command;

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

    for link in links {
        println!("▶ Downloading: {}", link);

        let status = Command::new("yt-dlp")
            .args([
                "-f",
                "bv[height<=720][ext=mp4]+ba[ext=m4a]/best[ext=mp4]",
                "--merge-output-format",
                "mp4",
                "--check-formats",
                "--no-part",
                "--no-overwrites",
                "-o",
                &format!("{}/%(title)s.%(ext)s", output_dir),
                &link,
            ])
            .status()
            .expect("Failed to run yt-dlp");

        if !status.success() {
            eprintln!("❌ yt-dlp failed on: {}", link);
        }
    }
}
