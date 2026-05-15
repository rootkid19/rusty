use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::commands::new::open_and_refresh;
use crate::core::NoteStore;
use crate::tui::event::{is_quit_key, next_event, TuiEvent};
use crate::tui::render;
use crate::tui::state::{TuiState, View};

pub fn run(store: &NoteStore) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(&mut terminal, store);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    store: &NoteStore,
) -> Result<()> {
    let mut state = TuiState::new(store)?;

    loop {
        terminal
            .draw(|frame| render::draw(frame, &mut state, store.config().root_dir.as_path()))?;

        if state.should_quit {
            break;
        }

        match next_event(Duration::from_millis(200))? {
            TuiEvent::Tick => {}
            TuiEvent::Key(key) => {
                if state.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            state.search_mode = false;
                            state.status = "search cancelled".to_string();
                        }
                        KeyCode::Enter => state.submit_search(store)?,
                        KeyCode::Backspace => state.pop_query(),
                        KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            state.push_query(ch)
                        }
                        _ => {}
                    }
                    continue;
                }

                if is_quit_key(key.code, key.modifiers) {
                    state.should_quit = true;
                    continue;
                }

                match key.code {
                    KeyCode::Char('?') => state.show_help = !state.show_help,
                    KeyCode::Char('/') => state.enter_search(),
                    KeyCode::Char('r') => state.set_view(View::Recent, store)?,
                    KeyCode::Char('d') => state.jump_to_today_daily(store)?,
                    KeyCode::Char('j') | KeyCode::Down => state.select_next(store)?,
                    KeyCode::Char('k') | KeyCode::Up => state.select_prev(store)?,
                    KeyCode::Char('e') => {
                        if let Some(note) = &state.preview {
                            with_terminal_suspended(terminal, || {
                                open_and_refresh(store, store.config(), &note.path)
                            })?;
                            state.reload(store)?;
                            state.status = "returned from editor".to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn with_terminal_suspended<F>(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    action: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let action_result = action();

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    action_result
}
