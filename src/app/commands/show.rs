use anyhow::Result;

use crate::app::commands::{human_path, render_meta_line, short_id, title_or_untitled};
use crate::core::NoteStore;

pub fn run(store: &NoteStore, reference: &str) -> Result<String> {
    let note = store.resolve_ref(reference)?;
    let mut lines = vec![format!(
        "{}   {}",
        title_or_untitled(note.meta.title.as_deref()),
        render_meta_line(
            &note.meta.tags,
            note.meta.status,
            note.meta.kind,
            note.meta.updated_at,
        )
    )];
    lines.push(format!(
        "{}   {}",
        short_id(note.meta.id.as_str()),
        human_path(store.config().root_dir.as_path(), &note.path)
    ));

    lines.push("---".to_string());
    if note.body.trim().is_empty() {
        lines.push("(empty note)".to_string());
    } else {
        lines.push(note.body);
    }
    Ok(lines.join("\n"))
}
