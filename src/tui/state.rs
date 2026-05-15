use std::path::PathBuf;

use anyhow::Result;
use ratatui::widgets::ListState;
use time::OffsetDateTime;

use crate::core::index::{IndexedNote, NoteIndexStore};
use crate::core::query::{search_documents, SearchDocument, SearchFilters};
use crate::core::{Note, NoteKind, NoteStatus, NoteStore};
use crate::util::time as time_util;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Recent,
    Search,
}

#[derive(Debug, Clone)]
pub struct NoteSummary {
    pub path: PathBuf,
    pub id: String,
    pub title: Option<String>,
    pub updated_at: OffsetDateTime,
    pub tags: Vec<String>,
    pub kind: NoteKind,
    pub status: NoteStatus,
    pub preview: String,
    pub match_reason: Option<String>,
}

#[derive(Debug)]
pub struct TuiState {
    pub current_view: View,
    pub query: String,
    pub search_mode: bool,
    pub items: Vec<NoteSummary>,
    pub selected: usize,
    pub preview: Option<Note>,
    pub status: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub list_state: ListState,
}

impl TuiState {
    pub fn new(store: &NoteStore) -> Result<Self> {
        let mut state = Self {
            current_view: View::Recent,
            query: String::new(),
            search_mode: false,
            items: Vec::new(),
            selected: 0,
            preview: None,
            status: "recent".to_string(),
            show_help: false,
            should_quit: false,
            list_state: ListState::default(),
        };
        state.reload(store)?;
        Ok(state)
    }

    pub fn reload(&mut self, store: &NoteStore) -> Result<()> {
        let index = NoteIndexStore::new(store.config()).refresh(store)?;
        self.items = match self.current_view {
            View::Recent => index
                .entries
                .into_iter()
                .map(summary_from_indexed)
                .collect::<Vec<_>>(),
            View::Search => build_search_items(store, &self.query)?,
        };

        if self.items.is_empty() {
            self.selected = 0;
            self.preview = None;
            self.list_state.select(None);
            return Ok(());
        }

        if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
        self.list_state.select(Some(self.selected));
        self.preview = Some(store.read_note(&self.items[self.selected].path)?);
        Ok(())
    }

    pub fn select_next(&mut self, store: &NoteStore) -> Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        self.selected = (self.selected + 1).min(self.items.len() - 1);
        self.sync_preview(store)
    }

    pub fn select_prev(&mut self, store: &NoteStore) -> Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        self.selected = self.selected.saturating_sub(1);
        self.sync_preview(store)
    }

    pub fn set_view(&mut self, view: View, store: &NoteStore) -> Result<()> {
        self.current_view = view;
        self.search_mode = false;
        self.status = match view {
            View::Recent => "recent".to_string(),
            View::Search => format!("search: {}", self.query),
        };
        self.reload(store)
    }

    pub fn enter_search(&mut self) {
        self.search_mode = true;
        self.current_view = View::Search;
        self.status = "search: ".to_string();
    }

    pub fn push_query(&mut self, ch: char) {
        self.query.push(ch);
        self.status = format!("search: {}", self.query);
    }

    pub fn pop_query(&mut self) {
        self.query.pop();
        self.status = format!("search: {}", self.query);
    }

    pub fn submit_search(&mut self, store: &NoteStore) -> Result<()> {
        self.search_mode = false;
        self.set_view(View::Search, store)
    }

    pub fn sync_preview(&mut self, store: &NoteStore) -> Result<()> {
        if self.items.is_empty() {
            self.preview = None;
            self.list_state.select(None);
            return Ok(());
        }

        self.list_state.select(Some(self.selected));
        self.preview = Some(store.read_note(&self.items[self.selected].path)?);
        Ok(())
    }

    pub fn jump_to_today_daily(&mut self, store: &NoteStore) -> Result<()> {
        let daily = store.create_daily(time_util::today_local())?;
        self.current_view = View::Recent;
        self.search_mode = false;
        self.reload(store)?;
        if let Some(index) = self.items.iter().position(|item| item.path == daily.path) {
            self.selected = index;
            self.sync_preview(store)?;
        }
        self.status = "today's daily".to_string();
        Ok(())
    }
}

fn summary_from_indexed(item: IndexedNote) -> NoteSummary {
    NoteSummary {
        path: item.path,
        id: item.id.to_string(),
        title: item.title,
        updated_at: item.updated_at,
        tags: item.tags,
        kind: item.kind,
        status: item.status,
        preview: item.preview,
        match_reason: None,
    }
}

fn build_search_items(store: &NoteStore, query: &str) -> Result<Vec<NoteSummary>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let index = NoteIndexStore::new(store.config()).refresh(store)?;
    let docs = index
        .entries
        .into_iter()
        .map(|entry| {
            let note = store.read_note(&entry.path)?;
            Ok(SearchDocument {
                path: entry.path,
                id: entry.id,
                title: entry.title,
                updated_at: entry.updated_at,
                tags: entry.tags,
                kind: entry.kind,
                status: entry.status,
                preview: entry.preview,
                body: note.body,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let results = search_documents(trimmed, docs, &SearchFilters::default())?;
    Ok(results
        .into_iter()
        .map(|item| NoteSummary {
            path: item.path,
            id: item.id.to_string(),
            title: item.title,
            updated_at: item.updated_at,
            tags: item.tags,
            kind: item.kind,
            status: item.status,
            preview: item.snippet,
            match_reason: Some(format!("{:?}", item.match_kind).to_lowercase()),
        })
        .collect())
}
