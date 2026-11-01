//! Writes the string catalogue's export — `crates/stride-core/strings.json`.
//!
//! The export carries the strings **and the rules with them**: the ban list,
//! the closed direction set, the minimum pattern sample and the vital words.
//! The frontend's lint reads this file, so the two lints cannot drift apart
//! without a test failing.
//!
//! ```text
//! cargo run -p stride-core --example export_catalogue
//! ```

use std::path::Path;

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("strings.json");
    let json = stride_core::strings::catalogue_json();
    if let Err(error) = std::fs::write(&path, json) {
        eprintln!("export_catalogue: cannot write {}: {error}", path.display());
        std::process::exit(1);
    }
    println!("wrote {}", path.display());
}
