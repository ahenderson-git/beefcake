use std::fs;
use std::path::Path;

fn main() {
    // Ensure the dist directory exists so tauri::generate_context!() doesn't panic
    // during development or initial build.
    let dist_path = Path::new("dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!(
        "cargo:warning=Checking for dist directory at: {}",
        dist_path
            .canonicalize()
            .unwrap_or(dist_path.to_path_buf())
            .display()
    );

    if !dist_path.exists() {
        println!("cargo:warning=dist directory NOT found, creating it...");
        fs::create_dir_all(dist_path).expect("Failed to create dist directory");
    }

    // Always ensure index.html exists in dist
    let index_path = dist_path.join("index.html");
    if !index_path.exists() {
        println!("cargo:warning=index.html NOT found in dist, creating placeholder...");
        fs::write(&index_path, "<html><body>Placeholder</body></html>")
            .expect("Failed to create placeholder index.html");
    }

    // Flush to disk if possible
    if let Ok(f) = fs::File::open(dist_path) {
        let _ = f.sync_all();
    }

    println!("cargo:warning=dist directory ready for Tauri build.");
    tauri_build::build();
}
