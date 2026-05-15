use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use walkdir::WalkDir;

use crate::core::store::NoteStore;
use crate::core::{Config, Note, NoteId, NoteKind, NoteStatus};
use crate::util::text;

const INDEX_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStamp {
    pub seconds: i128,
    pub nanos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedNote {
    pub path: PathBuf,
    pub id: NoteId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub kind: NoteKind,
    pub status: NoteStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub preview: String,
    pub modified: FileStamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteIndex {
    pub version: u32,
    pub entries: Vec<IndexedNote>,
}

impl Default for NoteIndex {
    fn default() -> Self {
        Self {
            version: INDEX_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoteIndexStore {
    cache_path: PathBuf,
}

impl NoteIndexStore {
    pub fn new(config: &Config) -> Self {
        Self {
            cache_path: config.cache_dir.join("index.json"),
        }
    }

    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    pub fn load_best_effort(&self) -> Option<NoteIndex> {
        let contents = fs::read_to_string(&self.cache_path).ok()?;
        let index = serde_json::from_str::<NoteIndex>(&contents).ok()?;
        if index.version == INDEX_VERSION {
            Some(index)
        } else {
            None
        }
    }

    pub fn refresh(&self, store: &NoteStore) -> Result<NoteIndex> {
        store.ensure_layout()?;
        let cached = self
            .load_best_effort()
            .unwrap_or_default()
            .entries
            .into_iter()
            .map(|entry| (entry.path.clone(), entry))
            .collect::<HashMap<_, _>>();

        let mut entries = Vec::new();
        for path in discover_note_paths(store.config())? {
            let modified = file_stamp(&path)?;
            if let Some(entry) = cached.get(&path) {
                if entry.modified == modified {
                    entries.push(entry.clone());
                    continue;
                }
            }

            let note = store.read_note(&path)?;
            entries.push(indexed_note_from_note(note, modified));
        }

        entries.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        let index = NoteIndex {
            version: INDEX_VERSION,
            entries,
        };
        self.save(&index)?;
        Ok(index)
    }

    fn save(&self, index: &NoteIndex) -> Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let contents = serde_json::to_string_pretty(index)?;
        fs::write(&self.cache_path, contents)
            .with_context(|| format!("failed to write cache at {}", self.cache_path.display()))?;
        Ok(())
    }
}

fn indexed_note_from_note(note: Note, modified: FileStamp) -> IndexedNote {
    IndexedNote {
        path: note.path,
        id: note.meta.id,
        title: note.meta.title,
        created_at: note.meta.created_at,
        updated_at: note.meta.updated_at,
        tags: note.meta.tags,
        kind: note.meta.kind,
        status: note.meta.status,
        source: note.meta.source,
        preview: text::preview(&note.body, 120),
        modified,
    }
}

fn discover_note_paths(config: &Config) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for root in [&config.notes_dir, &config.daily_dir] {
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
            paths.push(entry.path().to_path_buf());
        }
    }

    paths.sort();
    Ok(paths)
}

fn file_stamp(path: &Path) -> Result<FileStamp> {
    let modified = fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?
        .modified()
        .with_context(|| format!("failed to read modified time for {}", path.display()))?;
    system_time_to_stamp(modified)
}

fn system_time_to_stamp(time: SystemTime) -> Result<FileStamp> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(FileStamp {
            seconds: duration.as_secs() as i128,
            nanos: duration.subsec_nanos(),
        }),
        Err(err) => {
            let duration = err.duration();
            Ok(FileStamp {
                seconds: -(duration.as_secs() as i128),
                nanos: duration.subsec_nanos(),
            })
        }
    }
}

impl PartialEq for FileStamp {
    fn eq(&self, other: &Self) -> bool {
        self.seconds == other.seconds && self.nanos == other.nanos
    }
}

impl Eq for FileStamp {}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::tempdir;
    use time::macros::datetime;

    use super::NoteIndexStore;
    use crate::core::store::{CreateNoteInput, NoteStore};
    use crate::core::{Config, NoteKind, NoteStatus};

    #[test]
    fn refresh_builds_cache_file() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);
        store.create_note(CreateNoteInput {
            title: Some("cached note".to_string()),
            body: "preview line".to_string(),
            tags: vec!["cache".to_string()],
            source: Some("cli".to_string()),
            status: NoteStatus::Active,
            kind: NoteKind::Note,
            created_at: datetime!(2026-03-25 18:10:00 UTC),
        })?;

        let index = NoteIndexStore::new(store.config()).refresh(&store)?;

        assert_eq!(index.entries.len(), 1);
        assert!(NoteIndexStore::new(store.config()).cache_path().exists());
        assert_eq!(index.entries[0].preview, "preview line");
        Ok(())
    }

    #[test]
    fn refresh_invalidates_modified_notes() -> Result<()> {
        let temp = tempdir()?;
        let store = NoteStore::new(Config::for_root(temp.path())?);
        let note = store.create_note(CreateNoteInput {
            title: Some("cached note".to_string()),
            body: "first body".to_string(),
            tags: Vec::new(),
            source: Some("cli".to_string()),
            status: NoteStatus::Active,
            kind: NoteKind::Note,
            created_at: datetime!(2026-03-25 18:11:00 UTC),
        })?;

        let cache = NoteIndexStore::new(store.config());
        cache.refresh(&store)?;

        let contents = fs::read_to_string(&note.path)?;
        let updated = contents.replace("first body", "updated body line");
        fs::write(&note.path, updated)?;

        let refreshed = cache.refresh(&store)?;
        assert_eq!(refreshed.entries[0].preview, "updated body line");
        Ok(())
    }
}
