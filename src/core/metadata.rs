use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::util::time as util_time;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoteId(String);

impl NoteId {
    pub fn generate(now: OffsetDateTime) -> Self {
        let suffix = Uuid::new_v4().simple().to_string();
        Self(format!(
            "{}-{}",
            util_time::format_note_id_timestamp(now),
            &suffix[..6]
        ))
    }

    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            bail!("note id cannot be empty");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NoteId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteKind {
    Note,
    Daily,
}

impl FromStr for NoteKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "note" => Ok(Self::Note),
            "daily" => Ok(Self::Daily),
            _ => bail!("invalid note kind `{value}`"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    #[serde(alias = "inbox")]
    Active,
    Archived,
}

impl FromStr for NoteStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "inbox" | "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => bail!("invalid note status `{value}`"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteMeta {
    pub id: NoteId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::util::time::serde_rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub kind: NoteKind,
    pub status: NoteStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl NoteMeta {
    pub fn new(id: NoteId, created_at: OffsetDateTime, kind: NoteKind, status: NoteStatus) -> Self {
        Self {
            id,
            title: None,
            created_at,
            updated_at: created_at,
            tags: Vec::new(),
            kind,
            status,
            source: None,
            extra: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use time::macros::datetime;

    use super::NoteId;

    #[test]
    fn note_ids_are_unique() {
        let now = datetime!(2026-03-21 23:59:00 UTC);
        let ids = (0..64)
            .map(|_| NoteId::generate(now).to_string())
            .collect::<HashSet<_>>();

        assert_eq!(ids.len(), 64);
    }
}
