pub mod daily;
pub mod doctor;
pub mod edit;
pub mod new;
pub mod quick;
pub mod recent;
pub mod search;
pub mod show;
pub mod tui;

use std::path::Path;

use crate::core::query::MatchKind;
use crate::core::{Note, NoteKind, NoteStatus};
use crate::util::time;

pub fn created_note_line(note: &Note) -> String {
    format!("{}\t{}", note.meta.id, display_path(&note.path))
}

pub fn parse_created_note_line(value: &str) -> Option<(&str, &str)> {
    let trimmed = value.trim();
    trimmed.split_once('\t')
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub fn human_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub fn short_id(value: &str) -> &str {
    value.rsplit('-').next().unwrap_or(value)
}

pub fn title_or_untitled(title: Option<&str>) -> &str {
    title.unwrap_or("(untitled)")
}

pub fn render_meta_line(
    tags: &[String],
    status: NoteStatus,
    kind: NoteKind,
    updated_at: ::time::OffsetDateTime,
) -> String {
    let mut parts = Vec::new();
    if !tags.is_empty() {
        parts.push(format!("[{}]", tags.join(" ")));
    }
    if status != NoteStatus::Active {
        parts.push(status_label(status).to_string());
    }
    if kind == NoteKind::Daily {
        parts.push("daily".to_string());
    }
    parts.push(time::format_human_time(updated_at));
    parts.join("   ")
}

pub fn render_result_card(title: &str, meta_line: &str, detail_lines: &[String]) -> String {
    let mut lines = Vec::with_capacity(2 + detail_lines.len());
    lines.push(format!("{title}   {meta_line}"));
    lines.extend(detail_lines.iter().map(|line| format!("  {line}")));
    lines.join("\n")
}

pub fn render_capture_confirmation(action: &str, note: &Note, root: &Path) -> String {
    format!(
        "{action}  {}\n  {}  {}",
        title_or_untitled(note.meta.title.as_deref()),
        short_id(note.meta.id.as_str()),
        human_path(root, &note.path)
    )
}

pub fn match_kind_label(value: MatchKind) -> &'static str {
    match value {
        MatchKind::Title => "title",
        MatchKind::BodyExact => "body",
        MatchKind::Tag => "tag",
        MatchKind::BodySubstring => "body",
    }
}

pub fn status_label(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::Active => "active",
        NoteStatus::Archived => "archived",
    }
}
