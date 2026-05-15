use std::path::Path;

use ::time::OffsetDateTime as Timestamp;
use anyhow::Result;

use crate::app::commands::created_note_line;
use crate::core::editor::resolve_editor;
use crate::core::index::NoteIndexStore;
use crate::core::{edit_note_in_editor, Config, CreateNoteInput, NoteKind, NoteStatus, NoteStore};
use crate::util::time as time_util;

#[derive(Debug, Clone)]
pub struct NewOptions {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub edit: bool,
    pub now: Timestamp,
}

pub fn run(store: &NoteStore, options: NewOptions) -> Result<String> {
    let note = store.create_note(CreateNoteInput {
        title: options.title,
        tags: options.tags,
        source: Some("cli".to_string()),
        status: NoteStatus::Active,
        kind: NoteKind::Note,
        created_at: options.now,
        ..CreateNoteInput::default()
    })?;

    if options.edit {
        open_and_refresh(store, store.config(), &note.path)?;
    }

    Ok(created_note_line(&note))
}

pub fn open_and_refresh(store: &NoteStore, config: &Config, path: &Path) -> Result<()> {
    let editor = resolve_editor(&config.default_editor)?;
    edit_note_in_editor(&editor, path)?;
    store.refresh_updated_at(&path.display().to_string(), time_util::now_utc())?;
    NoteIndexStore::new(config).refresh(store)?;
    Ok(())
}
