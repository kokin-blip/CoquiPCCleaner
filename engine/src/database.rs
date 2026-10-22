use crate::model::*;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection> {
    let c = Connection::open(path)?;
    c.busy_timeout(std::time::Duration::from_secs(10))?;
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let version: u32 = c.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    anyhow::ensure!(
        version <= 3,
        "This diagnostic database was created by a newer version of CoquiPCCleaner."
    );
    if version == 0 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS settings(id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS sessions(id TEXT PRIMARY KEY, started_at TEXT NOT NULL, status TEXT NOT NULL, scope TEXT NOT NULL, json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS files(session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
          path TEXT NOT NULL, parent TEXT NOT NULL, name TEXT NOT NULL, category TEXT NOT NULL,
          logical REAL NOT NULL, allocated REAL NOT NULL, directory INTEGER NOT NULL, modified TEXT NOT NULL, qualification TEXT NOT NULL,
          PRIMARY KEY(session_id,path));
        CREATE INDEX IF NOT EXISTS file_parent ON files(session_id,parent);
        CREATE INDEX IF NOT EXISTS file_size ON files(session_id,allocated DESC);
        PRAGMA user_version=1; COMMIT;")?;
    }
    if version < 2 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS cleanup_plans(id TEXT PRIMARY KEY, created_at TEXT NOT NULL, status TEXT NOT NULL, json TEXT NOT NULL);
        PRAGMA user_version=2; COMMIT;")?;
    }
    if version < 3 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS developer_reports(session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE, checked_at TEXT NOT NULL, status TEXT NOT NULL, json TEXT NOT NULL);
        PRAGMA user_version=3; COMMIT;")?;
    }
    Ok(c)
}

pub fn settings(c: &Connection) -> Result<AppSettings> {
    let value: Option<String> = c
        .query_row("SELECT json FROM settings WHERE id=1", [], |r| r.get(0))
        .optional()?;
    Ok(value
        .map(|v| serde_json::from_str(&v))
        .transpose()?
        .unwrap_or_default())
}
use rusqlite::OptionalExtension;
pub fn save_settings(c: &Connection, s: &AppSettings) -> Result<()> {
    anyhow::ensure!(
        (1..=3650).contains(&s.retention_days),
        "Retention must be between 1 and 3650 days"
    );
    anyhow::ensure!(
        ["system", "light", "dark"].contains(&s.appearance.as_str()),
        "Invalid appearance"
    );
    anyhow::ensure!(
        ["Balanced", "Performance", "Efficiency"].contains(&s.workload.preference.as_str()),
        "Invalid preference"
    );
    anyhow::ensure!(
        ["standard", "deep", "selected"].contains(&s.scan_mode.as_str()),
        "Invalid scan mode"
    );
    c.execute(
        "INSERT INTO settings VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
        [serde_json::to_string(s)?],
    )?;
    Ok(())
}
pub fn save_session(c: &Connection, s: &DiagnosticSession) -> Result<()> {
    c.execute("INSERT INTO sessions VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET status=excluded.status,json=excluded.json",
        params![s.id,s.started_at,s.status,s.scope,serde_json::to_string(s)?])?;
    Ok(())
}
pub fn session(c: &Connection, id: &str) -> Result<DiagnosticSession> {
    let json: String = c
        .query_row("SELECT json FROM sessions WHERE id=?1", [id], |r| r.get(0))
        .context("Scan not found")?;
    Ok(serde_json::from_str(&json)?)
}
pub fn history(c: &Connection, offset: u32, limit: u32) -> Result<Vec<DiagnosticSession>> {
    let mut q =
        c.prepare("SELECT json FROM sessions ORDER BY started_at DESC LIMIT ?1 OFFSET ?2")?;
    let rows = q.query_map(params![limit.clamp(1, 100), offset], |r| {
        r.get::<_, String>(0)
    })?;
    rows.map(|v| Ok(serde_json::from_str(&v?)?)).collect()
}
pub fn insert_files(c: &mut Connection, id: &str, batch: &[FileEntry]) -> Result<()> {
    let tx = c.transaction()?;
    {
        let mut q = tx.prepare_cached(
            "INSERT OR REPLACE INTO files VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        )?;
        for f in batch {
            q.execute(params![
                id,
                f.path,
                f.parent,
                f.name,
                f.category,
                f.logical_bytes,
                f.allocated_bytes,
                f.is_directory,
                f.modified_at,
                f.qualification
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
pub fn files(c: &Connection, q: &StorageQuery) -> Result<Vec<FileEntry>> {
    let order = match q.sort.as_str() {
        "name" => "name COLLATE NOCASE",
        "modified" => "modified",
        "logical" => "logical",
        _ => "allocated",
    };
    let dir = if q.descending { "DESC" } else { "ASC" };
    let sql=format!("SELECT path,parent,name,category,logical,allocated,directory,modified,qualification FROM files WHERE session_id=?1 AND (?2 IS NULL OR parent=?2) AND (?3 IS NULL OR category=?3) AND (?4=0 OR directory=0) ORDER BY {order} {dir},path ASC LIMIT ?5 OFFSET ?6");
    let mut s = c.prepare(&sql)?;
    let rows = s.query_map(
        params![
            q.session_id,
            q.parent,
            q.category,
            q.files_only,
            q.limit.clamp(1, 500),
            q.offset
        ],
        |r| {
            Ok(FileEntry {
                path: r.get(0)?,
                parent: r.get(1)?,
                name: r.get(2)?,
                category: r.get(3)?,
                logical_bytes: r.get(4)?,
                allocated_bytes: r.get(5)?,
                is_directory: r.get(6)?,
                modified_at: r.get(7)?,
                qualification: r.get(8)?,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
pub fn recover_interrupted(c: &Connection) -> Result<()> {
    let mut q = c.prepare("SELECT id FROM sessions WHERE status='running'")?;
    let ids = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        let mut s = session(c, &id)?;
        s.status = "interrupted".into();
        s.stage = "Application closed before scan completed".into();
        s.finished_at = Some(now());
        save_session(c, &s)?;
    }
    let mut q = c.prepare("SELECT session_id FROM developer_reports WHERE status='running'")?;
    let ids = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        if let Some(mut report) = crate::developer::get(c, &id)? {
            report.status = "interrupted".into();
            report.errors.push(
                "The app closed during validation. Recheck before using these observations.".into(),
            );
            crate::developer::save(c, &report)?;
        }
    }
    crate::cleanup::recover_interrupted(c)?;
    Ok(())
}
pub fn retain(c: &Connection, days: u32) -> Result<()> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days.into())).to_rfc3339();
    c.execute(
        "DELETE FROM sessions WHERE started_at < ?1 AND status != 'running'",
        [&cutoff],
    )?;
    c.execute(
        "DELETE FROM cleanup_plans WHERE created_at < ?1 AND status != 'running'",
        [&cutoff],
    )?;
    // Keep complete indexes per scope, plus inspectable partial scans until history expiry.
    c.execute("DELETE FROM files WHERE session_id IN (SELECT old.id FROM sessions old WHERE old.status='complete' AND EXISTS(SELECT 1 FROM sessions newer WHERE newer.status='complete' AND newer.scope=old.scope AND newer.started_at>old.started_at))",[])?;
    Ok(())
}
pub fn purge(c: &Connection) -> Result<()> {
    c.execute_batch("BEGIN IMMEDIATE; DELETE FROM developer_reports; DELETE FROM files; DELETE FROM sessions; DELETE FROM cleanup_plans; COMMIT; PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")?;
    Ok(())
}
