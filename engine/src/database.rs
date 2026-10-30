use crate::model::*;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection> {
    let c = Connection::open(path)?;
    c.busy_timeout(std::time::Duration::from_secs(10))?;
    c.execute_batch("PRAGMA foreign_keys=ON; PRAGMA cache_size=-8192; PRAGMA temp_store=FILE;")?;
    let version: u32 = c.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    anyhow::ensure!(
        version <= 8,
        "This diagnostic database was created by a newer version of CoquiPCCleaner."
    );
    if version == 0 {
        c.execute_batch("PRAGMA journal_mode=WAL;")?;
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
    if version < 7 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS administrator_records(id TEXT PRIMARY KEY,kind TEXT NOT NULL,created_at TEXT NOT NULL,json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS storage_metrics(session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,path TEXT NOT NULL,file_count INTEGER NOT NULL,small_file_count INTEGER NOT NULL,small_allocated REAL NOT NULL,largest_file REAL NOT NULL,PRIMARY KEY(session_id,path)) WITHOUT ROWID;
        CREATE INDEX IF NOT EXISTS session_scope_retention ON sessions(scope,status,started_at DESC);
        CREATE INDEX IF NOT EXISTS scan_gap_export ON scan_gaps(session_id);
        PRAGMA user_version=7; COMMIT;")?;
    }
    if version < 8 {
        c.execute_batch("BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS cache_batches(id TEXT PRIMARY KEY,session_id TEXT NOT NULL,policy TEXT NOT NULL,revision INTEGER NOT NULL,status TEXT NOT NULL,created_at TEXT NOT NULL,reviewed_at TEXT,expires_at TEXT,approved_at TEXT,finished_at TEXT,detail TEXT NOT NULL);
        CREATE INDEX IF NOT EXISTS cache_batch_history ON cache_batches(created_at DESC,id);
        CREATE TABLE IF NOT EXISTS cache_batch_items(batch_id TEXT NOT NULL REFERENCES cache_batches(id) ON DELETE CASCADE,path TEXT NOT NULL,namespace TEXT NOT NULL,logical REAL NOT NULL,allocated REAL NOT NULL,indexed_allocated REAL NOT NULL,modified TEXT NOT NULL,eligible INTEGER NOT NULL DEFAULT 0,status TEXT NOT NULL,detail TEXT NOT NULL,identity TEXT,trash_path TEXT,PRIMARY KEY(batch_id,path)) WITHOUT ROWID;
        CREATE INDEX IF NOT EXISTS cache_batch_group ON cache_batch_items(batch_id,namespace);
        PRAGMA user_version=8; COMMIT;")?;
    }
    // An already migrated reader must not execute DDL or acquire a write lock.
    // Reuse file_parent for child lookup; avoid duplicating millions of full paths
    // in metric-specific indexes. SQLite sorts bounded output with file-backed temp.
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

pub fn insert_indexed_files(
    c: &mut Connection,
    id: &str,
    batch: &[IndexedStorageEntry],
) -> Result<()> {
    let tx = c.transaction()?;
    {
        let mut entry = tx.prepare_cached(
            "INSERT OR REPLACE INTO files VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        )?;
        let mut metrics =
            tx.prepare_cached("INSERT OR REPLACE INTO storage_metrics VALUES(?1,?2,?3,?4,?5,?6)")?;
        for item in batch {
            let f = &item.entry;
            entry.execute(params![
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
            if let Some(m) = &item.metrics {
                metrics.execute(params![
                    id,
                    f.path,
                    m.file_count,
                    m.small_file_count,
                    m.small_allocated_bytes,
                    m.largest_file_bytes
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

fn entry_at(r: &rusqlite::Row<'_>) -> rusqlite::Result<FileEntry> {
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
}

fn metrics_at(r: &rusqlite::Row<'_>) -> rusqlite::Result<Option<StorageMetrics>> {
    let Some(file_count) = r.get::<_, Option<u64>>(9)? else {
        return Ok(None);
    };
    Ok(Some(StorageMetrics {
        file_count,
        small_file_count: r.get(10)?,
        small_allocated_bytes: r.get(11)?,
        largest_file_bytes: r.get(12)?,
    }))
}

/// Immediate-child totals are computed over every child, independently of the bounded tiles.
pub fn storage_map(c: &Connection, query: &StorageMapQuery) -> Result<StorageMap> {
    let s = session(c, &query.session_id)?;
    let (allocated,logical,mut count): (f64,f64,Option<f64>) = c.query_row("SELECT COALESCE(SUM(f.allocated),0),COALESCE(SUM(f.logical),0),CASE WHEN COUNT(*)=COUNT(CASE WHEN f.directory=0 THEN 1 ELSE m.file_count END) THEN COALESCE(SUM(CASE WHEN f.directory=0 THEN 1 ELSE m.file_count END),0) ELSE NULL END FROM files f LEFT JOIN storage_metrics m ON m.session_id=f.session_id AND m.path=f.path WHERE f.session_id=?1 AND f.parent=?2",params![query.session_id,query.parent],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    let order = if query.metric == "logical" {
        "f.logical"
    } else {
        "f.allocated"
    };
    // Restrict work to this folder before ranking. Without this constraint SQLite
    // can choose file_size and walk millions of unrelated paths to fill a page.
    let sql = format!("SELECT f.path,f.parent,f.name,f.category,f.logical,f.allocated,f.directory,f.modified,f.qualification,m.file_count,m.small_file_count,m.small_allocated,m.largest_file FROM files f INDEXED BY file_parent LEFT JOIN storage_metrics m ON m.session_id=f.session_id AND m.path=f.path WHERE f.session_id=?1 AND f.parent=?2 ORDER BY {order} DESC,f.path LIMIT ?3");
    let mut statement = c.prepare(&sql)?;
    let rows = statement.query_map(
        params![query.session_id, query.parent, query.limit.clamp(1, 60)],
        |r| Ok((entry_at(r)?, metrics_at(r)?)),
    )?;
    let developer = crate::developer::get(c, &query.session_id)?;
    let mut nodes = Vec::new();
    for row in rows {
        let (entry, metrics) = row?;
        let metrics = if entry.is_directory {
            metrics
        } else {
            Some(StorageMetrics {
                file_count: 1,
                small_file_count: u64::from(entry.logical_bytes <= 65_536.),
                small_allocated_bytes: if entry.logical_bytes <= 65_536. {
                    entry.allocated_bytes
                } else {
                    0.
                },
                largest_file_bytes: entry.allocated_bytes,
            })
        };
        let context = indexed_project_context(c, &query.session_id, &entry.path)?;
        let mut node = crate::storage::identify(entry, metrics.as_ref(), context.as_deref());
        let git_marker = Path::new(&node.entry.path)
            .join(".git")
            .to_string_lossy()
            .into_owned();
        if node.entry.is_directory
            && c.query_row(
                "SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1 AND path=?2)",
                params![query.session_id, git_marker],
                |r| r.get::<_, bool>(0),
            )?
        {
            node.development.push(DevelopmentIndicator {label:"Git project · keep".into(), attention:true,
                detail:"The scan indexed a .git marker directly in this folder. It may hold source, local commits, worktrees and ignored files. Review current Git state and all unique contents; age or a remote branch match is not a whole-folder backup.".into()});
        }
        if !node.development.is_empty() {
            if let Some(artifact) = developer.as_ref().and_then(|r| {
                r.artifacts
                    .iter()
                    .filter(|a| {
                        Path::new(&node.entry.path).starts_with(&a.path)
                            || a.git.root.as_deref() == Some(node.entry.path.as_str())
                            || (a.technology == "Git"
                                && Path::new(&node.entry.path).starts_with(&a.project_root))
                    })
                    .max_by_key(|a| a.path.len())
            }) {
                node.development.extend(crate::development::git_indicators(
                    &artifact.git,
                    &artifact.checked_at,
                ));
            } else {
                node.development.push(DevelopmentIndicator {label:"Git not checked".into(), attention:true,
                    detail:"No retained developer check covers this exact output. Use Optimization → Developer artifacts to check current Git/worktree state. Size and timestamps come from the storage scan.".into()});
            }
        }
        nodes.push(node);
    }
    let shown_allocated: f64 = nodes.iter().map(|n| n.entry.allocated_bytes).sum();
    let shown_logical: f64 = nodes.iter().map(|n| n.entry.logical_bytes).sum();
    let shown_count = nodes
        .iter()
        .try_fold(0., |sum, n| n.file_count.map(|count| sum + count));
    let index_unavailable = nodes.is_empty() && query.parent.is_empty() && s.file_count > 0;
    if index_unavailable {
        count = None;
    }
    let unfinished = s.status != "complete" && s.status != "partial"
        || (s.categories.is_empty() && s.file_count > 0);
    let qualification = format!("{} {} Sizes describe inspected metadata; allocated space is not guaranteed recoverable space. APFS clones, compression and shared volume capacity can differ from these totals. Small files are logical sizes of 64 KiB or less. Largest file is measured in allocated bytes. Analysis labels do not authorize removal.",if index_unavailable {"Folder index unavailable: it was not retained or finalization/import did not finish. An empty map does not mean the disk is empty."} else if unfinished {"Partial or unfinished index: folder totals may be incomplete."} else if s.skipped_count>0 {"Inspected scope only; coverage omissions remain."} else {"Inspected scope; mounted and excluded locations remain outside these totals."},if count.is_none() {"Descendant counts are unavailable for older or unfinished folder records; run a new scan for these measurements."} else {"Remainder includes every smaller immediate child omitted from the bounded map."});
    Ok(StorageMap {
        nodes,
        total_allocated_bytes: allocated,
        total_logical_bytes: logical,
        file_count: count,
        remainder_allocated_bytes: (allocated - shown_allocated).max(0.),
        remainder_logical_bytes: (logical - shown_logical).max(0.),
        remainder_file_count: count
            .zip(shown_count)
            .map(|(all, shown)| (all - shown).max(0.)),
        qualification,
    })
}

fn indexed_project_context(c: &Connection, id: &str, path: &str) -> Result<Option<String>> {
    // Exact indexed marker paths only. This reads no project contents or cloud data.
    for ancestor in Path::new(path).ancestors().skip(1).take(24) {
        for (marker, kind) in [
            ("Cargo.toml", "rust"),
            ("package.json", "javascript"),
            ("pyvenv.cfg", "python"),
            ("build.gradle", "gradle"),
            ("build.gradle.kts", "gradle"),
            ("ProjectSettings/ProjectVersion.txt", "unity"),
        ] {
            let marker = ancestor.join(marker).to_string_lossy().into_owned();
            if c.query_row("SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1 AND path=?2 AND directory=0)",params![id,marker],|r|r.get::<_,bool>(0))? { return Ok(Some(kind.into())); }
        }
    }
    Ok(None)
}
pub fn files(c: &Connection, q: &StorageQuery) -> Result<Vec<FileEntry>> {
    let order = match q.sort.as_str() {
        "name" => "name COLLATE NOCASE",
        "modified" => "modified",
        "logical" => "logical",
        _ => "allocated",
    };
    let dir = if q.descending { "DESC" } else { "ASC" };
    // A nullable OR prevents a parent equality from constraining the index.
    // Folder pages use the existing parent index; global large-file/search pages
    // retain their planner-selected index and original filter/order semantics.
    let (source, parent) = if q.parent.is_some() {
        ("files INDEXED BY file_parent", "parent=?2")
    } else {
        ("files", "?2 IS NULL")
    };
    let sql=format!("SELECT path,parent,name,category,logical,allocated,directory,modified,qualification FROM {source} WHERE session_id=?1 AND {parent} AND (?3 IS NULL OR category=?3) AND (?4=0 OR directory=0) AND (?7 IS NULL OR instr(lower(path),lower(?7))>0) ORDER BY {order} {dir},path ASC LIMIT ?5 OFFSET ?6");
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
    crate::cache_bulk::recover(c)?;
    crate::removal::recover(c)?;
    Ok(())
}
pub fn retain(c: &Connection, days: u32) -> Result<()> {
    retain_inner(c, days, None)
}

pub fn retain_after_scan(
    c: &Connection,
    days: u32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<()> {
    retain_inner(c, days, Some(cancel))
}

fn retain_inner(
    c: &Connection,
    days: u32,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<()> {
    if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
        return Ok(());
    }
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days.into())).to_rfc3339();
    // Select expired indexes from session metadata once. A monolithic indexed delete
    // can hold a writer for many seconds; short batches yield between commits.
    let expired = {
        let mut q =
            c.prepare("SELECT id FROM sessions WHERE started_at<?1 AND status!='running'")?;
        let rows = q.query_map([&cutoff], |r| r.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    for id in &expired {
        if !prune_index(c, id, true, cancel)? {
            return Ok(());
        }
    }
    c.execute(
        "DELETE FROM sessions WHERE started_at < ?1 AND status != 'running'",
        [&cutoff],
    )?;
    c.execute(
        "DELETE FROM cache_batches WHERE created_at < ?1 AND status NOT IN ('running','preparing')",
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
    // Keep the latest complete and latest unfinished index per scope; summaries and
    // omission evidence remain retained until the configured history expiry.
    let stale = {
        let mut q = c.prepare("SELECT id FROM (SELECT id,ROW_NUMBER() OVER(PARTITION BY scope,(status='complete') ORDER BY started_at DESC,rowid DESC) AS rank FROM sessions WHERE status!='running') WHERE rank>1")?;
        let rows = q.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    for id in stale {
        if !prune_index(c, &id, false, cancel)? {
            return Ok(());
        }
    }
    c.execute(
        "DELETE FROM scan_plans WHERE created_at < ?1",
        [(chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339()],
    )?;
    Ok(())
}

fn prune_index(
    c: &Connection,
    id: &str,
    gaps: bool,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<bool> {
    for table in ["files", "storage_metrics"] {
        // storage_metrics is WITHOUT ROWID; use the indexed path key in either table.
        let sql = format!("DELETE FROM {table} WHERE session_id=?1 AND path IN (SELECT path FROM {table} WHERE session_id=?1 LIMIT 1000)");
        loop {
            if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
                return Ok(false);
            }
            if c.execute(&sql, [id])? == 0 {
                break;
            }
            std::thread::yield_now();
        }
    }
    if gaps {
        loop {
            if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
                return Ok(false);
            }
            if c.execute("DELETE FROM scan_gaps WHERE rowid IN(SELECT rowid FROM scan_gaps WHERE session_id=?1 LIMIT 1000)",[id])?==0 {break;}
            std::thread::yield_now();
        }
    }
    Ok(true)
}
pub fn purge(c: &Connection) -> Result<()> {
    c.execute_batch("BEGIN IMMEDIATE; DELETE FROM cache_batches; DELETE FROM application_observations; DELETE FROM removal_plans;DELETE FROM administrator_records; DELETE FROM scan_plans; DELETE FROM scan_gaps; DELETE FROM records; UPDATE firmware_preferences SET json=json_set(json,'$.dismissedReportId',NULL); DELETE FROM developer_reports; DELETE FROM files; DELETE FROM sessions; DELETE FROM cleanup_plans; COMMIT; PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")?;
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
    coverage_filtered(c, id, offset, None)
}
pub fn coverage_filtered(
    c: &Connection,
    id: &str,
    offset: u32,
    reason: Option<&str>,
) -> Result<Vec<CoverageGap>> {
    anyhow::ensure!(
        reason.is_none_or(|v| v == "permissionDenied"),
        "Unsupported coverage filter"
    );
    let mut q=c.prepare("SELECT root,path,reason FROM scan_gaps WHERE session_id=?1 AND (?3 IS NULL OR reason=?3) ORDER BY root,path LIMIT 100 OFFSET ?2")?;
    let rows = q.query_map(params![id, offset, reason], |r| {
        Ok(CoverageGap {
            root: r.get(0)?,
            path: r.get(1)?,
            reason: r.get(2)?,
            guidance: crate::storage::permission_guidance(
                &r.get::<_, String>(1)?,
                &r.get::<_, String>(2)?,
            ),
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

pub fn indexed_files_after(
    c: &Connection,
    id: &str,
    after: &str,
) -> Result<Vec<IndexedStorageEntry>> {
    let mut q=c.prepare("SELECT f.path,f.parent,f.name,f.category,f.logical,f.allocated,f.directory,f.modified,f.qualification,m.file_count,m.small_file_count,m.small_allocated,m.largest_file FROM files f LEFT JOIN storage_metrics m ON m.session_id=f.session_id AND m.path=f.path WHERE f.session_id=?1 AND f.path>?2 ORDER BY f.path LIMIT 500")?;
    let rows = q.query_map(params![id, after], |r| {
        Ok(IndexedStorageEntry {
            entry: entry_at(r)?,
            metrics: metrics_at(r)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoverageExport {
    pub gaps: Vec<CoverageGap>,
    pub after: u64,
}

pub fn gaps_after(c: &Connection, id: &str, after: u64) -> Result<CoverageExport> {
    let mut q=c.prepare("SELECT rowid,root,path,reason FROM scan_gaps WHERE session_id=?1 AND rowid>?2 ORDER BY rowid LIMIT 500")?;
    let mut rows = q.query(params![id, after])?;
    let mut result = CoverageExport {
        gaps: Vec::new(),
        after,
    };
    while let Some(r) = rows.next()? {
        result.after = r.get(0)?;
        result.gaps.push(CoverageGap {
            root: r.get(1)?,
            path: r.get(2)?,
            reason: r.get(3)?,
            guidance: crate::storage::permission_guidance(
                &r.get::<_, String>(2)?,
                &r.get::<_, String>(3)?,
            ),
        });
    }
    Ok(result)
}
