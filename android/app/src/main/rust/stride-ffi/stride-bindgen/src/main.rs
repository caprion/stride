//! Generates the Kotlin bindings for the phone's Rust core.
//!
//! ```text
//! stride-bindgen <the.udl> <the output directory>
//! ```
//!
//! The Gradle build runs it before the Kotlin compile; the file it writes is
//! the only thing the Kotlin side needs from UniFFI.

use camino::Utf8PathBuf;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let udl = args
        .next()
        .ok_or("the UDL to read is missing: stride-bindgen <the.udl> <the output directory>")?;
    let out = args.next().ok_or(
        "the output directory is missing: stride-bindgen <the.udl> <the output directory>",
    )?;

    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out)?;

    let udl = Utf8PathBuf::from(udl);
    let out = Utf8PathBuf::from_path_buf(out).map_err(|path| {
        format!(
            "the output directory is not valid UTF-8: {}",
            path.display()
        )
    })?;

    uniffi_bindgen::generate_bindings(
        &udl,
        None,
        uniffi_bindgen::bindings::KotlinBindingGenerator,
        Some(&out),
        None,
        None,
        false,
    )?;

    Ok(())
}
