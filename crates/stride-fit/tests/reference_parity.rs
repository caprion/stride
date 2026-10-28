//! Parity with the Python reference on real FIT files.
//!
//! The nine golden fixtures are synthetic: they use a narrow slice of the FIT
//! format. The files under `/tmp/realfit` come off real watches and use the
//! rest of it — compressed speed and distance records, event messages,
//! developer fields, bigger profiles.
//!
//! This test parses each of them with the Rust importer *and* with the
//! reference implementation (`backend/app/ingest/fit_importer.py`, driven
//! through the golden harness's own code so the two sides build the same
//! JSON) and compares the two.
//!
//! It is skipped when the real files or the backend virtualenv are not
//! there: the phone neither has the files nor runs Python.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use stride_fit::parse_fit;

use common::{compare, import_section, repo_root};

/// Where the real files live. Override with `STRIDE_REAL_FIT_DIR`.
fn real_fit_dir() -> PathBuf {
    std::env::var("STRIDE_REAL_FIT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/realfit"))
}

/// The Python interpreter that has the reference implementation.
fn reference_python() -> Option<PathBuf> {
    let candidate = repo_root().join("backend/.venv/bin/python");
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

/// The program that runs the reference importer over one file and prints the
/// import section as JSON.
///
/// It reuses `scripts/golden.py`, so the JSON it prints is built by exactly
/// the code that produced the golden files.
const REFERENCE_PROGRAM: &str = r#"
import io
import json
import os
import sys
import time
from pathlib import Path

repo = Path(sys.argv[1])
targets = [Path(argument) for argument in sys.argv[2:]]

os.environ["TZ"] = "UTC"
if hasattr(time, "tzset"):
    time.tzset()
os.environ.setdefault("STRIDE_DATA_DIR", "/tmp/stride-parity")
os.environ["STRIDE_KEYVAULT_ENABLED"] = "false"

sys.path.insert(0, str(repo / "backend"))
sys.path.insert(0, str(repo / "scripts"))

import golden

from app.ingest import fit_importer

# Only the importer is needed here, so the database layer is never imported:
# the reference then starts in a fraction of a second.
app = {"fit_importer": fit_importer}

from app.ingest.fit_importer import enum_name
import fitparse

results = {}
for target in targets:
    parsed = golden.parse_case("real", target, app)

    sport = None
    with target.open("rb") as handle:
        fit = fitparse.FitFile(io.BytesIO(handle.read()))
        for message in fit.get_messages("session"):
            field = message.get("sport")
            if field is not None:
                sport = enum_name(str(field.value))
                break

    results[target.name] = golden._round_floats(golden.import_section("real", parsed, sport))

print(json.dumps(results))
"#;

/// Run the reference importer over every file and return its import sections.
fn reference_imports(
    python: &Path,
    repo: &Path,
    files: &[PathBuf],
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut command = Command::new(python);
    command.arg("-c").arg(REFERENCE_PROGRAM).arg(repo);
    for file in files {
        command.arg(file);
    }
    let output = command
        .output()
        .map_err(|error| format!("could not run {}: {error}", python.display()))?;
    if !output.status.success() {
        return Err(format!(
            "the reference failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let parsed: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .map_err(|error| format!("the reference printed bad JSON: {error}"))?;
    match parsed {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err("the reference did not print an object".to_string()),
    }
}

/// Every `.fit` file in the directory, in name order.
fn real_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| {
                    path.extension().map(|ext| ext.eq_ignore_ascii_case("fit")).unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

#[test]
fn the_real_files_match_the_reference() {
    let dir = real_fit_dir();
    let Some(python) = reference_python() else {
        eprintln!("skipped: backend/.venv is not here");
        return;
    };
    if !dir.is_dir() {
        eprintln!("skipped: {} is not here", dir.display());
        return;
    }
    let files = real_files(&dir);
    if files.is_empty() {
        eprintln!("skipped: {} holds no .fit files", dir.display());
        return;
    }
    let repo = repo_root();

    let reference = reference_imports(&python, &repo, &files)
        .unwrap_or_else(|error| panic!("the reference importer failed: {error}"));

    for file in &files {
        let name = file
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();

        let expected = reference
            .get(&name)
            .unwrap_or_else(|| panic!("{name}: the reference produced no output for it"));

        let bytes = std::fs::read(file)
            .unwrap_or_else(|error| panic!("{name}: cannot read the file: {error}"));
        let activity = parse_fit(&bytes, &name)
            .unwrap_or_else(|error| panic!("{name}: the Rust importer failed: {error}"));

        compare(&format!("{name}.import"), expected, &import_section(&activity));
    }
}
