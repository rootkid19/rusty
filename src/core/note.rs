use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};

use crate::core::metadata::NoteMeta;

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub path: PathBuf,
    pub meta: NoteMeta,
    pub body: String,
}

impl Note {
    pub fn from_markdown(path: PathBuf, contents: &str) -> Result<Self> {
        let (meta, body) = split_frontmatter(contents)?;
        let meta = serde_yaml::from_str::<NoteMeta>(&meta)
            .with_context(|| format!("invalid frontmatter in {}", path.display()))?;
        Ok(Self { path, meta, body })
    }

    pub fn body_from_markdown(contents: &str) -> Result<String> {
        let (_, body) = split_frontmatter(contents)?;
        Ok(body)
    }

    pub fn to_markdown(&self) -> Result<String> {
        let mut yaml = serde_yaml::to_string(&self.meta)?;
        if let Some(stripped) = yaml.strip_prefix("---\n") {
            yaml = stripped.to_string();
        }

        if !yaml.ends_with('\n') {
            yaml.push('\n');
        }

        Ok(format!("---\n{}---\n{}", yaml, self.body))
    }
}

fn split_frontmatter(contents: &str) -> Result<(String, String)> {
    let mut lines = contents.split_inclusive('\n');
    let Some(first) = lines.next() else {
        bail!("note is empty");
    };

    if first != "---\n" && first != "---\r\n" && first != "---" {
        bail!("missing frontmatter delimiter");
    }

    let mut yaml = String::new();
    let mut consumed = first.len();
    let mut found_closing = false;

    for line in lines {
        consumed += line.len();
        if line == "---\n" || line == "---\r\n" || line == "---" {
            found_closing = true;
            break;
        }
        yaml.push_str(line);
    }

    if !found_closing {
        return Err(anyhow!("unterminated frontmatter"));
    }

    Ok((yaml, contents[consumed..].to_string()))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use time::macros::datetime;

    use super::Note;
    use crate::core::metadata::{NoteId, NoteKind, NoteMeta, NoteStatus};

    #[test]
    fn frontmatter_roundtrip_preserves_unknown_fields() -> Result<()> {
        let note = Note {
            path: "note.md".into(),
            meta: NoteMeta {
                id: NoteId::new("20260321T235900Z-8f3a")?,
                title: Some("auth bypass notes".to_string()),
                created_at: datetime!(2026-03-21 23:59:00 UTC),
                updated_at: datetime!(2026-03-21 23:59:00 UTC),
                tags: vec!["security".to_string()],
                kind: NoteKind::Note,
                status: NoteStatus::Active,
                source: Some("cli".to_string()),
                extra: serde_yaml::from_str("rating: high\nteam: appsec\n")?,
            },
            body: "body stays untouched\nsecond line\n".to_string(),
        };

        let encoded = note.to_markdown()?;
        let decoded = Note::from_markdown("note.md".into(), &encoded)?;

        assert_eq!(decoded.meta, note.meta);
        assert_eq!(decoded.body, note.body);
        Ok(())
    }
}
