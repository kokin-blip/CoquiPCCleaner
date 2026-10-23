use crate::{model::*, platform};
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use std::os::darwin::fs::MetadataExt as DarwinMetadataExt;
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct WalkSummary {
    pub stage: String,
    pub files: u32,
    pub logical: f64,
    pub allocated: f64,
    pub skipped: u32,
    pub permissions: Vec<PermissionRequirement>,
    pub errors: Vec<String>,
    pub categories: Vec<StorageCategory>,
    pub candidates: Vec<CleanupCandidate>,
    pub coverage: Vec<RootCoverage>,
}
pub fn normalized_roots(roots: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut roots = roots
        .into_iter()
        .map(|p| {
            anyhow::ensure!(p.is_absolute(), "Scan roots must be absolute");
            anyhow::ensure!(
                !fs::symlink_metadata(&p)?.file_type().is_symlink(),
                "A scan root cannot be a symbolic link"
            );
            let p = p.canonicalize()?;
            anyhow::ensure!(p.is_dir(), "Scan roots must be directories");
            Ok(p)
        })
        .collect::<Result<Vec<_>>>()?;
    roots.sort_by(|a, b| {
        a.components()
            .count()
            .cmp(&b.components().count())
            .then(a.cmp(b))
    });
    let mut result: Vec<PathBuf> = vec![];
    for p in roots {
        let identity = crate::volumes::mounted_identity(&p)?;
        if !result.iter().any(|r| {
            let a = fs::metadata(&p).ok();
            let b = fs::metadata(r).ok();
            matches!((a,b),(Some(a),Some(b)) if a.dev()==b.dev() && a.ino()==b.ino())
                || (p.starts_with(r)
                    && crate::volumes::mounted_identity(r).is_ok_and(|v| v == identity))
        }) {
            result.push(p);
        }
    }
    anyhow::ensure!(
        !result.is_empty(),
        "Select at least one readable scan folder"
    );
    Ok(result)
}
pub fn roots(request: &ScanRequest) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let values = match request.mode.as_str() {
        "standard" => vec![platform::home()?],
        "deep" => anyhow::bail!("Whole-disk scanning requires a reviewed mounted-volume plan"),
        "selected" => request.roots.iter().map(PathBuf::from).collect(),
        _ => anyhow::bail!("Unknown scan mode"),
    };
    anyhow::ensure!(!values.is_empty(), "Select at least one scan folder");
    let mut valid = Vec::new();
    let mut gaps = Vec::new();
    for path in values {
        match normalized_roots(vec![path.clone()]) {
            Ok(mut roots)=>valid.append(&mut roots),
            Err(_)=>gaps.push(format!("Selected location {} is unavailable, unreadable, or not a directory. Retry when it is accessible, or remove it from the scope.",path.display())),
        }
    }
    if !valid.is_empty() {
        valid = normalized_roots(valid)?;
    }
    Ok((valid, gaps))
}
pub fn category(path: &Path) -> &'static str {
    let p = path.to_string_lossy();
    if p.contains("/Library/Caches/") || p.contains("/private/var/tmp/") {
        return "Caches";
    }
    if p.contains(".app/") {
        return "Applications";
    }
    if p.contains("/target/")
        || p.contains("/node_modules/")
        || p.contains("/DerivedData/")
        || p.contains("/.pub-cache/")
    {
        return "Development";
    }
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "png" | "heic" | "gif" | "webp" | "mov" | "mp4" | "mkv" | "raw" => {
            "Photos & video"
        }
        "zip" | "gz" | "dmg" | "tar" | "7z" | "rar" => "Archives",
        "pdf" | "docx" | "txt" | "md" | "pages" | "xlsx" | "csv" => "Documents",
        _ if p.contains("/Downloads/") => "Downloads",
        _ => "Other",
    }
}
pub fn placeholder(path: &Path, metadata: &fs::Metadata) -> bool {
    #[cfg(target_os = "macos")]
    if metadata.st_flags() & 0x40000000 != 0 {
        return true;
    } // Darwin UF_DATALESS.
    let _ = metadata;
    path.extension().is_some_and(|v| v == "icloud")
}
fn modified(m: &fs::Metadata) -> String {
    m.modified()
        .ok()
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
        .unwrap_or_default()
}

/// Disk-backed identity and folder accumulation keeps memory independent of entry count.
pub fn walk<F: FnMut(FileEntry) -> Result<()>, P: FnMut(&WalkSummary) -> Result<()>>(
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    emit: F,
    progress: P,
) -> Result<WalkSummary> {
    walk_with_gaps(roots, exclusions, cancel, emit, progress, |_| Ok(()))
}

pub fn walk_with_gaps<
    F: FnMut(FileEntry) -> Result<()>,
    P: FnMut(&WalkSummary) -> Result<()>,
    G: FnMut(CoverageGap) -> Result<()>,
>(
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    mut emit: F,
    mut progress: P,
    mut gap: G,
) -> Result<WalkSummary> {
    use rusqlite::{params, Connection};
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let scratch =
        Scratch(std::env::temp_dir().join(format!("coqui-walk-{}.db", uuid::Uuid::new_v4())));
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&scratch.0)?;
    let sql = Connection::open(&scratch.0)?;
    sql.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA cache_size=-8192; PRAGMA temp_store=FILE; CREATE TABLE identities(dev INTEGER,ino INTEGER,PRIMARY KEY(dev,ino)) WITHOUT ROWID; CREATE TABLE folders(path TEXT PRIMARY KEY,parent TEXT,depth INTEGER,json TEXT,logical REAL DEFAULT 0,allocated REAL DEFAULT 0); CREATE INDEX folder_depth ON folders(depth DESC); CREATE TABLE pending(path TEXT PRIMARY KEY,parent_dev INTEGER,parent_ino INTEGER);")?;
    let mut summary = WalkSummary {
        stage: "Storage metadata • traversing readable locations".into(),
        coverage: roots
            .iter()
            .map(|r| RootCoverage {
                root: crate::volumes::friendly(r).to_string_lossy().into_owned(),
                state: "notStarted".into(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let mut cats: HashMap<String, StorageCategory> = HashMap::new();
    let mut last = std::time::Instant::now();
    for (index, root) in roots.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let friendly_root = crate::volumes::friendly(root)
            .to_string_lossy()
            .into_owned();
        summary.coverage[index].state = "running".into();
        let mut note = |summary: &mut WalkSummary, path: &Path, reason: &str| -> Result<()> {
            summary.skipped = summary.skipped.saturating_add(1);
            let coverage = &mut summary.coverage[index];
            let n = coverage.skips.entry(reason.into()).or_default();
            *n = n.saturating_add(1);
            let item = CoverageGap {
                root: friendly_root.clone(),
                path: crate::volumes::friendly(path)
                    .to_string_lossy()
                    .into_owned(),
                reason: reason.into(),
            };
            if coverage.examples.len() < 25 {
                coverage.examples.push(item.clone());
            }
            gap(item)
        };
        let root_dev = match fs::symlink_metadata(root) {
            Ok(m) => m.dev(),
            Err(_) => {
                note(&mut summary, root, "disconnected")?;
                summary.coverage[index].state = "unavailable".into();
                summary
                    .errors
                    .push(format!("Scan root {} became unavailable", root.display()));
                continue;
            }
        };
        let root_mount = crate::volumes::mounted_identity(root)?;
        use rusqlite::OptionalExtension;
        let rm = fs::symlink_metadata(root.parent().unwrap_or(root))?;
        sql.execute(
            "INSERT OR IGNORE INTO pending VALUES(?1,?2,?3)",
            params![
                root.to_str().context("Unrepresentable root")?,
                rm.dev(),
                rm.ino()
            ],
        )?;
        sql.execute_batch("BEGIN")?;
        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let next: Option<(String, u64, u64)> = sql
                .query_row(
                    "SELECT path,parent_dev,parent_ino FROM pending LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?;
            let Some((next, parent_dev, parent_ino)) = next else {
                break;
            };
            sql.execute("DELETE FROM pending WHERE path=?1", [&next])?;
            let owned = PathBuf::from(next);
            let p = owned.as_path();
            let parent = p
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(p);
            if !fs::symlink_metadata(parent).is_ok_and(|m| {
                m.is_dir()
                    && !m.file_type().is_symlink()
                    && m.dev() == parent_dev
                    && m.ino() == parent_ino
            }) {
                note(&mut summary, p, "changedParent")?;
                continue;
            }
            summary.coverage[index].visited = summary.coverage[index].visited.saturating_add(1);
            if summary.coverage[index].visited.is_multiple_of(1000) {
                sql.execute_batch("COMMIT;BEGIN")?;
            }
            if last.elapsed() >= std::time::Duration::from_millis(250) {
                progress(&summary)?;
                last = std::time::Instant::now();
            }
            if p == scratch.0
                || exclusions
                    .iter()
                    .any(|x| crate::volumes::same_or_below(p, x))
            {
                note(&mut summary, p, "excluded")?;
                continue;
            }
            let metadata = match fs::symlink_metadata(p) {
                Ok(m) => m,
                Err(_) => {
                    note(&mut summary, p, "filesystemRace")?;
                    if summary.errors.len() < 100 {
                        summary
                            .errors
                            .push("An entry disappeared or became unreadable".into());
                    }
                    continue;
                }
            };
            let reason = if metadata.dev() != root_dev
                || (metadata.is_dir()
                    && crate::volumes::mounted_identity(p).is_ok_and(|v| v != root_mount))
            {
                Some("mountBoundary")
            } else if metadata.file_type().is_symlink() {
                Some("symbolicLink")
            } else if placeholder(p, &metadata) {
                Some("cloudPlaceholder")
            } else {
                None
            };
            if let Some(reason) = reason {
                note(&mut summary, p, reason)?;
                continue;
            }
            if p.to_str().is_none() {
                note(&mut summary, p, "unrepresentableName")?;
                continue;
            }
            if !metadata.is_file() && !metadata.is_dir() {
                note(&mut summary, p, "specialFile")?;
                continue;
            }
            if sql.execute(
                "INSERT OR IGNORE INTO identities VALUES(?1,?2)",
                params![metadata.dev(), metadata.ino()],
            )? == 0
            {
                note(&mut summary, p, "duplicateIdentity")?;
                continue;
            }
            let path = crate::volumes::friendly(p).to_string_lossy().into_owned();
            let parent = if p == root {
                String::new()
            } else {
                crate::volumes::friendly(p.parent().unwrap_or(root))
                    .to_string_lossy()
                    .into_owned()
            };
            let mut f = FileEntry {
                path: path.clone(),
                parent: parent.clone(),
                name: p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&path)
                    .into(),
                category: category(p).into(),
                logical_bytes: 0.,
                allocated_bytes: 0.,
                is_directory: metadata.is_dir(),
                modified_at: modified(&metadata),
                qualification: String::new(),
            };
            if metadata.is_dir() {
                sql.execute(
                    "INSERT OR IGNORE INTO folders(path,parent,depth,json) VALUES(?1,?2,?3,?4)",
                    params![
                        path,
                        parent,
                        p.components().count(),
                        serde_json::to_string(&f)?
                    ],
                )?;
                match fs::read_dir(p) {
                    Ok(entries) => {
                        // Stream one directory into an on-disk frontier; no recursive path stack or entry list.
                        for child in entries {
                            if cancel.load(Ordering::Relaxed) {
                                break;
                            }
                            match child {
                                Ok(e) => {
                                    let path = e.path();
                                    if let Some(text) = path.to_str() {
                                        sql.execute(
                                            "INSERT OR IGNORE INTO pending VALUES(?1,?2,?3)",
                                            params![text, metadata.dev(), metadata.ino()],
                                        )?;
                                    } else {
                                        note(&mut summary, &path, "unrepresentableName")?;
                                    }
                                }
                                Err(_) => {
                                    note(&mut summary, p, "filesystemRace")?;
                                }
                            }
                            if last.elapsed() >= std::time::Duration::from_millis(250) {
                                progress(&summary)?;
                                last = std::time::Instant::now();
                            }
                        }
                    }
                    Err(e) => {
                        let denied = e.kind() == std::io::ErrorKind::PermissionDenied;
                        note(
                            &mut summary,
                            p,
                            if denied {
                                "permissionDenied"
                            } else {
                                "filesystemRace"
                            },
                        )?;
                        if denied && summary.permissions.len() < 100 {
                            summary.permissions.push(PermissionRequirement{path:crate::volumes::friendly(p).to_string_lossy().into_owned(),reason:"macOS denied directory access".into(),guidance:"Review Full Disk Access or folder permissions and retry. Administrator access does not override all protections.".into()});
                        }
                        if !denied && summary.errors.len() < 100 {
                            summary
                                .errors
                                .push(format!("Could not enumerate {}", p.display()));
                        }
                    }
                }
                continue;
            }
            f.logical_bytes = metadata.len() as f64;
            f.allocated_bytes = metadata.blocks() as f64 * 512.;
            summary.files = summary.files.saturating_add(1);
            summary.coverage[index].files = summary.coverage[index].files.saturating_add(1);
            summary.logical += f.logical_bytes;
            summary.allocated += f.allocated_bytes;
            let c = cats.entry(f.category.clone()).or_insert(StorageCategory {
                name: f.category.clone(),
                logical_bytes: 0.,
                allocated_bytes: 0.,
                file_count: 0,
            });
            c.logical_bytes += f.logical_bytes;
            c.allocated_bytes += f.allocated_bytes;
            c.file_count = c.file_count.saturating_add(1);
            sql.execute(
                "UPDATE folders SET logical=logical+?1,allocated=allocated+?2 WHERE path=?3",
                params![f.logical_bytes, f.allocated_bytes, parent],
            )?;
            emit(f).context("Could not persist scanned file")?;
            if summary.files.is_multiple_of(250) {
                progress(&summary)?;
                last = std::time::Instant::now();
            }
        }
        sql.execute_batch("COMMIT")?;
        summary.coverage[index].state = if cancel.load(Ordering::Relaxed) {
            "cancelled"
        } else if summary.coverage[index].skips.is_empty() {
            "complete"
        } else {
            "completedWithOmissions"
        }
        .into();
    }
    // Aggregate one folder at a time, deepest first; SQLite owns the sorting and identity set.
    summary.stage = if cancel.load(Ordering::Relaxed) {
        "Retaining partial folder totals after cancellation"
    } else {
        "Aggregating inspected folder allocation"
    }
    .into();
    progress(&summary)?;
    sql.execute_batch("BEGIN")?;
    let mut q =
        sql.prepare("SELECT json,logical,allocated FROM folders ORDER BY depth DESC,path")?;
    let mut rows = q.query([])?;
    while let Some(row) = rows.next()? {
        // Cancellation still retains discovered folder context, with sizes explicitly partial.
        let mut f: FileEntry = serde_json::from_str(&row.get::<_, String>(0)?)?;
        // Read the current total because children updated it after this ordered cursor began.
        (f.logical_bytes, f.allocated_bytes) = sql.query_row(
            "SELECT logical,allocated FROM folders WHERE path=?1",
            [&f.path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if cancel.load(Ordering::Relaxed) {
            f.qualification =
                "Partial allocation at cancellation; descendants may be missing.".into();
        }
        sql.execute(
            "UPDATE folders SET logical=logical+?1,allocated=allocated+?2 WHERE path=?3",
            params![f.logical_bytes, f.allocated_bytes, f.parent],
        )?;
        if f.path.ends_with("/Library/Caches") || f.path.ends_with("/private/var/tmp") {
            if summary.candidates.len() < 500 {
                summary.candidates.push(CleanupCandidate{path:f.path.clone(),allocated_bytes:f.allocated_bytes,logical_bytes:f.logical_bytes,modified_at:f.modified_at.clone(),origin:"Known cache/temporary directory".into(),reason:"Directory name and size alone do not establish disposability.".into(),safety:"Review required".into(),consequences:"Applications may regenerate contents; removal can discard useful data or slow subsequent work.".into()});
            }
        }
        emit(f)?;
        if last.elapsed() >= std::time::Duration::from_millis(250) {
            progress(&summary)?;
            last = std::time::Instant::now();
        }
    }
    sql.execute_batch("COMMIT")?;
    summary.categories = cats.into_values().collect();
    summary
        .categories
        .sort_by(|a, b| b.allocated_bytes.total_cmp(&a.allocated_bytes));
    progress(&summary)?;
    Ok(summary)
}
