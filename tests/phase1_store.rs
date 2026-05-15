use anyhow::Result;
use tempfile::tempdir;
use time::macros::{date, datetime};

use rusty::core::{Config, CreateNoteInput, NoteKind, NoteStatus, NoteStore};

#[test]
fn list_notes_returns_recent_first() -> Result<()> {
    let temp = tempdir()?;
    let store = NoteStore::new(Config::for_root(temp.path())?);

    store.create_note(CreateNoteInput {
        title: Some("older".to_string()),
        body: "older body".to_string(),
        tags: Vec::new(),
        source: None,
        status: NoteStatus::Active,
        kind: NoteKind::Note,
        created_at: datetime!(2026-03-21 23:59:00 UTC),
    })?;

    store.create_note(CreateNoteInput {
        title: Some("newer".to_string()),
        body: "newer body".to_string(),
        tags: Vec::new(),
        source: None,
        status: NoteStatus::Active,
        kind: NoteKind::Note,
        created_at: datetime!(2026-03-22 12:00:00 UTC),
    })?;

    store.create_daily(date!(2026 - 03 - 22))?;

    let notes = store.list_notes()?;

    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].meta.title.as_deref(), Some("newer"));
    Ok(())
}
