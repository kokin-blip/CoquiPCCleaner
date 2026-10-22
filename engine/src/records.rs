use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
pub fn save(
    c: &Connection,
    kind: &str,
    id: &str,
    created: &str,
    status: &str,
    title: &str,
    value: &impl Serialize,
) -> Result<()> {
    c.execute("INSERT INTO records VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET status=excluded.status,title=excluded.title,json=excluded.json",params![id,kind,created,status,title,serde_json::to_string(value)?])?;
    Ok(())
}
pub fn get<T: DeserializeOwned>(c: &Connection, kind: &str, id: &str) -> Result<T> {
    let json: String = c
        .query_row(
            "SELECT json FROM records WHERE kind=?1 AND id=?2",
            params![kind, id],
            |r| r.get(0),
        )
        .context("Retained record not found")?;
    Ok(serde_json::from_str(&json)?)
}
pub fn latest<T: DeserializeOwned>(c: &Connection, kind: &str) -> Result<Option<T>> {
    let json: Option<String> = c
        .query_row(
            "SELECT json FROM records WHERE kind=?1 ORDER BY created_at DESC LIMIT 1",
            [kind],
            |r| r.get(0),
        )
        .optional()?;
    json.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
}
pub fn list(
    c: &Connection,
    kind: Option<&str>,
    offset: u32,
) -> Result<Vec<crate::model::ActivityRecord>> {
    let mut q=c.prepare("SELECT id,kind,created_at,title,status FROM records WHERE (?1 IS NULL OR kind=?1) ORDER BY created_at DESC LIMIT 50 OFFSET ?2")?;
    let rows = q.query_map(params![kind, offset], |r| {
        Ok(crate::model::ActivityRecord {
            id: r.get(0)?,
            kind: r.get(1)?,
            created_at: r.get(2)?,
            title: r.get(3)?,
            status: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
