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
    // A VM's sparse .raw image is not a camera photograph.
    if p.contains("/com.docker.docker/")
        || p.contains(".vmwarevm/")
        || p.contains(".pvm/")
        || matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("vmdk" | "qcow2" | "vdi")
        )
    {
        return "Virtual machines";
    }
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

/// Conservative, evidence-described interpretations of indexed paths and aggregate metadata.
/// No classification here participates in the cleanup authorization policy.
pub fn identify(
    mut entry: FileEntry,
    metrics: Option<&StorageMetrics>,
    project: Option<&str>,
) -> StorageMapNode {
    let path = entry.path.replace('\\', "/");
    let parts = path.split('/').collect::<Vec<_>>();
    let has = |name: &str| parts.contains(&name);
    if category(Path::new(&entry.path)) == "Virtual machines" {
        entry.category = "Virtual machines".into();
    }
    let (kind, title, reason, consequences) = if has("target")
        && (has("debug") || has("release") || project == Some("rust"))
    {
        ("rustBuild","Rust build artifacts","The indexed path matches Cargo target output or a target directory beside an indexed Cargo.toml. Debug objects, fingerprints, build-script outputs and incremental state can accumulate many small files.","Rebuilding can take time and require toolchains or dependencies. Project and Git/worktree validation is still required before any supported cleanup.")
    } else if has("node_modules") {
        ("dependencies","JavaScript dependencies","The node_modules path identifies an installed dependency tree. Numerous small packages can occupy substantial allocated space.","Dependencies may include local changes, offline-only packages or native builds. Use the project package manager; identification alone does not permit removal.")
    } else if has(".npm")
        || has(".pnpm-store")
        || path.contains("/.cache/pip/")
        || path.contains("/.cache/uv/")
        || path.contains("/Caches/Homebrew")
        || path.contains("/Caches/Yarn")
    {
        ("packageCache","Package manager resources","The indexed path matches a named package-manager cache or content-addressed package store, where many small objects can add up.","These resources may be shared across projects or needed offline. Use the owning package manager's maintenance controls; shared stores are not ordinary project build output.")
    } else if has(".next")
        || has(".nuxt")
        || has(".vite")
        || has(".parcel-cache")
        || has(".turbo")
        || (project == Some("javascript") && (has("dist") || has("build")))
    {
        ("webBuild","Web build and tool output","The indexed path matches a web tool cache/output name, or build/dist inside an indexed JavaScript project.","Output can be needed for deployment or offline use. Inspect the project and tool configuration before a separately supported cleanup review.")
    } else if has("__pycache__") || has(".pytest_cache") || has(".mypy_cache") || has(".ruff_cache")
    {
        ("pythonCache","Python tool cache","The indexed path matches Python bytecode or a named Python tool cache.","Tools may regenerate this data and the next run may be slower. Path identification does not verify that every descendant is disposable.")
    } else if has(".venv")
        || has("venv")
        || project == Some("python")
        || (has("envs") && (has("miniconda3") || has("anaconda3") || has("miniforge3")))
    {
        ("environment","Python environment","The path uses a common virtual-environment name or contains an indexed pyvenv.cfg context.","Environments contain installed packages and may contain local changes. Recreating them can require network access and exact dependencies; use their owning tool.")
    } else if path.contains("/Library/Developer/Xcode/DerivedData") {
        ("xcodeBuild","Xcode Derived Data","The indexed path matches Xcode's DerivedData location, which holds build products, indexes and intermediate files.","Xcode may need to rebuild and reindex. Simulator runtimes, device support and archives are separate resources and are not inferred disposable.")
    } else if has(".gradle") || (project == Some("gradle") && has("build")) {
        ("gradleBuild","Gradle cache and output","The indexed path matches .gradle, or a build directory in an indexed Gradle project.","Offline dependencies, local state and outputs may be required. Use Gradle/project review and preserve project sources.")
    } else if project == Some("unity") && (has("Library") || has("Temp") || has("obj")) {
        ("unityBuild","Unity generated project data","The path matches a common Unity generated-data directory beside an indexed project marker.","Reimporting assets may be expensive and require the project's exact Unity version. Project validation remains necessary.")
    } else if entry.category == "Virtual machines"
        || path.ends_with(".vmwarevm")
        || path.ends_with(".pvm")
    {
        ("virtualMachine","Virtual machine data","The indexed path or disk-image format identifies a virtual-machine resource. Sparse images can have a much larger logical size than allocated space.","This may contain an entire operating system, containers and personal data. Manage it in the owning VM/container application; do not remove individual disk segments.")
    } else if parts.iter().any(|part| part.ends_with(".photoslibrary")) {
        ("photoLibrary","Photo library","The indexed path is inside a macOS Photos library package, which combines originals, thumbnails, derivatives and databases.","This can contain original photographs and edits. Use Photos and its library/storage controls; individual package files are not standalone cleanup candidates.")
    } else if has("models")
        && (path.contains("/ComfyUI")
            || path.contains("/.lmstudio/")
            || path.contains("/.ollama/")
            || path.ends_with(".safetensors")
            || path.ends_with(".gguf"))
        || path.ends_with(".safetensors")
        || path.ends_with(".gguf")
    {
        ("models","AI model resources","The indexed path or model file format identifies downloaded model resources.","Models may be difficult or costly to download again and workflows can depend on exact versions. Keep the owning application and workflow requirements in mind.")
    } else if path.ends_with(".sqlite")
        || path.ends_with(".sqlite3")
        || path.ends_with(".db")
        || has(".git")
    {
        ("database","Application or repository state","The indexed filename/path matches a database or Git repository state. Small records can accumulate into large retained stores.","This can contain history, indexes or irreplaceable application/repository data. Use the owning application's retention or maintenance controls.")
    } else if has("Caches") || path.contains("/private/var/tmp/") {
        ("cache","Application cache location","The indexed path matches a conventional cache or temporary location. The name describes origin, not safe disposal.","Caches may include unsaved work, offline data and active application state. Review exact items and consequences in the existing cleanup workflow.")
    } else if metrics.is_some_and(|m| {
        m.file_count >= 128
            && m.small_file_count as f64 / m.file_count as f64 >= 0.8
            && m.small_allocated_bytes >= 16. * 1024. * 1024.
    }) {
        ("manySmallFiles","Many small files","At least 80% of inspected descendant files are 64 KiB or smaller, with at least 16 MiB allocated together. Aggregate size reveals storage that a large-file list misses.","Small size and high file count do not establish disposability. Inspect ownership and folder purpose before any removal review.")
    } else {
        (if entry.is_directory {"folder"} else {"file"},if entry.is_directory {"Folder allocation"} else {"File allocation"},"Measured metadata from the inspected scope; folder allocation includes indexed descendants.","Size and age do not establish disposability. Review exact paths and the owning application's requirements.")
    };
    StorageMapNode {
        qualification: if entry.is_directory && metrics.is_none() {
            "Descendant counts unavailable for this legacy or unfinished folder record.".into()
        } else {
            "Observed path/metadata interpretation; not a cleanup eligibility decision.".into()
        },
        file_count: metrics.map(|m| m.file_count as f64),
        small_file_count: metrics.map(|m| m.small_file_count as f64),
        small_allocated_bytes: metrics.map(|m| m.small_allocated_bytes),
        largest_file_bytes: metrics.map(|m| m.largest_file_bytes),
        entry,
        kind: kind.into(),
        title: title.into(),
        reason: reason.into(),
        consequences: consequences.into(),
        development: crate::development::path_indicators(&path, kind),
    }
}
/// Location context, not a diagnosis of which macOS enforcement mechanism denied access.
pub fn permission_guidance(path: &str, reason: &str) -> Option<String> {
    if reason != "permissionDenied" {
        return None;
    }
    let path = Path::new(
        path.strip_prefix("/System/Volumes/Data")
            .filter(|s| s.starts_with('/'))
            .unwrap_or(path),
    );
    let system = [
        "/System",
        "/private/var/db",
        "/private/var/protected",
        "/private/var/networkd",
        "/private/var/OOPJit",
        "/private/var/folders/zz",
    ]
    .iter()
    .any(|p| path.starts_with(p));
    Some(if system {
        "macOS system/service location. Full Disk Access and administrator rights do not override every macOS restriction. If the paired helper already has access and this remains denied, keep the omission; do not change ownership, permissions or system protections to make the scan pass. The saved reason alone does not identify the exact protection."
    } else {
        "Review Full Disk Access for the process performing the scan (the paired helper for administrator scans), and the folder owner's access settings. Retry only after an intended access change. A denial does not prove the folder is empty; administrator rights do not override all protections."
    }.into())
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
    progress: P,
    gap: G,
) -> Result<WalkSummary> {
    walk_indexed(
        roots,
        exclusions,
        cancel,
        false,
        &std::env::temp_dir(),
        |item| emit(item.entry),
        progress,
        gap,
    )
}

/// Production collection emits initial folder context, then replaces it with final aggregates.
pub fn walk_indexed_with_gaps<
    F: FnMut(IndexedStorageEntry) -> Result<()>,
    P: FnMut(&WalkSummary) -> Result<()>,
    G: FnMut(CoverageGap) -> Result<()>,
>(
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    emit: F,
    progress: P,
    gap: G,
) -> Result<WalkSummary> {
    walk_indexed(
        roots,
        exclusions,
        cancel,
        true,
        &std::env::temp_dir(),
        emit,
        progress,
        gap,
    )
}

#[derive(Debug)]
pub(crate) struct ScanDatabaseStep {
    name: &'static str,
    system_errno: i32,
}
impl std::fmt::Display for ScanDatabaseStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (OS error {})", self.name, self.system_errno)
    }
}
pub(crate) fn scan_sql<T, E: Into<anyhow::Error>>(
    connection: &rusqlite::Connection,
    name: &'static str,
    result: std::result::Result<T, E>,
) -> Result<T> {
    result.map_err(|error| {
        // Capture SQLite's connection-local OS error before another statement runs.
        // Persist only the step and numeric errno, never a raw temporary/user path.
        let system_errno = unsafe { rusqlite::ffi::sqlite3_system_errno(connection.handle()) };
        error
            .into()
            .context(ScanDatabaseStep { name, system_errno })
    })
}

fn walk_indexed<
    F: FnMut(IndexedStorageEntry) -> Result<()>,
    P: FnMut(&WalkSummary) -> Result<()>,
    G: FnMut(CoverageGap) -> Result<()>,
>(
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    initial_folders: bool,
    scratch_directory: &Path,
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
        Scratch(scratch_directory.join(format!("coqui-walk-{}.db", uuid::Uuid::new_v4())));
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&scratch.0)?;
    let scratch_identity = fs::symlink_metadata(&scratch.0)?;
    let sql = Connection::open(&scratch.0)?;
    scan_sql(&sql, "Creating the temporary traversal index", sql.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA cache_size=-8192; PRAGMA temp_store=FILE; CREATE TABLE identities(dev INTEGER,ino INTEGER,PRIMARY KEY(dev,ino)) WITHOUT ROWID; CREATE TABLE folders(path TEXT PRIMARY KEY,parent TEXT,depth INTEGER,json TEXT,logical REAL DEFAULT 0,allocated REAL DEFAULT 0,file_count INTEGER DEFAULT 0,small_count INTEGER DEFAULT 0,small_allocated REAL DEFAULT 0,largest REAL DEFAULT 0); CREATE INDEX folder_depth ON folders(depth DESC,path); CREATE TABLE pending(path TEXT PRIMARY KEY,parent_dev INTEGER,parent_ino INTEGER);"))?;
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
                guidance: permission_guidance(
                    &crate::volumes::friendly(path).to_string_lossy(),
                    reason,
                ),
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
            let next: Option<(String, u64, u64)> = scan_sql(
                &sql,
                "Reading the directory frontier",
                sql.query_row(
                    "SELECT path,parent_dev,parent_ino FROM pending LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional(),
            )?;
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
            if exclusions
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
            // The same temp file can be reached through /var, /private/var or a
            // Data-volume alias. Never count the scanner's growing scratch index.
            if metadata.dev() == scratch_identity.dev() && metadata.ino() == scratch_identity.ino()
            {
                note(&mut summary, p, "excluded")?;
                continue;
            }
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
            if scan_sql(
                &sql,
                "Recording a filesystem identity",
                sql.execute(
                    "INSERT OR IGNORE INTO identities VALUES(?1,?2)",
                    params![metadata.dev(), metadata.ino()],
                ),
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
                scan_sql(
                    &sql,
                    "Recording a folder",
                    sql.execute(
                        "INSERT OR IGNORE INTO folders(path,parent,depth,json) VALUES(?1,?2,?3,?4)",
                        params![
                            path,
                            parent,
                            p.components().count(),
                            serde_json::to_string(&f)?
                        ],
                    ),
                )?;
                if initial_folders {
                    let mut pending = f.clone();
                    pending.qualification = "Folder aggregation is unfinished; displayed bytes do not yet include descendants.".into();
                    emit(IndexedStorageEntry {
                        entry: pending,
                        metrics: None,
                    })?;
                }
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
                                        scan_sql(
                                            &sql,
                                            "Extending the directory frontier",
                                            sql.execute(
                                                "INSERT OR IGNORE INTO pending VALUES(?1,?2,?3)",
                                                params![text, metadata.dev(), metadata.ino()],
                                            ),
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
                            let path = crate::volumes::friendly(p).to_string_lossy().into_owned();
                            summary.permissions.push(PermissionRequirement {
                                guidance: permission_guidance(&path, "permissionDenied")
                                    .unwrap_or_default(),
                                path,
                                reason: format!(
                                    "macOS denied directory access (OS error {})",
                                    e.raw_os_error()
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|| "unavailable".into())
                                ),
                            });
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
            scan_sql(&sql, "Accumulating file measurements", sql.execute(
                "UPDATE folders SET logical=logical+?1,allocated=allocated+?2,file_count=file_count+1,small_count=small_count+?4,small_allocated=small_allocated+?5,largest=MAX(largest,?2) WHERE path=?3",
                params![f.logical_bytes, f.allocated_bytes, parent,u32::from(f.logical_bytes<=65_536.),if f.logical_bytes<=65_536. {f.allocated_bytes} else {0.}],
            ))?;
            emit(IndexedStorageEntry {
                entry: f,
                metrics: None,
            })
            .context("Could not persist scanned file")?;
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
        "Scan cancelled; folder aggregation is unfinished"
    } else {
        "Aggregating inspected folder allocation"
    }
    .into();
    progress(&summary)?;
    // A cancelled scan must not spend minutes finalizing millions of folder records.
    // Initial folder context remains indexed with unavailable descendant measurements.
    if cancel.load(Ordering::Relaxed) {
        summary.categories = cats.into_values().collect();
        return Ok(summary);
    }
    sql.execute_batch("BEGIN")?;
    let mut q = scan_sql(
        &sql,
        "Preparing folder aggregation",
        sql.prepare("SELECT json,logical,allocated FROM folders ORDER BY depth DESC,path"),
    )?;
    let mut rows = q.query([])?;
    while let Some(row) = rows.next()? {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let mut f: FileEntry = serde_json::from_str(&row.get::<_, String>(0)?)?;
        // Read the current total because children updated it after this ordered cursor began.
        let (logical,allocated,metrics) = scan_sql(&sql, "Reading folder totals", sql.query_row(
            "SELECT logical,allocated,file_count,small_count,small_allocated,largest FROM folders WHERE path=?1",
            [&f.path],
            |r| Ok((r.get(0)?,r.get(1)?,StorageMetrics {file_count:r.get(2)?,small_file_count:r.get(3)?,small_allocated_bytes:r.get(4)?,largest_file_bytes:r.get(5)?})),
        ))?;
        f.logical_bytes = logical;
        f.allocated_bytes = allocated;
        scan_sql(&sql, "Aggregating folder totals", sql.execute(
            "UPDATE folders SET logical=logical+?1,allocated=allocated+?2,file_count=file_count+?4,small_count=small_count+?5,small_allocated=small_allocated+?6,largest=MAX(largest,?7) WHERE path=?3",
            params![f.logical_bytes,f.allocated_bytes,f.parent,metrics.file_count,metrics.small_file_count,metrics.small_allocated_bytes,metrics.largest_file_bytes],
        ))?;
        if f.path.ends_with("/Library/Caches") || f.path.ends_with("/private/var/tmp") {
            if summary.candidates.len() < 500 {
                summary.candidates.push(CleanupCandidate{path:f.path.clone(),allocated_bytes:f.allocated_bytes,logical_bytes:f.logical_bytes,modified_at:f.modified_at.clone(),origin:"Known cache/temporary directory".into(),reason:"Directory name and size alone do not establish disposability.".into(),safety:"Review required".into(),consequences:"Applications may regenerate contents; removal can discard useful data or slow subsequent work.".into()});
            }
        }
        emit(IndexedStorageEntry {
            entry: f,
            metrics: Some(metrics),
        })?;
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

#[cfg(test)]
#[test]
fn temporary_index_is_excluded_when_its_directory_has_another_path() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("contents");
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    fs::write(root.join("keep.txt"), b"keep").unwrap();
    let alias = fixture.path().join("temporary-alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let mut gaps = vec![];
    let summary = walk_indexed(
        std::slice::from_ref(&root),
        &[],
        &AtomicBool::new(false),
        false,
        &alias,
        |_| Ok(()),
        |_| Ok(()),
        |gap| {
            gaps.push(gap);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(summary.files, 1);
    assert_eq!(summary.logical, 4.);
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].reason, "excluded");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    assert_eq!(fs::read(root.join("keep.txt")).unwrap(), b"keep");
}
