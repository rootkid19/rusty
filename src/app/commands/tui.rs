use anyhow::Result;

use crate::core::NoteStore;

pub fn run(store: &NoteStore) -> Result<()> {
    crate::tui::app::run(store)
}
