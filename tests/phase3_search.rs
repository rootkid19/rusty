use std::fs;
use std::process::Command;

use anyhow::Result;
use tempfile::tempdir;

fn rusty() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty"))
}

#[test]
fn search_finds_notes_by_body_title_and_tag() -> Result<()> {
    let temp = tempdir()?;

    let first = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("legacy admin bypass")
        .arg("--title")
        .arg("auth notes")
        .arg("--tag")
        .arg("security")
        .output()?;
    assert!(first.status.success(), "{first:?}");

    let second = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("unrelated note")
        .arg("--tag")
        .arg("docs")
        .output()?;
    assert!(second.status.success(), "{second:?}");

    let search = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("search")
        .arg("auth")
        .arg("--json")
        .output()?;
    assert!(search.status.success(), "{search:?}");
    let stdout = String::from_utf8(search.stdout)?;
    assert!(stdout.contains("\"title\":\"auth notes\""));
    assert!(stdout.contains("\"match_kind\":\"title\""));

    let tagged = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("search")
        .arg("security")
        .arg("--tag")
        .arg("security")
        .arg("--json")
        .output()?;
    assert!(tagged.status.success(), "{tagged:?}");
    let stdout = String::from_utf8(tagged.stdout)?;
    assert!(stdout.contains("\"title\":\"auth notes\""));
    assert!(!stdout.contains("unrelated note"));
    Ok(())
}

#[test]
fn search_human_output_uses_body_preview_and_match_reason() -> Result<()> {
    let temp = tempdir()?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("--title")
        .arg("auth notes")
        .arg("we found a bypass in the legacy admin flow")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let search = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("search")
        .arg("auth")
        .output()?;
    assert!(search.status.success(), "{search:?}");

    let stdout = String::from_utf8(search.stdout)?;
    assert!(stdout.contains("auth notes"));
    assert!(stdout.contains("we found a bypass in the legacy admin flow"));
    assert!(stdout.contains("match: title"));
    Ok(())
}

#[test]
fn search_rebuilds_cache_when_deleted() -> Result<()> {
    let temp = tempdir()?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("cache rebuild test")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let first = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("search")
        .arg("rebuild")
        .arg("--json")
        .output()?;
    assert!(first.status.success(), "{first:?}");

    let cache_path = temp.path().join("cache/index.json");
    assert!(cache_path.exists());
    fs::remove_file(&cache_path)?;

    let second = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("search")
        .arg("rebuild")
        .arg("--json")
        .output()?;
    assert!(second.status.success(), "{second:?}");
    assert!(cache_path.exists());
    assert_eq!(
        String::from_utf8(first.stdout)?,
        String::from_utf8(second.stdout)?
    );
    Ok(())
}
