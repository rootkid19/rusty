use std::io::{self, IsTerminal, Read};

use anyhow::{bail, Result};
use time::OffsetDateTime;

use crate::app::commands::created_note_line;
use crate::core::{CreateDailyInput, CreateNoteInput, NoteStore};

#[derive(Debug, Clone)]
pub struct QuickOptions {
    pub text: Option<String>,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub daily: bool,
    pub stdin: bool,
    pub now: OffsetDateTime,
}

pub fn run(store: &NoteStore, options: QuickOptions) -> Result<String> {
    let body = capture_body(&options)?;

    if options.daily {
        let note = store.create_daily_with_input(CreateDailyInput {
            date: options.now.date(),
            body,
            tags: options.tags,
            source: Some("cli".to_string()),
            created_at: options.now,
        })?;
        return Ok(created_note_line(&note));
    }

    let note = store.create_note(CreateNoteInput {
        title: options.title,
        body,
        tags: options.tags,
        source: Some("cli".to_string()),
        created_at: options.now,
        ..CreateNoteInput::default()
    })?;

    Ok(created_note_line(&note))
}

fn capture_body(options: &QuickOptions) -> Result<String> {
    let piped_stdin = !io::stdin().is_terminal();

    match (&options.text, options.stdin || piped_stdin) {
        (Some(_), true) if options.stdin => {
            bail!("`rusty quick` accepts either inline text or `--stdin`, not both")
        }
        (Some(text), _) => Ok(text.clone()),
        (None, true) => read_stdin(),
        (None, false) => bail!("`rusty quick` needs text or stdin input"),
    }
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        bail!("stdin was empty")
    }
    Ok(input)
}
