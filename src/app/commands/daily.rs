use anyhow::Result;
use time::OffsetDateTime;

use crate::app::commands::created_note_line;
use crate::app::commands::new::open_and_refresh;
use crate::core::NoteStore;
use crate::util::time as time_util;

#[derive(Debug, Clone)]
pub struct DailyOptions {
    pub edit: bool,
    pub print_path: bool,
    pub now: OffsetDateTime,
}

pub fn run(store: &NoteStore, options: DailyOptions) -> Result<String> {
    let note = store.create_daily(time_util::local_date(options.now))?;

    if options.edit {
        open_and_refresh(store, store.config(), &note.path)?;
    }

    if options.print_path {
        return Ok(note.path.display().to_string());
    }

    Ok(created_note_line(&note))
}
