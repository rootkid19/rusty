use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn resolve_editor(default_editor: &str) -> Result<String> {
    if let Ok(editor) = env::var("EDITOR") {
        let editor = editor.trim();
        if !editor.is_empty() {
            return Ok(editor.to_string());
        }
    }

    let fallback = default_editor.trim();
    if fallback.is_empty() {
        bail!("$EDITOR is unset and no default editor is configured. Set $EDITOR or config.default_editor.");
    }

    Ok(fallback.to_string())
}

pub fn edit_note_in_editor(editor: &str, path: &Path) -> Result<()> {
    let status = Command::new("sh")
        .arg("-lc")
        .arg("eval \"exec $RUSTY_EDITOR_CMD \\\"\\$@\\\"\"")
        .env("RUSTY_EDITOR_CMD", editor)
        .arg("rusty-editor")
        .arg(path)
        .status()
        .with_context(|| format!("failed to launch editor `{editor}` for {}", path.display()))?;

    if !status.success() {
        bail!(
            "editor `{editor}` exited with status {status} for {}",
            path.display()
        );
    }

    Ok(())
}
