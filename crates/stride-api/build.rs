//! The build's fingerprint.
//!
//! `STRIDE_VERSION` is read with `option_env!`, and **cargo does not track an
//! arbitrary environment variable**: change it and cargo considers the crate
//! unchanged, so the binary keeps the version of the *previous* build. That is
//! how the app shipped "0.1.0" while the release was called 0.10.0.
//!
//! So the variable is declared here, and cargo rebuilds when it changes.
fn main() {
    println!("cargo:rerun-if-env-changed=STRIDE_VERSION");
    println!("cargo:rerun-if-env-changed=STRIDE_CORE");
}
