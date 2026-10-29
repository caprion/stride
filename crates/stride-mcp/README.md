# `stride-mcp` — the harness tools

The MCP surface that lets Claude Code, Codex or Gemini CLI ask questions
about the athlete's own runs. The tools are written **once, over the Stride
router**, so the desktop and the phone answer the same question the same way
and the numbers are the numbers the app shows.

The design is `docs/harness-integration.md`; the frozen contract is
`docs/rust-core-spec.md`.

## The tools

Nine read-only tools. Each answers with the same envelope:

```json
{ "tool": "...", "data": { ... }, "coverage": { ... }, "limitations": [ ... ] }
```

`coverage` says which activity or which date range the answer came from.
`limitations` says what the answer cannot be used for.

| tool | what it answers | inputs |
|---|---|---|
| `training_digest` | how the training is going: load, change against the period before, the plan and how it was kept, readiness, the mechanics, the records | `period` (`7d`, `28d`, `block`, `custom`), `from`, `to` |
| `search_activities` | find runs | `from`, `to`, `sport`, `min_distance_m`, `max_distance_m`, `q`, `sort`, `limit` (1–100), `cursor` |
| `get_activity` | one run: summary, splits, series, zones, route | `activity_id`, `include` (`laps`, `streams`, `zones`, `route`) |
| `activity_analysis` | what happened in one run: the splits as segments, what the numbers show, an optional comparison | `activity_id`, `compare_with`, `focus` |
| `training_statistics` | the trends: volume, pace, elevation, zones, records | `from`, `to`, `group_by` (`week`, `month`), `metrics` |
| `plan_context` | the plan and how it was kept | `view` (`current`, `week`, `block`, `results`), `date`, `block_id`, `from`, `to` |
| `physiology_context` | readiness, sleep, HRV, resting heart rate, day by day | `from`, `to`, `focus` |
| `fingerprint_trends` | the mechanics over time | `from`, `to`, `metrics`, `group_by` (`run`, `week`, `long_run`) |
| `compare_runs` | two to five runs on one distance axis | `activity_ids` (2–5), `points` (20–240), `metrics` |

## The rules the layer holds

- **Read-only.** Nothing writes. The design's two write tools
  (`update_activity_notes`, `edit_plan`) are deliberately absent: they need an
  audit trail and an undo token, and the router has no audit surface to write
  that trail to. There is no switch to turn one on.
- **Bounded.** Every list is capped (`MAX_ITEMS` 100, `MAX_SAMPLES` 240,
  `MAX_SEGMENTS` 60, `MAX_SERIES` 120) and every answer is kept inside
  `MAX_OUTPUT_BYTES` (128 KiB) by a backstop that trims the largest list and
  says it did.
- **Attributable.** Every answer carries the activity or the date range it
  came from.
- **A disclosure, not a dump.** Only the fields an athlete would want in a
  model's context are passed on: no Garmin identifiers or links, no device
  names, no free-text notes, no credentials, no paths. The route — the
  athlete's location — only when it is asked for by name, and then downsampled.
- **Honest.** A missing measurement is `null`, never zero. An estimate is
  labelled as one. The recommendation for the day is a suggestion with its
  evidence, never a command. A slower split on a climb is reported as a
  coincidence, not as a cause.

## How a host wires it

The host owns the process, the pairing capability and the revocation; this
crate is the layer that is the same on both hosts.

```rust
use std::sync::Arc;
use stride_mcp::Tools;

// The router and the tools are built with the same day, so "this week"
// means the same week on both sides of the seam.
let tools = Tools::over(store, day);            // or Tools::over_now(store)
let stdin = std::io::stdin();
let mut stdout = std::io::stdout();
stride_mcp::serve(tools, stdin.lock(), &mut stdout)?;
```

A host that already holds a router builds the tools from it:

```rust
let api = stride_api::Api::new(store);
let tools = Tools::new(Arc::new(api));          // or Tools::with_today(api, day)
```

`serve` speaks JSON-RPC 2.0 over stdio, one message per line, and answers
`initialize`, `tools/list`, `tools/call` and `ping`. Notifications get no
answer. A tool that fails answers with `isError: true` and the message; an
unknown tool is a protocol error.

**Nothing but protocol messages may be written to stdout** — the harness
reads that stream. Diagnostics go to stderr.

The `initialize` handshake negotiates the protocol version: the server
answers with the version the client asked for when it implements it
(`2025-06-18` and `2024-11-05` today), and with its own when it does not.
A tool's answer is returned both as text and as `structuredContent`.

A host that reaches Stride over a socket rather than in process implements
`stride_mcp::StrideApi` and hands that to `Tools::new` instead.

## What it does not do

- **No HTTP client and no binary.** The crate depends on `stride-api` and
  nothing else, so the sidecar that talks to a running desktop over HTTP is
  not here; the desktop and the phone both embed this crate and speak MCP
  stdio from their own process.
- **No write tools.** See above.
- **No golden fixture of its own.** The fixtures in `backend/tests/golden/`
  are the reference's; `tests/golden.rs` drives the readiness and plan cases
  through the tools and checks that the reference's numbers come back
  unchanged.
