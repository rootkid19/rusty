# Rusty
Rusty is a local-first CLI and TUI for capturing notes quickly and finding them again quickly. Good O'l Rusty stores plain markdown files and uses `$EDITOR` for real editing.

## Installation
Install from GitHub:
```bash
cargo install --git https://github.com/rootkid19/rusty
```

Install from a local checkout:
```bash
cargo install --path .
```

Rusty installs a `rusty` binary and stores notes under `~/.rusty` by default.

## Daily Use
Save a quick note:
```bash
rusty quick "auth bypass notes"
```

Save text coming from another command:
```bash
pbpaste | rusty quick --stdin
printf 'we found a bypass in the legacy admin flow\n' | rusty quick --stdin --title "legacy admin auth" --tag security
```

Create a blank note:
```bash
rusty new --title "ops checklist"
```

Open or create today’s daily note:
```bash
rusty daily
rusty daily --edit
```

Recall notes:
```bash
rusty recent
rusty search auth
rusty search auth --tag security
rusty show <NOTE_ID>
```

Edit a note in your editor:
```bash
rusty edit <NOTE_ID>
```

Browse in the terminal UI:
```bash
rusty tui
```

Inside the TUI:
- `j` / `k`: move
- `e`: open selected note in `$EDITOR`
- `/`: search
- `r`: recent
- `d`: jump to today’s daily note
- `?`: help
- `q`: quit

Inspect paths or rebuild the cache:
```bash
rusty doctor --paths
rusty doctor --reindex
```

## TL;DR: 
- Use `quick` when the note is already in your head.
- Use `quick --stdin` when the note text is coming from somewhere else.
- Use `recent` when you know the note was touched recently.
- Use `search` when you remember words, tags, or subject matter.
- Use `show` to preview and confirm before opening an editor.
- Use `daily` as today’s anchor note, not as a separate notes system.

## Storage
- Default root: `~/.rusty`
- Notes: `~/.rusty/notes`
- Daily notes: `~/.rusty/daily`
- Cache: `~/.rusty/cache`

No worries, files stay readable without Rusty.
