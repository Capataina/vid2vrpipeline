mod converter;
mod downloader;

use std::fs;
use std::path::Path;

fn ensure_dirs() {
    let download_dir = "downloaded_videos";
    let vr_dir = "vr_versions";

    if !Path::new(download_dir).exists() {
        fs::create_dir_all(download_dir).expect("Failed to create download directory");
    }

    if !Path::new(vr_dir).exists() {
        fs::create_dir_all(vr_dir).expect("Failed to create VR directory");
    }
}

fn main() {
    ensure_dirs();
    downloader::download_all("links.json", "downloaded_videos");
    converter::convert_all_to_vr("downloaded_videos", "vr_versions", "gpu");
}
