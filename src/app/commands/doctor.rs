use anyhow::Result;

use crate::core::index::NoteIndexStore;
use crate::core::NoteStore;

#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub paths: bool,
    pub reindex: bool,
}

pub fn run(store: &NoteStore, options: DoctorOptions) -> Result<String> {
    let mut sections = Vec::new();

    if options.paths || !options.reindex {
        sections.push(render_paths(store));
    }

    if options.reindex {
        let index = NoteIndexStore::new(store.config()).refresh(store)?;
        sections.push(format!("reindexed {} notes", index.entries.len()));
    }

    Ok(sections.join("\n\n"))
}

fn render_paths(store: &NoteStore) -> String {
    let index = NoteIndexStore::new(store.config());
    format!(
        "root   {}\nnotes  {}\ndaily  {}\ncache  {}\nindex  {}\nconfig {}",
        store.config().root_dir.display(),
        store.config().notes_dir.display(),
        store.config().daily_dir.display(),
        store.config().cache_dir.display(),
        index.cache_path().display(),
        store.config().config_path.display(),
    )
}
