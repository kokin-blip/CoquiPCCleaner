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
        version <= 6,
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
    if version < 4 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE records(id TEXT PRIMARY KEY,kind TEXT NOT NULL,created_at TEXT NOT NULL,status TEXT NOT NULL,title TEXT NOT NULL,json TEXT NOT NULL);
        CREATE INDEX record_kind ON records(kind,created_at DESC);
        CREATE TABLE maintenance(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL);
        PRAGMA user_version=4; COMMIT;")?;
    }
    if version < 5 {
        c.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS firmware_preferences(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL); PRAGMA user_version=5; COMMIT;")?;
    }
    if version < 6 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS scan_plans(id TEXT PRIMARY KEY,created_at TEXT NOT NULL,json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS scan_gaps(session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,root TEXT NOT NULL,path TEXT NOT NULL,reason TEXT NOT NULL);
        CREATE INDEX IF NOT EXISTS scan_gap_page ON scan_gaps(session_id,root,path);
        CREATE TABLE IF NOT EXISTS removal_plans(id TEXT PRIMARY KEY,created_at TEXT NOT NULL,status TEXT NOT NULL,json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS application_observations(path TEXT PRIMARY KEY,json TEXT NOT NULL,missing_at TEXT);
        CREATE TABLE IF NOT EXISTS application_preferences(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL);
        PRAGMA user_version=6; COMMIT;")?;
    }
    c.execute_batch("CREATE TABLE IF NOT EXISTS administrator_records(id TEXT PRIMARY KEY,kind TEXT NOT NULL,created_at TEXT NOT NULL,json TEXT NOT NULL);")?;
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
    let sql=format!("SELECT path,parent,name,category,logical,allocated,directory,modified,qualification FROM files WHERE session_id=?1 AND (?2 IS NULL OR parent=?2) AND (?3 IS NULL OR category=?3) AND (?4=0 OR directory=0) AND (?7 IS NULL OR instr(lower(path),lower(?7))>0) ORDER BY {order} {dir},path ASC LIMIT ?5 OFFSET ?6");
    let mut s = c.prepare(&sql)?;
    let rows = s.query_map(
        params![
            q.session_id,
            q.parent,
            q.category,
            q.files_only,
            q.limit.clamp(1, 500),
            q.offset,
            q.search
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
    c.execute("UPDATE records SET status='interrupted',json=json_set(json,'$.status','interrupted') WHERE status='running'",[])?;
    crate::cleanup::recover_interrupted(c)?;
    crate::removal::recover(c)?;
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
    c.execute(
        "DELETE FROM records WHERE created_at < ?1 AND status != 'running'",
        [&cutoff],
    )?;
    c.execute(
        "DELETE FROM removal_plans WHERE created_at < ?1 AND status != 'running'",
        [&cutoff],
    )?;
    c.execute(
        "DELETE FROM application_observations WHERE json_extract(json,'$.observedAt') < ?1",
        [&cutoff],
    )?;
    c.execute(
        "DELETE FROM administrator_records WHERE created_at < ?1",
        [&cutoff],
    )?;
    // Keep the latest complete and latest unfinished index per scope; summaries remain retained.
    c.execute("DELETE FROM files WHERE session_id IN (SELECT old.id FROM sessions old WHERE old.status!='running' AND EXISTS(SELECT 1 FROM sessions newer WHERE newer.status!='running' AND (newer.status='complete')=(old.status='complete') AND newer.scope=old.scope AND (newer.started_at>old.started_at OR (newer.started_at=old.started_at AND newer.rowid>old.rowid))))",[])?;
    c.execute(
        "DELETE FROM scan_plans WHERE created_at < ?1",
        [(chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339()],
    )?;
    Ok(())
}
pub fn purge(c: &Connection) -> Result<()> {
    c.execute_batch("BEGIN IMMEDIATE; DELETE FROM application_observations; DELETE FROM removal_plans;DELETE FROM administrator_records; DELETE FROM scan_plans; DELETE FROM scan_gaps; DELETE FROM records; UPDATE firmware_preferences SET json=json_set(json,'$.dismissedReportId',NULL); DELETE FROM developer_reports; DELETE FROM files; DELETE FROM sessions; DELETE FROM cleanup_plans; COMMIT; PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")?;
    Ok(())
}

pub fn insert_gaps(c: &mut Connection, id: &str, batch: &[CoverageGap]) -> Result<()> {
    let tx = c.transaction()?;
    for gap in batch {
        tx.execute(
            "INSERT INTO scan_gaps VALUES(?1,?2,?3,?4)",
            params![id, gap.root, gap.path, gap.reason],
        )?;
    }
    tx.commit()?;
    Ok(())
}
pub fn coverage(c: &Connection, id: &str, offset: u32) -> Result<Vec<CoverageGap>> {
    let mut q=c.prepare("SELECT root,path,reason FROM scan_gaps WHERE session_id=?1 ORDER BY root,path LIMIT 100 OFFSET ?2")?;
    let rows = q.query_map(params![id, offset], |r| {
        Ok(CoverageGap {
            root: r.get(0)?,
            path: r.get(1)?,
            reason: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Keyset export for transferring a completed helper index without growing offset costs.
pub fn files_after(c: &Connection, id: &str, after: &str) -> Result<Vec<FileEntry>> {
    let mut q=c.prepare("SELECT path,parent,name,category,logical,allocated,directory,modified,qualification FROM files WHERE session_id=?1 AND path>?2 ORDER BY path LIMIT 500")?;
    let rows = q.query_map(rusqlite::params![id, after], |r| {
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
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
