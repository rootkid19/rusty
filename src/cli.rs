use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::app::commands;
use crate::core::{Config, NoteStore};
use crate::util::time;

#[derive(Debug, Parser)]
#[command(
    name = "rusty",
    version,
    about = "Local-first note capture and recall tool.",
    long_about = "Rusty helps you save a thought quickly and find it again quickly.\n\nIt stores plain markdown files, uses small CLI commands, and hands real editing off to $EDITOR."
)]
struct Cli {
    #[arg(long, global = true, hide = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Save a quick note from inline text or text piped in from another command")]
    Quick(QuickCommand),
    #[command(about = "Create a blank note, with optional editor handoff")]
    New(NewCommand),
    #[command(about = "Open or create today's daily note")]
    Daily(DailyCommand),
    #[command(about = "List recently updated notes")]
    Recent(RecentCommand),
    #[command(about = "Preview a note without opening an editor")]
    Show(ShowCommand),
    #[command(about = "Open a note in $EDITOR")]
    Edit(EditCommand),
    #[command(about = "Search note titles, tags, and bodies")]
    Search(SearchCommand),
    #[command(about = "Browse notes in a terminal UI")]
    Tui,
    #[command(about = "Inspect paths and rebuild the note index")]
    Doctor(DoctorCommand),
}

#[derive(Debug, Args)]
struct QuickCommand {
    #[arg(value_name = "text")]
    text: Vec<String>,
    #[arg(long, help = "Set an explicit title instead of inferring one")]
    title: Option<String>,
    #[arg(long = "tag", help = "Add a tag to the note")]
    tags: Vec<String>,
    #[arg(
        long,
        help = "Save the text as today's daily note if it does not already exist"
    )]
    daily: bool,
    #[arg(
        long,
        help = "Read note text from another command, for example: pbpaste | rusty quick --stdin"
    )]
    stdin: bool,
}

#[derive(Debug, Args)]
struct NewCommand {
    #[arg(long, help = "Set the note title")]
    title: Option<String>,
    #[arg(long = "tag", help = "Add a tag to the note")]
    tags: Vec<String>,
    #[arg(long, help = "Open the new note in $EDITOR right away")]
    edit: bool,
}

#[derive(Debug, Args)]
struct DailyCommand {
    #[arg(long, help = "Open today's daily note in $EDITOR")]
    edit: bool,
    #[arg(long, help = "Print only the daily note path")]
    print_path: bool,
}

#[derive(Debug, Args)]
struct RecentCommand {
    #[arg(long, default_value_t = 20, help = "Maximum number of notes to show")]
    limit: usize,
    #[arg(long, help = "Print machine-readable JSON")]
    json: bool,
}

#[derive(Debug, Args)]
struct ShowCommand {
    reference: String,
}

#[derive(Debug, Args)]
struct EditCommand {
    reference: String,
}

#[derive(Debug, Args)]
struct SearchCommand {
    #[arg(value_name = "query")]
    query: Vec<String>,
    #[arg(long = "tag", help = "Require one or more tags")]
    tags: Vec<String>,
    #[arg(long, help = "Filter by kind: note or daily")]
    kind: Option<String>,
    #[arg(long, help = "Filter by status: active or archived")]
    status: Option<String>,
    #[arg(long, default_value_t = 20, help = "Maximum number of results to show")]
    limit: usize,
    #[arg(long, help = "Print machine-readable JSON")]
    json: bool,
}

#[derive(Debug, Args)]
struct DoctorCommand {
    #[arg(long, help = "Print resolved filesystem paths")]
    paths: bool,
    #[arg(long, help = "Rebuild the note index from disk")]
    reindex: bool,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = match cli.root {
        Some(root) => Config::for_root(root)?,
        None => Config::load()?,
    };
    let store = NoteStore::new(config);
    let now = time::now_utc();

    match cli.command {
        Commands::Quick(cmd) => {
            let text = if cmd.text.is_empty() {
                None
            } else {
                Some(cmd.text.join(" "))
            };
            let output = commands::quick::run(
                &store,
                commands::quick::QuickOptions {
                    text,
                    title: cmd.title,
                    tags: cmd.tags,
                    daily: cmd.daily,
                    stdin: cmd.stdin,
                    now,
                },
            )?;
            emit_created_note_output(&store, &output, if cmd.daily { "daily" } else { "saved" })?;
        }
        Commands::New(cmd) => {
            let output = commands::new::run(
                &store,
                commands::new::NewOptions {
                    title: cmd.title,
                    tags: cmd.tags,
                    edit: cmd.edit,
                    now,
                },
            )?;
            emit_created_note_output(&store, &output, "saved")?;
        }
        Commands::Daily(cmd) => {
            let output = commands::daily::run(
                &store,
                commands::daily::DailyOptions {
                    edit: cmd.edit,
                    print_path: cmd.print_path,
                    now,
                },
            )?;
            if cmd.print_path {
                println!("{output}");
            } else {
                emit_created_note_output(&store, &output, "daily")?;
            }
        }
        Commands::Recent(cmd) => {
            let output = commands::recent::run(
                &store,
                commands::recent::RecentOptions {
                    limit: cmd.limit,
                    json: cmd.json,
                },
            )?;
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Commands::Show(cmd) => {
            println!("{}", commands::show::run(&store, &cmd.reference)?);
        }
        Commands::Edit(cmd) => {
            commands::edit::run(&store, &cmd.reference)?;
        }
        Commands::Search(cmd) => {
            let query = cmd.query.join(" ");
            let output = commands::search::run(
                &store,
                commands::search::SearchOptions {
                    query,
                    tags: cmd.tags,
                    kind: cmd
                        .kind
                        .map(|value| crate::core::NoteKind::from_str(&value))
                        .transpose()?,
                    status: cmd
                        .status
                        .map(|value| crate::core::NoteStatus::from_str(&value))
                        .transpose()?,
                    limit: cmd.limit,
                    json: cmd.json,
                },
            )?;
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Commands::Tui => commands::tui::run(&store)?,
        Commands::Doctor(cmd) => {
            let output = commands::doctor::run(
                &store,
                commands::doctor::DoctorOptions {
                    paths: cmd.paths,
                    reindex: cmd.reindex,
                },
            )?;
            if !output.is_empty() {
                println!("{output}");
            }
        }
    }

    Ok(())
}

fn emit_created_note_output(store: &NoteStore, output: &str, action: &str) -> Result<()> {
    if io::stdout().is_terminal() {
        if let Some((_, path)) = commands::parse_created_note_line(output) {
            let note = store.read_note(path)?;
            println!(
                "{}",
                commands::render_capture_confirmation(
                    action,
                    &note,
                    store.config().root_dir.as_path()
                )
            );
            return Ok(());
        }
    }

    println!("{output}");
    Ok(())
}
