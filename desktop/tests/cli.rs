//! The command line a stranger uses.

use std::path::PathBuf;
use std::process::Command;

fn scratch(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("stride-cli-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[test]
fn export_and_import_work_from_the_command_line() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("export-import")?;
    let export = directory.join("export");

    let written = Command::new(env!("CARGO_BIN_EXE_stride-desktop"))
        .arg("export")
        .arg("--data-dir")
        .arg(&directory)
        .arg("--output")
        .arg(&export)
        .output()?;
    assert!(written.status.success(), "{written:?}");
    let output = String::from_utf8(written.stdout)?;
    assert!(output.contains("the export is at"), "{output}");
    assert!(
        output.contains("the Garmin key is not in the export"),
        "{output}"
    );
    assert!(export.join("manifest.json").is_file());
    assert!(export.join("stride.db").is_file());
    assert!(export.join("db.key").is_file());

    let imported = Command::new(env!("CARGO_BIN_EXE_stride-desktop"))
        .arg("import")
        .arg("--data-dir")
        .arg(&directory)
        .arg(&export)
        .output()?;
    assert!(imported.status.success(), "{imported:?}");
    let output = String::from_utf8(imported.stdout)?;
    assert!(output.contains("read 0 rows"), "{output}");
    assert!(output.contains("format version 1"), "{output}");
    assert!(output.contains("refused: nothing"), "{output}");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}
