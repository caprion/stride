# The FFI seam

The phone runs the Rust core **in this process**. This directory is the
seam between the two: the surface the Kotlin shell calls, and the boundary
where a panic in the core stops being a process kill.

## What is here

| file | what it is |
|---|---|
| `src/stride.udl` | the surface: six functions, every one answering with a JSON envelope |
| `src/seam.rs` | the trait the core has to meet, and the envelope helpers |
| `src/boundary.rs` | the panic boundary: a panic becomes an error the app can show |
| `src/stub.rs` | the core that is not in this build yet |
| `build.rs` | the UniFFI scaffolding, and the `unsafe` allowance for it alone |
| `../stride-bindgen` | the generator that writes the Kotlin bindings |

## The rules the seam keeps

* **The Garmin key is an argument.** It is passed in, used and dropped. No
  function here returns it, and nothing here logs it.
* **The database key is an argument.** It is never written anywhere but the
  file the Keystore key protects.
* **A panic is an error.** Every call across the boundary is wrapped: the
  panic is caught, turned into `{"ok": false, "status": 500, ...}`, and
  handed back as an ordinary answer. The panic's *payload* is not logged —
  a payload is whatever the failing code was holding, and that might be the
  key. The location is logged; the message is not.
* **The routes are the frozen ones.** The route table is
  the frozen route table (`docs/contract/api-paths.md`) in the repository root. This crate does not
  decide which route a path names — the router does — and it does not add
  one.
* **`unsafe` is allowed here and nowhere else.** The crate denies
  `unsafe_code`; the generated scaffolding is the one exception, and the
  build script gives it the allowance item by item.

## What the core has to provide

The core is `crates/stride-api` in the workspace above this directory. When
it lands, it has to satisfy `seam::Core`:

```rust
pub trait Core: Send + Sync {
    fn open(&self, db_path: &str, db_key_hex: &str) -> Result<Value, String>;
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Value;
    fn garmin_sync(&self, api_key: &str, days: i64) -> Value;
    fn set_garmin_key(&self, api_key: &str) -> Value;
    fn set_garmin_connected(&self, connected: bool) -> Value;
    fn status(&self) -> Value;
}
```

The envelope is the one the desktop's router answers with, so the phone and
the desktop cannot disagree about a number:

```text
{"ok": true,  "status": 200, "body": { ... }}
{"ok": false, "status": 404, "error": {"detail": "..."}}
```

The router matches on the path **without** the query string: the query
string is not part of the route and never changes which route is called.

## Until then: the stub

`crates/stride-api` is not in the tree yet, so the seam is built against
`stub::StubCore`, and the APK answers every route with:

> the Rust core is not in this build. Nothing is shown until it is.

A stub can refuse; it cannot be wrong. There is no route it answers with a
number, so an APK built against it shows nothing at all — which is the only
honest thing an APK without a core can do.

The release build refuses to be assembled against the stub. A published APK
whose core is the stub would be a product that does nothing.

## Building it

The Gradle build does this, under the host lock:

```bash
flock /run/user/$(id -u)/cf-heavy.lock ./gradlew :app:assembleDebug
```

which runs, in order:

```bash
cargo run --release --manifest-path android/app/src/main/rust/stride-bindgen/Cargo.toml \
    -- android/app/src/main/rust/stride-ffi/src/stride.udl <the generated directory>
cargo ndk -t arm64-v8a build --release \
    --manifest-path android/app/src/main/rust/stride-ffi/Cargo.toml
```

The tests are plain Rust tests:

```bash
cargo test --manifest-path android/app/src/main/rust/stride-ffi/Cargo.toml
```
