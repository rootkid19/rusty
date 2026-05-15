use std::fs;

use anyhow::Result;
use serde::Serialize;
use time::OffsetDateTime;

use crate::app::commands::{
    human_path, match_kind_label, render_meta_line, render_result_card, short_id, title_or_untitled,
};
use crate::core::index::NoteIndexStore;
use crate::core::query::{search_documents, MatchKind, SearchDocument, SearchFilters};
use crate::core::{Note, NoteKind, NoteStatus, NoteStore};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub tags: Vec<String>,
    pub kind: Option<NoteKind>,
    pub status: Option<NoteStatus>,
    pub limit: usize,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SearchItem {
    id: String,
    title: Option<String>,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    updated_at: OffsetDateTime,
    tags: Vec<String>,
    path: std::path::PathBuf,
    kind: NoteKind,
    status: NoteStatus,
    snippet: String,
    match_kind: MatchKind,
}

pub fn run(store: &NoteStore, options: SearchOptions) -> Result<String> {
    let index = NoteIndexStore::new(store.config()).refresh(store)?;
    let documents = index
        .entries
        .into_iter()
        .map(|entry| {
            let contents = fs::read_to_string(&entry.path)?;
            Ok(SearchDocument {
                path: entry.path,
                id: entry.id,
                title: entry.title,
                updated_at: entry.updated_at,
                tags: entry.tags,
                kind: entry.kind,
                status: entry.status,
                preview: entry.preview,
                body: Note::body_from_markdown(&contents)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let results = search_documents(
        &options.query,
        documents,
        &SearchFilters {
            tags: options.tags,
            kind: options.kind,
            status: options.status,
            limit: options.limit,
        },
    )?;

    let items = results
        .into_iter()
        .map(|result| SearchItem {
            id: result.id.to_string(),
            title: result.title,
            updated_at: result.updated_at,
            tags: result.tags,
            path: result.path,
            kind: result.kind,
            status: result.status,
            snippet: result.snippet,
            match_kind: result.match_kind,
        })
        .collect::<Vec<_>>();

    if options.json {
        return Ok(serde_json::to_string(&items)?);
    }

    if items.is_empty() {
        return Ok("no matching notes".to_string());
    }

    Ok(items
        .into_iter()
        .map(|item| {
            render_result_card(
                title_or_untitled(item.title.as_deref()),
                &render_meta_line(&item.tags, item.status, item.kind, item.updated_at),
                &[
                    item.snippet,
                    format!(
                        "match: {}   {}   {}",
                        match_kind_label(item.match_kind),
                        short_id(&item.id),
                        human_path(store.config().root_dir.as_path(), &item.path)
                    ),
                ],
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n"))
}
