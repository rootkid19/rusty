use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use time::{Date, OffsetDateTime};
use walkdir::WalkDir;

use crate::core::config::Config;
use crate::core::metadata::{NoteId, NoteKind, NoteMeta, NoteStatus};
use crate::core::note::Note;
use crate::util::{slug, text, time as util_time};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateNoteInput {
    pub title: Option<String>,
    pub body: String,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub status: NoteStatus,
    pub kind: NoteKind,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDailyInput {
    pub date: Date,
    pub body: String,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub created_at: OffsetDateTime,
}

impl Default for CreateNoteInput {
    fn default() -> Self {
        Self {
            title: None,
            body: String::new(),
            tags: Vec::new(),
            source: None,
            status: NoteStatus::Active,
            kind: NoteKind::Note,
            created_at: util_time::now_utc(),
        }
    }
}

impl Default for CreateDailyInput {
    fn default() -> Self {
        let now = util_time::now_utc();
        Self {
            date: now.date(),
            body: String::new(),
            tags: Vec::new(),
            source: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoteStore {
    config: Config,
}

impl NoteStore {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn ensure_layout(&self) -> Result<()> {
        self.config.ensure_layout()
    }

    pub fn create_note(&self, input: CreateNoteInput) -> Result<Note> {
        self.ensure_layout()?;

        let created_at = input.created_at;
        let title = input.title.or_else(|| text::infer_title(&input.body));
        let mut meta = NoteMeta::new(
            NoteId::generate(created_at),
            created_at,
            input.kind,
            input.status,
        );
        meta.title = title.clone();
        meta.tags = input.tags;
        meta.source = input.source;

        let note = Note {
            path: self.note_path_for(&meta, title.as_deref(), created_at.date()),
            meta,
            body: normalize_body(&input.body),
        };
        self.save_note(&note)?;
        Ok(note)
    }

    pub fn create_daily(&self, date: Date) -> Result<Note> {
        self.create_daily_with_input(CreateDailyInput {
            date,
            created_at: util_time::start_of_day_local(date)?,
            ..CreateDailyInput::default()
        })
    }

    pub fn create_daily_with_input(&self, input: CreateDailyInput) -> Result<Note> {
        self.ensure_layout()?;
        let path = self.daily_path(input.date);

        if path.exists() {
            if !input.body.trim().is_empty() || !input.tags.is_empty() || input.source.is_some() {
                bail!(
                    "daily note already exists at {}. Appending capture into an existing daily note is deferred in M1.",
                    path.display()
                );
            }
            return self.read_note(&path);
        }

        let mut meta = NoteMeta::new(
            NoteId::generate(input.created_at),
            input.created_at,
            NoteKind::Daily,
            self.config.default_status,
        );
        meta.title = Some(input.date.to_string());
        meta.tags = input.tags;
        meta.source = input.source;

        let note = Note {
            path,
            meta,
            body: normalize_body(&input.body),
        };
        self.save_note(&note)?;
        Ok(note)
    }

    pub fn read_note(&self, path: impl AsRef<Path>) -> Result<Note> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read note at {}", path.display()))?;
        Note::from_markdown(path.to_path_buf(), &contents)
    }

    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        for root in [&self.config.notes_dir, &self.config.daily_dir] {
            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }
                notes.push(self.read_note(entry.path())?);
            }
        }

        notes.sort_by(|left, right| right.meta.updated_at.cmp(&left.meta.updated_at));
        Ok(notes)
    }

    pub fn resolve_ref(&self, reference: &str) -> Result<Note> {
        let path = Path::new(reference);
        if path.exists() {
            return self.read_note(path);
        }

        self.list_notes()?
            .into_iter()
            .find(|note| note.meta.id.as_str() == reference)
            .ok_or_else(|| anyhow!("note not found for ref `{reference}`"))
    }

    pub fn save_note(&self, note: &Note) -> Result<()> {
        if let Some(parent) = note.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let markdown = note.to_markdown()?;
        fs::write(&note.path, markdown)
            .with_context(|| format!("failed to write note at {}", note.path.display()))?;
        Ok(())
    }

    pub fn refresh_updated_at(&self, reference: &str, now: OffsetDateTime) -> Result<Note> {
        let mut note = self.resolve_ref(reference)?;
        note.meta.updated_at = now;
        self.save_note(&note)?;
        Ok(note)
    }

    pub fn note_path_for(&self, meta: &NoteMeta, title: Option<&str>, date: Date) -> PathBuf {
        match meta.kind {
            NoteKind::Note => {
                let year = date.year();
                let month = util_time::month_to_number(date.month());
                let slug = slug::slugify(title.unwrap_or("note"));
                self.config
                    .notes_dir
                    .join(format!("{year:04}"))
                    .join(format!("{month:02}"))
                    .join(format!("{slug}--{}.md", meta.id))
            }
            NoteKind::Daily => self.daily_path(date),
        }
    }

    fn daily_path(&self, date: Date) -> PathBuf {
        self.config.daily_dir.join(format!("{date}.md"))
    }
}

fn normalize_body(body: &str) -> String {
    if body.is_empty() || body.ends_with('\n') {
        body.to_string()
    } else {
        format!("{body}\n")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::tempdir;
    use time::macros::{date, datetime};

    use super::{CreateDailyInput, CreateNoteInput, NoteStore};
    use crate::core::config::Config;
    use crate::core::metadata::{NoteId, NoteKind, NoteMeta, NoteStatus};

    #[test]
    fn note_path_generation_matches_spec_shape() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);
        let meta = NoteMeta::new(
            NoteId::new("20260321T235900Z-8f3a")?,
            datetime!(2026-03-21 23:59:00 UTC),
            NoteKind::Note,
            NoteStatus::Active,
        );

        let path = store.note_path_for(&meta, Some("auth bypass notes"), date!(2026 - 03 - 21));

        assert_eq!(
            path,
            temp.path()
                .join("notes")
                .join("2026")
                .join("03")
                .join("auth-bypass-notes--20260321T235900Z-8f3a.md")
        );
        Ok(())
    }

    #[test]
    fn create_and_read_note_roundtrip() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);
        let created = store.create_note(CreateNoteInput {
            title: None,
            body: "Investigate auth bypass in legacy admin flow".to_string(),
            tags: vec!["security".to_string(), "auth".to_string()],
            source: Some("cli".to_string()),
            status: NoteStatus::Active,
            kind: NoteKind::Note,
            created_at: datetime!(2026-03-21 23:59:00 UTC),
        })?;

        let loaded = store.read_note(&created.path)?;

        assert_eq!(loaded.meta.id, created.meta.id);
        assert_eq!(
            loaded.meta.title.as_deref(),
            Some("Investigate auth bypass in legacy admin flow")
        );
        assert_eq!(loaded.meta.tags, created.meta.tags);
        assert_eq!(
            loaded.body,
            "Investigate auth bypass in legacy admin flow\n"
        );
        Ok(())
    }

    #[test]
    fn create_daily_is_deterministic() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);

        let first = store.create_daily(date!(2026 - 03 - 22))?;
        let second = store.create_daily(date!(2026 - 03 - 22))?;

        assert_eq!(first.path, second.path);
        assert_eq!(first.meta.kind, NoteKind::Daily);
        Ok(())
    }

    #[test]
    fn daily_capture_rejects_existing_note_append() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);

        store.create_daily(date!(2026 - 03 - 22))?;
        let result = store.create_daily_with_input(CreateDailyInput {
            date: date!(2026 - 03 - 22),
            body: "captured text".to_string(),
            tags: vec!["tag".to_string()],
            source: Some("cli".to_string()),
            created_at: datetime!(2026-03-22 12:00:00 UTC),
        });

        assert!(result.is_err());
        Ok(())
    }
}
