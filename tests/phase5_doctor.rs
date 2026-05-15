use std::path::Path;
use std::process::Command;

use anyhow::Result;
use tempfile::tempdir;

fn rusty() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty"))
}

#[test]
fn doctor_paths_prints_resolved_locations() -> Result<()> {
    let temp = tempdir()?;

    let output = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .arg("--paths")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("root"));
    assert!(stdout.contains(&temp.path().display().to_string()));
    assert!(stdout.contains("notes"));
    assert!(stdout.contains("daily"));
    assert!(stdout.contains("index"));
    Ok(())
}

#[test]
fn doctor_reindex_rebuilds_cache_from_disk() -> Result<()> {
    let temp = tempdir()?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("doctor cache rebuild")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let cache_path = temp.path().join("cache/index.json");
    if cache_path.exists() {
        std::fs::remove_file(&cache_path)?;
    }

    let output = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .arg("--reindex")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    assert!(Path::new(&cache_path).exists());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("reindexed 1 notes"));
    Ok(())
}
