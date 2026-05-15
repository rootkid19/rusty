use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::commands::{human_path, short_id, status_label, title_or_untitled};
use crate::tui::state::{NoteSummary, TuiState, View};
use crate::util::time;

pub fn draw(frame: &mut Frame<'_>, state: &mut TuiState, root: &std::path::Path) {
    let theme = Theme::default();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    let list = list_widget(state.current_view, &state.items, root, &theme);
    frame.render_stateful_widget(list, main[0], &mut state.list_state);
    frame.render_widget(preview_widget(state, root, &theme), main[1]);
    frame.render_widget(status_widget(state, &theme), chunks[1]);

    if state.show_help {
        let area = centered_rect(frame.area(), 70, 40);
        frame.render_widget(Clear, area);
        frame.render_widget(help_widget(&theme), area);
    }
}

fn list_widget<'a>(
    view: View,
    items: &'a [NoteSummary],
    root: &std::path::Path,
    theme: &Theme,
) -> List<'a> {
    let items = if items.is_empty() {
        vec![ListItem::new(vec![Line::from(Span::styled(
            empty_message(view),
            theme.text_muted,
        ))])]
    } else {
        items
            .iter()
            .map(|item| list_item(item, root, theme))
            .collect::<Vec<_>>()
    };

    List::new(items)
        .block(
            Block::default()
                .title(view_title(view))
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .highlight_style(theme.focus)
}

fn list_item(item: &NoteSummary, root: &std::path::Path, theme: &Theme) -> ListItem<'static> {
    let mut meta_parts = Vec::new();
    if !item.tags.is_empty() {
        meta_parts.push(format!("[{}]", item.tags.join(" ")));
    }
    if item.kind == crate::core::NoteKind::Daily {
        meta_parts.push("daily".to_string());
    } else if item.status != crate::core::NoteStatus::Active {
        meta_parts.push(status_label(item.status).to_string());
    }
    meta_parts.push(time::format_human_time(item.updated_at));
    let meta = meta_parts.join("   ");

    ListItem::new(vec![
        Line::from(Span::styled(
            title_or_untitled(item.title.as_deref()).to_string(),
            theme.text_primary,
        )),
        Line::from(vec![
            Span::styled(meta, theme.text_muted),
            Span::raw("   "),
            Span::styled(
                format!("{}   {}", short_id(&item.id), human_path(root, &item.path)),
                theme.text_subtle,
            ),
        ]),
    ])
}

fn preview_widget(state: &TuiState, root: &std::path::Path, theme: &Theme) -> Paragraph<'static> {
    let lines = match &state.preview {
        Some(note) => {
            let mut header = vec![Line::from(vec![
                Span::styled(
                    title_or_untitled(note.meta.title.as_deref()).to_string(),
                    theme.text_primary.add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(
                    {
                        let mut parts = Vec::new();
                        if !note.meta.tags.is_empty() {
                            parts.push(format!("[{}]", note.meta.tags.join(" ")));
                        }
                        if note.meta.status != crate::core::NoteStatus::Active {
                            parts.push(status_label(note.meta.status).to_string());
                        }
                        if note.meta.kind == crate::core::NoteKind::Daily {
                            parts.push("daily".to_string());
                        }
                        parts.join("   ")
                    },
                    theme.text_muted,
                ),
            ])];
            header.push(Line::from(Span::styled(
                format!(
                    "{}   {}   {}",
                    time::format_human_time(note.meta.updated_at),
                    short_id(note.meta.id.as_str()),
                    human_path(root, &note.path)
                ),
                theme.text_subtle,
            )));
            header.push(Line::from(Span::styled("─".repeat(40), theme.border)));

            if note.body.trim().is_empty() {
                header.push(Line::from(Span::styled("(empty note)", theme.text_muted)));
            } else {
                header.extend(note.body.lines().map(|line| Line::from(line.to_string())));
            }
            header
        }
        None => vec![Line::from(Span::styled(
            "Select a note to preview it.",
            theme.text_muted,
        ))],
    };

    Paragraph::new(lines)
        .block(
            Block::default()
                .title("Preview")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .wrap(Wrap { trim: false })
}

fn status_widget<'a>(state: &'a TuiState, theme: &Theme) -> Paragraph<'a> {
    let text = if state.search_mode {
        format!(
            "search: {} | [Enter] run | [Esc] cancel | [q] quit",
            state.query
        )
    } else {
        format!(
            "{} | [j/k] move | [e] edit | [/] search | [r] recent | [d] today | [?] help | [q] quit",
            state.status
        )
    };

    Paragraph::new(text).style(theme.text_muted)
}

fn help_widget<'a>(theme: &Theme) -> Paragraph<'a> {
    Paragraph::new(
        "Recent and Search are navigation views.\n\nd: jump to today's daily note\nj/k or arrows: move\ne: open in $EDITOR\n/: start search\nr: recent\n?: toggle help\nq: quit",
    )
    .block(
        Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(theme.border),
    )
    .style(theme.text_primary)
    .wrap(Wrap { trim: true })
}

fn view_title(view: View) -> &'static str {
    match view {
        View::Recent => "Recent",
        View::Search => "Search",
    }
}

fn empty_message(view: View) -> &'static str {
    match view {
        View::Recent => "No recent notes yet.",
        View::Search => "Type / to search and press Enter.",
    }
}

fn centered_rect(area: ratatui::layout::Rect, width: u16, height: u16) -> ratatui::layout::Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Percentage(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Percentage(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(popup[1])[1]
}

struct Theme {
    border: Style,
    text_primary: Style,
    text_muted: Style,
    text_subtle: Style,
    focus: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::Indexed(238)),
            text_primary: Style::default().fg(Color::Indexed(252)),
            text_muted: Style::default().fg(Color::Indexed(245)),
            text_subtle: Style::default().fg(Color::Indexed(240)),
            focus: Style::default()
                .fg(Color::Indexed(252))
                .bg(Color::Indexed(236))
                .add_modifier(Modifier::BOLD),
        }
    }
}
