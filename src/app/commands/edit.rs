use anyhow::Result;

use crate::app::commands::new::open_and_refresh;
use crate::core::NoteStore;

pub fn run(store: &NoteStore, reference: &str) -> Result<()> {
    let note = store.resolve_ref(reference)?;
    open_and_refresh(store, store.config(), &note.path)
}
