use std::sync::Arc;
// A one-off check that the tile proxy really talks to OpenStreetMap.
fn main() {
    let dir = std::env::temp_dir().join("stride-tile-live");
    let _ = std::fs::remove_dir_all(&dir);
    let backend = stride_api::store::backend(Arc::new(
        stride_store::Store::open_and_migrate(
            &std::env::temp_dir().join("stride-tile-live.db"),
            b"0123456789abcdef0123456789abcdef",
        )
        .expect("the store opens"),
    ));
    let tiles = stride_api::TileBackend::new(backend, &dir);
    match tiles.tile(12, 2048, 2048) {
        Ok(tile) => {
            let is_png = tile.bytes.starts_with(&[0x89, b'P', b'N', b'G']);
            println!("fetched {} bytes, source {}, png: {}", tile.bytes.len(), tile.source, is_png);
            // the second call must come from the cache
            match tiles.tile(12, 2048, 2048) {
                Ok(second) => println!("second call source: {}", second.source),
                Err(err) => println!("second call failed: {err}"),
            }
        }
        Err(err) => println!("the fetch failed: {err}"),
    }
}
