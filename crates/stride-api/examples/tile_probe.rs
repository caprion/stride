//! Is the tile proxy working?
//!
//! A diagnostic, not a test: it reaches out to OpenStreetMap, so it must not
//! run in a test suite.
//!
//! ```text
//! cargo run -p stride-api --example tile_probe
//! ```
//!
//! It fetches one tile through the proxy, prints where the answer came from
//! — the upstream or the cache — and checks that the answer is a PNG.

use std::sync::Arc;

use stride_api::Backend;
use stride_api::TileBackend;

/// The tile it asks for: somewhere in the middle of the grid.
const ZOOM: i64 = 12;
const X: i64 = 2048;
const Y: i64 = 2048;

fn main() {
    let cache = std::env::temp_dir().join("stride-tile-probe");
    let _ = std::fs::remove_dir_all(&cache);
    let database = std::env::temp_dir().join("stride-tile-probe.db");
    let _ = std::fs::remove_file(&database);

    let store =
        match stride_store::Store::open_and_migrate(&database, b"0123456789abcdef0123456789abcdef")
        {
            Ok(store) => store,
            Err(err) => {
                println!("the probe could not open its database: {err}");
                return;
            }
        };
    let tiles = TileBackend::new(stride_api::store::backend(Arc::new(store)), &cache);

    match tiles.tile(ZOOM, X, Y) {
        Ok(tile) => {
            let is_png = tile.bytes.starts_with(&[0x89, b'P', b'N', b'G']);
            println!(
                "the tile came from {}: {} bytes, a PNG: {}",
                tile.source,
                tile.bytes.len(),
                is_png
            );
        }
        Err(err) => println!("the proxy could not fetch the tile: {err}"),
    }
    match tiles.tile(ZOOM, X, Y) {
        Ok(tile) => println!("the second request came from {}", tile.source),
        Err(err) => println!("the second request failed: {err}"),
    }
}
