use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::Serialize;
use time::OffsetDateTime;

use crate::core::{NoteId, NoteKind, NoteStatus};
use crate::util::text;

#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub tags: Vec<String>,
    pub kind: Option<NoteKind>,
    pub status: Option<NoteStatus>,
    pub limit: usize,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            kind: None,
            status: None,
            limit: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub path: PathBuf,
    pub id: NoteId,
    pub title: Option<String>,
    pub updated_at: OffsetDateTime,
    pub tags: Vec<String>,
    pub kind: NoteKind,
    pub status: NoteStatus,
    pub preview: String,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    Title,
    BodyExact,
    Tag,
    BodySubstring,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub id: NoteId,
    pub title: Option<String>,
    pub updated_at: OffsetDateTime,
    pub tags: Vec<String>,
    pub kind: NoteKind,
    pub status: NoteStatus,
    pub snippet: String,
    pub match_kind: MatchKind,
}

pub fn search_documents(
    query: &str,
    documents: impl IntoIterator<Item = SearchDocument>,
    filters: &SearchFilters,
) -> Result<Vec<SearchResult>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("search query cannot be empty");
    }

    let query_lower = query.to_ascii_lowercase();
    let mut results = documents
        .into_iter()
        .filter(|doc| matches_filters(doc, filters))
        .filter_map(|doc| search_document(doc, query, &query_lower))
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        priority(&right.match_kind)
            .cmp(&priority(&left.match_kind))
            .then_with(|| right.updated_at.cmp(&left.updated_at))
    });
    results.truncate(filters.limit);
    Ok(results)
}

fn matches_filters(doc: &SearchDocument, filters: &SearchFilters) -> bool {
    if let Some(kind) = filters.kind {
        if doc.kind != kind {
            return false;
        }
    }

    if let Some(status) = filters.status {
        if doc.status != status {
            return false;
        }
    }

    filters.tags.iter().all(|required| {
        let required = required.to_ascii_lowercase();
        doc.tags
            .iter()
            .any(|tag| tag.to_ascii_lowercase() == required)
    })
}

fn search_document(doc: SearchDocument, query: &str, query_lower: &str) -> Option<SearchResult> {
    let title = doc.title.clone().unwrap_or_default();
    let title_lower = title.to_ascii_lowercase();
    let body_lower = doc.body.to_ascii_lowercase();
    let tag_match = doc
        .tags
        .iter()
        .any(|tag| tag.to_ascii_lowercase().contains(query_lower));

    let title_match = title_lower.contains(query_lower);
    let body_match = body_lower.find(query_lower);

    let preferred_preview = preferred_preview(&doc);

    let (match_kind, snippet) = if title_match {
        (MatchKind::Title, preferred_preview.unwrap_or(title))
    } else if let Some(position) = body_match {
        let char_position = body_lower[..position].chars().count();
        let query_char_len = query.chars().count();
        if is_exact_body_hit(&body_lower, position, query_lower.len()) {
            (
                MatchKind::BodyExact,
                extract_snippet(&doc.body, char_position, query_char_len),
            )
        } else {
            (
                MatchKind::BodySubstring,
                extract_snippet(&doc.body, char_position, query_char_len),
            )
        }
    } else if tag_match {
        (
            MatchKind::Tag,
            preferred_preview.unwrap_or_else(|| format!("tags: {}", doc.tags.join(", "))),
        )
    } else {
        return None;
    };

    Some(SearchResult {
        path: doc.path,
        id: doc.id,
        title: doc.title,
        updated_at: doc.updated_at,
        tags: doc.tags,
        kind: doc.kind,
        status: doc.status,
        snippet,
        match_kind,
    })
}

fn preferred_preview(doc: &SearchDocument) -> Option<String> {
    let preview = doc.preview.trim();
    if preview.is_empty() {
        return None;
    }

    let title = doc.title.as_deref().unwrap_or_default().trim();
    if !title.is_empty() && preview.eq_ignore_ascii_case(title) {
        return None;
    }

    Some(preview.to_string())
}

fn is_exact_body_hit(body: &str, position: usize, query_len: usize) -> bool {
    let before = body[..position].chars().next_back();
    let after = body[position + query_len..].chars().next();
    is_boundary(before) && is_boundary(after)
}

fn is_boundary(ch: Option<char>) -> bool {
    ch.map(|value| !value.is_ascii_alphanumeric())
        .unwrap_or(true)
}

fn extract_snippet(body: &str, position: usize, query_len: usize) -> String {
    let chars = body.chars().collect::<Vec<_>>();
    let start = position.saturating_sub(30);
    let end = (position + query_len + 30).min(chars.len());
    let snippet = chars[start..end].iter().collect::<String>();
    let trimmed = snippet.trim();

    let mut rendered = String::new();
    if start > 0 {
        rendered.push_str("...");
    }
    rendered.push_str(&text::truncate(trimmed, 80));
    if end < chars.len() {
        rendered.push_str("...");
    }
    rendered
}

fn priority(kind: &MatchKind) -> u8 {
    match kind {
        MatchKind::Title => 4,
        MatchKind::BodyExact => 3,
        MatchKind::Tag => 2,
        MatchKind::BodySubstring => 1,
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use time::macros::datetime;

    use super::{search_documents, MatchKind, SearchDocument, SearchFilters};
    use crate::core::{NoteId, NoteKind, NoteStatus};

    #[test]
    fn title_hits_rank_above_body_hits() -> Result<()> {
        let results = search_documents(
            "auth",
            vec![
                SearchDocument {
                    path: "body.md".into(),
                    id: NoteId::new("body-id")?,
                    title: Some("legacy admin notes".to_string()),
                    updated_at: datetime!(2026-03-25 18:20:00 UTC),
                    tags: Vec::new(),
                    kind: NoteKind::Note,
                    status: NoteStatus::Active,
                    preview: "preview".to_string(),
                    body: "found auth issue in body".to_string(),
                },
                SearchDocument {
                    path: "title.md".into(),
                    id: NoteId::new("title-id")?,
                    title: Some("auth notes".to_string()),
                    updated_at: datetime!(2026-03-25 18:19:00 UTC),
                    tags: Vec::new(),
                    kind: NoteKind::Note,
                    status: NoteStatus::Active,
                    preview: "preview".to_string(),
                    body: "body".to_string(),
                },
            ],
            &SearchFilters::default(),
        )?;

        assert_eq!(results[0].match_kind, MatchKind::Title);
        Ok(())
    }

    #[test]
    fn exact_body_hits_rank_above_substrings() -> Result<()> {
        let results = search_documents(
            "auth",
            vec![
                SearchDocument {
                    path: "substring.md".into(),
                    id: NoteId::new("substring-id")?,
                    title: None,
                    updated_at: datetime!(2026-03-25 18:21:00 UTC),
                    tags: Vec::new(),
                    kind: NoteKind::Note,
                    status: NoteStatus::Active,
                    preview: "preview".to_string(),
                    body: "authentication details".to_string(),
                },
                SearchDocument {
                    path: "exact.md".into(),
                    id: NoteId::new("exact-id")?,
                    title: None,
                    updated_at: datetime!(2026-03-25 18:20:00 UTC),
                    tags: Vec::new(),
                    kind: NoteKind::Note,
                    status: NoteStatus::Active,
                    preview: "preview".to_string(),
                    body: "the auth flow failed".to_string(),
                },
            ],
            &SearchFilters::default(),
        )?;

        assert_eq!(results[0].match_kind, MatchKind::BodyExact);
        Ok(())
    }
}
