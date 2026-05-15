use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use anyhow::Result;
use tempfile::tempdir;

fn rusty() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty"))
}

#[test]
fn quick_creates_note_from_inline_text() -> Result<()> {
    let temp = tempdir()?;

    let output = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("capture this thought")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8(output.stdout)?;
    let mut fields = stdout.trim().split('\t');
    let id = fields.next().unwrap();
    let path = fields.next().unwrap();

    assert!(id.contains('-'));
    let contents = fs::read_to_string(path)?;
    assert!(contents.contains("capture this thought"));
    Ok(())
}

#[test]
fn recent_json_lists_created_notes() -> Result<()> {
    let temp = tempdir()?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("new")
        .arg("--title")
        .arg("first note")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let output = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("recent")
        .arg("--limit")
        .arg("1")
        .arg("--json")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("\"title\":\"first note\""));
    Ok(())
}

#[test]
fn daily_print_path_is_deterministic() -> Result<()> {
    let temp = tempdir()?;

    let first = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("daily")
        .arg("--print-path")
        .output()?;
    let second = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("daily")
        .arg("--print-path")
        .output()?;

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(
        String::from_utf8(first.stdout)?,
        String::from_utf8(second.stdout)?
    );
    Ok(())
}

#[test]
fn show_accepts_note_id_reference() -> Result<()> {
    let temp = tempdir()?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("quick")
        .arg("show me later")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let id = String::from_utf8(create.stdout)?
        .trim()
        .split('\t')
        .next()
        .unwrap()
        .to_string();

    let show = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("show")
        .arg(id)
        .output()?;

    assert!(show.status.success(), "{show:?}");
    let stdout = String::from_utf8(show.stdout)?;
    assert!(stdout.starts_with("show me later   today"));
    assert!(stdout.contains("---"));
    assert!(stdout.contains("show me later"));
    Ok(())
}

#[test]
fn edit_uses_editor_handoff() -> Result<()> {
    let temp = tempdir()?;
    let editor_script = temp.path().join("fake-editor.sh");
    let editor_log = temp.path().join("editor.log");

    fs::write(
        &editor_script,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > {}\n", editor_log.display()),
    )?;
    let mut perms = fs::metadata(&editor_script)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&editor_script, perms)?;

    let create = rusty()
        .arg("--root")
        .arg(temp.path())
        .arg("new")
        .arg("--title")
        .arg("editor note")
        .output()?;
    assert!(create.status.success(), "{create:?}");

    let id = String::from_utf8(create.stdout)?
        .trim()
        .split('\t')
        .next()
        .unwrap()
        .to_string();

    let edit = rusty()
        .env("EDITOR", &editor_script)
        .arg("--root")
        .arg(temp.path())
        .arg("edit")
        .arg(id)
        .output()?;

    assert!(edit.status.success(), "{edit:?}");
    let logged = fs::read_to_string(editor_log)?;
    assert!(logged.ends_with(".md"));
    Ok(())
}
