//! The build script.
//!
//! Three jobs, and all of them are about the app bundle:
//!
//! 1. **find the development bundle** — `frontend/dist` is built by the
//!    frontend's own toolchain and embedded while a developer is working.
//! 2. **find the released bundle** — `desktop/assets` carries the same app
//!    for a clone that only has Rust. A stranger does not need Node or npm
//!    to build the rig.
//! 3. **say so when neither is there** — a rig built without a bundle must
//!    still build and still run, and it must say plainly what is missing.
//!
//! The released bundle is preferred in a clean checkout. A present
//! `frontend/dist` is preferred in a development checkout, so the normal
//! edit-reload loop still sees the change that was just made.

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
    let released = manifest_dir.join("assets");

    println!("cargo:rerun-if-changed={}", bundle.display());
    println!("cargo:rerun-if-changed={}", root.join("frontend").display());
    println!("cargo:rerun-if-changed={}", released.display());

    let development = bundle.join("index.html").is_file();
    let released_built = released.join("index.html").is_file();
    // The development folder always exists. The embedder is pointed at it
    // even when the released bundle is the one being served, and a missing
    // folder is a compile error with a message nobody wrote.
    let _ = std::fs::create_dir_all(&bundle);
    if !development && !released_built {
        let _ = std::fs::create_dir_all(&released);
        println!("cargo:warning=the app bundle is not present in this checkout");
    }

    println!("cargo:rustc-env=STRIDE_FRONTEND_DIR={}", bundle.display());
    println!(
        "cargo:rustc-env=STRIDE_FRONTEND_BUILT={}",
        if development || released_built {
            "1"
        } else {
            "0"
        }
    );
    println!(
        "cargo:rustc-env=STRIDE_USE_DEVELOPMENT_BUNDLE={}",
        if development { "1" } else { "0" }
    );
}
