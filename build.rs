use std::fs;
use std::path::Path;

fn main() {
    // Ensure the dist directory exists so tauri::generate_context!() doesn't panic
    // during development or initial build.
    let dist_path = Path::new("dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=dist/index.html");
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
        println!(
            "cargo:warning=⚠️  FRONTEND NOT BUILT! Run 'npm run build' first to build the frontend."
        );
        fs::write(
            &index_path,
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Beefcake - Build Required</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #ce412b 0%, #a03221 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 1rem;
            max-width: 600px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
        }
        h1 { margin: 0 0 1rem 0; font-size: 2.5rem; }
        p { margin: 0.5rem 0; font-size: 1.1rem; line-height: 1.6; }
        .command {
            background: rgba(0, 0, 0, 0.5);
            padding: 1rem;
            border-radius: 0.5rem;
            margin: 1.5rem 0;
            font-family: 'Courier New', monospace;
            border-left: 4px solid #ce412b;
            text-align: left;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>⚠️ Frontend Not Built</h1>
        <p>The Beefcake frontend needs to be built before the application can run properly.</p>
        <p><strong>Please run the build command:</strong></p>
        <div class="command">npm run build</div>
        <p>Then restart the application.</p>
        <p style="margin-top: 2rem; font-size: 0.9rem; opacity: 0.8;">
            This placeholder is shown because <code>dist/index.html</code> was missing during the Rust build.
        </p>
    </div>
</body>
</html>"#
        )
        .expect("Failed to create placeholder index.html");
    } else {
        // If index.html exists, check if it's our placeholder or a real build
        if let Ok(content) = fs::read_to_string(&index_path)
            && content.contains("Beefcake - Build Required")
        {
            println!(
                "cargo:warning=⚠️  STILL USING PLACEHOLDER! Run 'npm run build' and then rebuild the backend."
            );
        }
    }

    // Flush to disk if possible
    if let Ok(f) = fs::File::open(dist_path) {
        let _ = f.sync_all();
    }

    println!("cargo:warning=dist directory ready for Tauri build.");
    tauri_build::build();
}
