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
| `src/real.rs` | the core: `stride-api`'s router, in this process |
| `src/stub.rs` | the core that is not there, for the builds that ask for it |
| `build.rs` | the UniFFI scaffolding |
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
  `docs/contract/api-paths.md` in the repository root. This crate does not
  decide which route a path names — the router does — and it does not add
  one.
* **`unsafe` is allowed here and nowhere else.** The workspace's lint table
  sets `unsafe_code = "allow"` — "warn" would make unsafe denied
  everywhere, including this one place — and the rule is enforced by
  `scripts/check-contract-greps.sh` (no `unsafe` outside the FFI crate),
  which a `#[allow]` cannot talk its way out of.

## The retry rule: the shell owns it

**`stride_garmin_sync` is one attempt.** The core performs exactly one
token exchange and one HTTP attempt and then answers. When the token has
expired the answer is the typed failure `needs_reconnect`:

```text
{"ok": false, "status": 401, "error": {"code": "needs_reconnect", "detail": "..."}}
```

The **caller** may make exactly **one** more attempt, and that attempt
performs one fresh token exchange. A second failure stops: the caller
surfaces the error and the next scheduled sync tries again.

This is the contract's rule, not this crate's invention:
`docs/contract/core-contract.md` §7 — "The retry rule, and who owns it".
The client does not re-exchange and try again, because if it did *and* the
shell did, the phone would exchange and retry twice and "exactly one" would
be a coincidence rather than a rule.

The shell's side of it is `PullPolicy` and `SyncEngine`, and the shell's
test asserts one retry per sync.

## What the core has to provide

The core is `crates/stride-api` in the workspace above this directory, and
it satisfies `seam::Core` through `src/real.rs`:

```rust
pub trait Core: Send + Sync {
    fn open(&self, db_path: &str, db_key_hex: &str) -> Result<Value, String>;
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Value;
    fn garmin_sync(&self, key: &str, days: i64) -> Value;
    fn set_garmin_key(&self, key: &str) -> Value;
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

## The core, and the stub

The core is `crates/stride-api`, and the seam calls it **in this process**
through the api's own FFI surface — the router's `ffi.rs` is the other
side of the seam's `stride_call`. So the phone's answer is the router's
answer, and the desktop's answer is the same router's answer.

The body crosses the boundary as **the bytes the core wrote**: the seam
carries it as text, and the shell hands it to the app as it stands. A body
that was parsed and written out again on the way past would not be those
bytes — the numbers would lose their decimals and the slashes would gain
their escapes — and then the phone's answer would not be the router's.

The stub is still here, and it is still honest: a stub can refuse; it
cannot be wrong. A build without the `real-core` feature runs it, and every
route answers with:

> the Rust core is not in this build. Nothing is shown until it is.

The release build refuses to be assembled against the stub. A published APK
whose core is the stub would be a product that does nothing.

## Building it

The crate is a member of the repository's workspace, so the root gate sees
it:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The Gradle build cross-compiles it for the phone, under the host lock:

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
