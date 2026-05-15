use anyhow::Result;
use serde::Serialize;
use time::OffsetDateTime;

use crate::app::commands::{
    human_path, render_meta_line, render_result_card, short_id, title_or_untitled,
};
use crate::core::{NoteKind, NoteStatus, NoteStore};

#[derive(Debug, Clone)]
pub struct RecentOptions {
    pub limit: usize,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RecentItem {
    id: String,
    title: Option<String>,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    updated_at: OffsetDateTime,
    tags: Vec<String>,
    path: std::path::PathBuf,
    kind: NoteKind,
    status: NoteStatus,
}

pub fn run(store: &NoteStore, options: RecentOptions) -> Result<String> {
    let items = store
        .list_notes()?
        .into_iter()
        .take(options.limit)
        .map(|note| RecentItem {
            id: note.meta.id.to_string(),
            title: note.meta.title,
            updated_at: note.meta.updated_at,
            tags: note.meta.tags,
            path: note.path,
            kind: note.meta.kind,
            status: note.meta.status,
        })
        .collect::<Vec<_>>();

    if options.json {
        return Ok(serde_json::to_string(&items)?);
    }

    if items.is_empty() {
        return Ok("no recent notes".to_string());
    }

    Ok(items
        .into_iter()
        .map(|item| {
            render_result_card(
                title_or_untitled(item.title.as_deref()),
                &render_meta_line(&item.tags, item.status, item.kind, item.updated_at),
                &[format!(
                    "{}   {}",
                    short_id(&item.id),
                    human_path(store.config().root_dir.as_path(), &item.path)
                )],
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n"))
}
