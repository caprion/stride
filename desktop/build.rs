//! The build script.
//!
//! Two jobs, and both of them are about the frontend bundle:
//!
//! 1. **find it** — the bundle is built once, by `npm --prefix frontend run
//!    build`, and it is embedded in the binary at compile time. This script
//!    tells the crate where it is.
//! 2. **say so when it is not there** — a rig built without the bundle must
//!    still build and still run, and it must say plainly what is missing and
//!    what to do about it. A build failure with a stack trace is not a
//!    sentence an athlete can act on.
//!
//! So the folder is created if it is missing (an empty bundle is a bundle
//! with no files in it), and the crate is told, at compile time, whether the
//! bundle is really there.

use std::path::PathBuf;

fn main() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            println!("cargo:warning=the desktop rig cannot find its own directory");
            return;
        }
    };

    // desktop/ -> the repository root -> frontend/dist
    let root = manifest_dir.parent().unwrap_or(&manifest_dir).to_path_buf();
    let bundle = root.join("frontend").join("dist");

    println!("cargo:rerun-if-changed={}", bundle.display());
    println!("cargo:rerun-if-changed={}", root.join("frontend").display());

    let built = bundle.join("index.html").is_file();
    if !built {
        // The folder has to exist: the embedder is pointed at it, and a
        // missing folder is a compile error with a message nobody wrote.
        let _ = std::fs::create_dir_all(&bundle);
        println!(
            "cargo:warning=the frontend bundle is not built — run `npm --prefix frontend run build`"
        );
    }

    println!("cargo:rustc-env=STRIDE_FRONTEND_DIR={}", bundle.display());
    println!(
        "cargo:rustc-env=STRIDE_FRONTEND_BUILT={}",
        if built { "1" } else { "0" }
    );
}
