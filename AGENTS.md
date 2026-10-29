# Stride agent instructions

Read `/home/sumit/CLAUDE.md` before working on this shared host.

## Resource budget

- Before heavy work, check `df -h / /tmp`, `free -h`, `uptime`, and active jobs.
  Do not start below 10 GiB free disk or 6 GiB MemAvailable.
- Coordinate one host-wide heavy build, test suite, ingestion, or emulator at a
  time. Use `CARGO_BUILD_JOBS=1` and `-- --test-threads=1` for Rust tests.
  Model-waiting agents do not each get a separate heavy-job budget.
- Hold `flock -n /run/user/$(id -u)/cf-heavy.lock <command>` for the whole
  heavy job. If it is held, wait; do not bypass it.
- Use the root workspace's `target/`. Do not create another target tree under
  `android/app/src/main/rust/stride-ffi/`. Avoid new debug/profile/feature
  combinations unless the task needs them.
- Run the smallest relevant tests once. Repeat only after a meaningful change
  or failure. Check disk and temporary-directory growth afterward.
- Reclaim old inactive build output during idle maintenance. Never remove a
  build tree while a build, test, or application uses it.

## Database fixtures

- Routine tests use small synthetic fixtures. A full live-data smoke test is
  opt-in and run once by the integration owner, not independently by every
  agent: `cargo test -p stride-desktop --test database -- --ignored --test-threads=1`.
- `backend/data/stride.db` is live application data. Never modify, rename,
  truncate, encrypt, or delete it for tests. Preserve its WAL and backups.
- Take a SQLite-consistent snapshot, not a raw copy of an active database.
  Share one isolated fixture across related assertions within an invocation.
  Do not cache mutable real-database fixtures across test runs.
- Scope temporary directories to an owning test or fixture. Remove them on
  success and panic; report cleanup failures. Static `OnceLock` values do not
  run destructors at process exit and must not own temporary directories.
- Failed or killed runs may leave artifacts. Remove only exact, inactive paths
  owned by your work, checking PID and open-file references first. Never sweep
  `/tmp`, another agent's target tree, or application data.

These instructions do not enforce admission or OS resource limits. Stop and
report a conflict rather than assuming another agent will make room.
