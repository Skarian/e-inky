use std::fs;
use std::io::Write;
use std::path::Path;

fn ensure_icon() {
    // Paths inside build scripts are relative to the crate root (`src-tauri/`).
    let icon_path = Path::new("icons/icon.png");

    if icon_path.exists() {
        return;
    }

    if let Some(parent) = icon_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            panic!("failed to create icon directory: {error}");
        }
    }

    // 1x1 transparent PNG to satisfy tauri-build without committing binary assets.
    const PLACEHOLDER_ICON: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x0bIDATx\x9cc```\x00\x00\x00\x05\x00\x01\x0d\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82";

    let mut file = fs::File::create(icon_path)
        .unwrap_or_else(|error| panic!("failed to create placeholder icon: {error}"));
    file.write_all(PLACEHOLDER_ICON)
        .unwrap_or_else(|error| panic!("failed to write placeholder icon: {error}"));
}

fn main() {
    ensure_icon();
    tauri_build::build();
}
