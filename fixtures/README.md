# The frozen golden corpus

This directory freezes the observable behaviour of Stride's domain logic as
**data**, so that the v2 Rust core can be *proved* equal to the old core instead
of believed equal to it. Every file here came out of `/home/sumit/stride` (the
parity oracle) and was **sanitised before it was committed**. Nothing here is
generated at build time, and nothing here is Python.

The scanner gate that keeps it clean is `scripts/scan-privacy.sh` (plus
`scripts/scan-secrets.sh`). The standard it enforces is the same one the old
repo's privacy guard enforced: no committed file may point at a real place,
carry a real name, or carry a Garmin identifier.

## Layout and provenance

| path | source in `/home/sumit/stride` | what it is |
|---|---|---|
| `fit/*.fit` | `backend/tests/golden/fixtures/*.fit` | the 15 synthetic FIT inputs |
| `golden/cases.json` | `backend/tests/golden/cases.json` | the frozen index of every input (sha256, size, sport, device, dates) |
| `golden/scenarios/readiness.json` | `backend/tests/golden/scenarios/readiness.json` | the physiology rows and day for the readiness scenario |
| `golden/scenarios/plan.json` | `backend/tests/golden/scenarios/plan.json` | the plan-engine inputs |
| `golden/golden/*.json` | `backend/tests/golden/golden/*.json` | the golden outputs (15 FIT cases + `plan` + `readiness`) |
| `stride-api/api_golden.json` | `crates/stride-api/tests/fixtures/api_golden.json` | the recorded API responses over the seeded corpus |
| `stride-api/sync.json`, `plan_import.json`, `openapi_paths.json` | `crates/stride-api/tests/fixtures/` | the sync, plan-import and OpenAPI-path fixtures |
| `stride-core/plan_sources/inputs.json`, `expected.json` | `crates/stride-core/tests/fixtures/plan_sources/` | the plan-source import inputs and the reference output |
| `stride-garmin/*.json` | `crates/stride-garmin/tests/fixtures/*.json` | the Garmin Personal API payloads and the reference output |
| `scan-tests/*` | new (not from the old repo) | the deliberate bad/good fixtures that prove the scanners work |

The old repo's generator scripts (`make_fixtures.py`, `fit_writer.py`,
`make_expected.py`, `record_*.py`) are **Python and are not copied** — the rule
is no Python in this tree. They live in the old repo, which is the oracle. The
procedure below is the shell equivalent for re-extraction.

## What was sanitised, and how

1. **Coordinates.** Every position in every FIT file and every golden JSON is
   the synthetic anchor `-35.0, -140.0` (the South Pacific, far from any coast
   and any real route). The guard is `scripts/scan-privacy.sh`, which fails on
   any coordinate that is not the anchor and falls in the **plausible-real
   band** — a fictional populated region, chosen to stand in for "somewhere
   real" without naming where the athlete runs. The canonical example
   coordinate is `15.0000, 78.0000`.
2. **Dates.** The corpus already lives on a fixed synthetic calendar (a week in
   March 2026, the literal values the old generator wrote). No real athlete
   dates were present, so no shift was needed; the relative offsets the plan
   engine is sensitive to are preserved exactly.
3. **Names and identifiers.** Two files carried a fabricated-but-plausible
   personal name and a Garmin user id: `stride-garmin/profile.json` and
   `stride-garmin/expected.json` (the reference output for that input). The
   values were replaced **by field**, not by literal:

   | field | sanitised |
   |---|---|
   | `displayName`, `fullName`, `nickName`, `athlete_name` | `ATHLETE` |
   | `userName` | `athlete` |
   | `id`, `userProfileId` | `0` |

4. **Identifiers.** The FIT serial numbers and the device names are synthetic —
   the old repo documents that product ids and serial numbers are invented. No
   Garmin tokens, keys, emails or account credentials were present anywhere;
   `scripts/scan-secrets.sh` asserts that stays true (it fails on private-key
   blocks, `GCDC_` Garmin keys, AWS key ids, and `Authorization:` headers).

## The privacy standard (reproduced)

The old repo's privacy guard stated the rule and its reason, and this corpus
keeps it, with the specific values replaced by fictional ones:

- **Anchor** = `-35.0, -140.0`; a position may stray no more than 1.0° from it.
- **The plausible-real band** the scan flags = lat `14–16 N`, lon `76–78 E`
  (fictional — not the athlete's routes). The canonical example coordinate is
  `15.0000, 78.0000`.
- **The breach this prevents**: the old repo once seeded synthetic demo data at
  the athlete's own coordinates. The guard exists so a real coordinate cannot
  reappear — in the old repo it was a test; here it is `scripts/scan-privacy.sh`,
  which runs in CI on every push and PR.

## Re-extracting (repeatable, no Python)

Run from `/home/sumit/stride-v2`, with the old repo at `/home/sumit/stride`:

```bash
mkdir -p fixtures/{fit,golden/golden,golden/scenarios,stride-core/plan_sources,stride-garmin,stride-api}

# FIT samples and golden outputs — copied byte-for-byte, already synthetic
cp /home/sumit/stride/backend/tests/golden/fixtures/*.fit   fixtures/fit/
cp /home/sumit/stride/backend/tests/golden/cases.json        fixtures/golden/
cp /home/sumit/stride/backend/tests/golden/scenarios/*.json  fixtures/golden/scenarios/
cp /home/sumit/stride/backend/tests/golden/golden/*.json     fixtures/golden/golden/

# Rust-core fixtures
cp /home/sumit/stride/crates/stride-core/tests/fixtures/plan_sources/*.json fixtures/stride-core/plan_sources/
cp /home/sumit/stride/crates/stride-api/tests/fixtures/*.json              fixtures/stride-api/

# Garmin fixtures — sanitise the two that carry a name and a user id, by field
for f in /home/sumit/stride/crates/stride-garmin/tests/fixtures/*.json; do
  case "$(basename "$f")" in
    profile.json|expected.json)
      sed -E \
        -e 's/"(displayName|fullName|nickName|athlete_name)": "[^"]*"/"\1": "ATHLETE"/g' \
        -e 's/"(userName)": "[^"]*"/"\1": "athlete"/g' \
        -e 's/"(id|userProfileId)": [0-9]+/"\1": 0/g' \
        "$f" > "fixtures/stride-garmin/$(basename "$f")"
      ;;
    *) cp "$f" fixtures/stride-garmin/ ;;
  esac
done
```

Then verify: `bash scripts/scan-secrets.sh && bash scripts/scan-privacy.sh`
must both pass.

## The FIT cases

Fifteen files, each shaped by what a *port* gets wrong (or by what a real watch
writes). Their names and purposes, condensed from the old repo's golden README:

| case | why it exists |
|---|---|
| `mixed_run` | walk warm-up + run + 20 s stand + run; stored cadence halved — the segmentation case |
| `hr_gps_run` | plain outdoor run with HR, GPS, six laps — the full metric set |
| `treadmill` | no GPS at all — a port must not assume a position stream |
| `no_cadence` | no cadence sensor — cadence metrics must be `null`, never `0` |
| `short_effort` | one lap — the thirds are empty, drift metrics must be `null`, not invented |
| `cycling` | a non-running sport whose cadence is **not** halved (90 stays 90) |
| `low_resolution` | five-second sampling — intervals are five seconds wide |
| `hilly` | climbs 60 m then descends 60 m |
| `gaps` | a paused watch — two 100-second holes; a hole is charged to the sample before it |
| `real_garmin_run` | a real watch's byte format: protocol 1.0, `enhanced_speed`/`enhanced_altitude`, fields the published FIT profile doesn't describe |
| `real_garmin_gap` | the same format with the 204-second hole real files contain |
| `real_garmin_nulls` | the reserved "no value" sentinels in the fields the importer reads |
| `vendor_strava_export` | Strava re-encoded it: manufacturer 265, no `device_info` |
| `vendor_coros_export` | COROS wrote it: manufacturer 294, product name `COROS DURA` |
| `vendor_wahoo_export` | Wahoo wrote it: manufacturer 32, no product name — named by manufacturer alone |

The three `vendor_*` ids sort after the other twelve on purpose: the API golden
numbers its recorded rows in seed order, and a case that sorted first would
renumber every recorded activity.
