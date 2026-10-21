use crate::{model::*, platform};
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use std::os::darwin::fs::MetadataExt as DarwinMetadataExt;
use std::{
    collections::{HashMap, HashSet},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct WalkSummary {
    pub files: u32,
    pub logical: f64,
    pub allocated: f64,
    pub skipped: u32,
    pub permissions: Vec<PermissionRequirement>,
    pub errors: Vec<String>,
    pub categories: Vec<StorageCategory>,
    pub candidates: Vec<CleanupCandidate>,
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
    roots.sort_by_key(|p| p.components().count());
    let mut result: Vec<PathBuf> = vec![];
    for p in roots {
        if !result.iter().any(|r| p.starts_with(r)) {
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
        "deep" => vec![if Path::new("/System/Volumes/Data").is_dir() {
            PathBuf::from("/System/Volumes/Data")
        } else {
            PathBuf::from("/")
        }],
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

pub fn walk<F: FnMut(FileEntry) -> Result<()>, P: FnMut(&WalkSummary) -> Result<()>>(
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    mut emit: F,
    mut progress: P,
) -> Result<WalkSummary> {
    let mut summary = WalkSummary::default();
    let mut identities = HashSet::new();
    let mut folders: HashMap<PathBuf, FileEntry> = HashMap::new();
    let mut cats: HashMap<String, StorageCategory> = HashMap::new();
    for root in roots {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let root_dev = match fs::symlink_metadata(root) {
            Ok(m) => m.dev(),
            Err(_) => {
                summary.skipped = summary.skipped.saturating_add(1);
                summary.errors.push(format!(
                    "Scan root {} became unavailable during analysis.",
                    root.display()
                ));
                continue;
            }
        };
        let mut it = walkdir::WalkDir::new(root).follow_links(false).into_iter();
        while let Some(next) = it.next() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let entry = match next {
                Ok(v) => v,
                Err(e) => {
                    summary.skipped = summary.skipped.saturating_add(1);
                    let path = e
                        .path()
                        .and_then(Path::to_str)
                        .unwrap_or("Unrepresentable path")
                        .to_owned();
                    if e.io_error()
                        .is_some_and(|x| x.kind() == std::io::ErrorKind::PermissionDenied)
                    {
                        if summary.permissions.len() < 100 {
                            summary.permissions.push(PermissionRequirement{path,reason:"macOS denied access".into(),guidance:"Review Privacy & Security → Full Disk Access or folder permissions. Grant access only if you want this location included, then retry; otherwise exclude it.".into()});
                        }
                    } else if summary.errors.len() < 100 {
                        summary.errors.push(format!("Could not inspect {path}; the location may have changed during scanning."));
                    }
                    continue;
                }
            };
            let p = entry.path();
            if exclusions.iter().any(|x| p.starts_with(x)) {
                if entry.file_type().is_dir() {
                    it.skip_current_dir();
                }
                summary.skipped = summary.skipped.saturating_add(1);
                continue;
            }
            let metadata = match fs::symlink_metadata(p) {
                Ok(m) => m,
                Err(_) => {
                    summary.skipped = summary.skipped.saturating_add(1);
                    if summary.errors.len() < 100 {
                        summary.errors.push(
                            "An entry disappeared or became unreadable during traversal.".into(),
                        );
                    }
                    continue;
                }
            };
            if metadata.dev() != root_dev
                || metadata.file_type().is_symlink()
                || placeholder(p, &metadata)
            {
                if metadata.is_dir() {
                    it.skip_current_dir();
                }
                summary.skipped = summary.skipped.saturating_add(1);
                continue;
            }
            let Some(path) = p.to_str() else {
                summary.skipped = summary.skipped.saturating_add(1);
                if metadata.is_dir() {
                    it.skip_current_dir();
                }
                if summary.errors.len() < 100 {
                    summary.errors.push("A filename cannot be represented as Unicode; it was skipped rather than changing its path.".into());
                }
                continue;
            };
            let parent = if p == root {
                "".to_owned()
            } else {
                p.parent().and_then(Path::to_str).unwrap_or("").to_owned()
            };
            let mut f = FileEntry {
                path: path.into(),
                parent,
                name: p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(path)
                    .into(),
                category: category(p).into(),
                logical_bytes: 0.,
                allocated_bytes: 0.,
                is_directory: metadata.is_dir(),
                modified_at: modified(&metadata),
                qualification: String::new(),
            };
            if metadata.is_dir() {
                folders.insert(p.to_owned(), f);
                continue;
            }
            if !metadata.is_file() {
                continue;
            }
            if !identities.insert((metadata.dev(), metadata.ino())) {
                summary.skipped = summary.skipped.saturating_add(1);
                continue;
            }
            f.logical_bytes = metadata.len() as f64;
            f.allocated_bytes = metadata.blocks() as f64 * 512.;
            summary.files = summary.files.saturating_add(1);
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
            for ancestor in p.ancestors().skip(1).take_while(|a| a.starts_with(root)) {
                if let Some(d) = folders.get_mut(ancestor) {
                    d.logical_bytes += f.logical_bytes;
                    d.allocated_bytes += f.allocated_bytes;
                }
            }
            emit(f).context("Could not persist scanned file")?;
            if summary.files.is_multiple_of(250) {
                progress(&summary)?;
            }
        }
    }
    for f in folders.into_values() {
        if f.path.ends_with("/Library/Caches") || f.path.ends_with("/private/var/tmp") {
            summary.candidates.push(CleanupCandidate{path:f.path.clone(),allocated_bytes:f.allocated_bytes,logical_bytes:f.logical_bytes,modified_at:f.modified_at.clone(),origin:"Known cache/temporary directory".into(),reason:"Applications may regenerate some contents. Directory size alone does not establish disposability.".into(),safety:"Review required; analysis only".into(),consequences:"Future removal may log you out, discard useful cached data or slow subsequent work. No files have been selected or deleted.".into()});
        }
        emit(f)?;
    }
    summary.categories = cats.into_values().collect();
    summary
        .categories
        .sort_by(|a, b| b.allocated_bytes.total_cmp(&a.allocated_bytes));
    progress(&summary)?;
    Ok(summary)
}
