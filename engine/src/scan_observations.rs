//! Explicit foreground metadata checks associated with a new storage scan.
use crate::{applications::ApplicationFootprint, model::*, records};
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const KIND: &str = "scanObservations";
const ENTRY_LIMIT: usize = 5_000;
const METADATA_BYTES_LIMIT: usize = 32 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanModules {
    pub system: bool,
    pub startup: bool,
    pub applications: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanCheck {
    pub module: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub count: Option<u32>,
    pub omissions: u32,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScanObservations {
    pub id: String,
    pub session_id: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub checks: Vec<ScanCheck>,
    pub system: Option<MachineSnapshot>,
    pub startup_items: Vec<StartupItem>,
    pub applications: Vec<ApplicationFootprint>,
    pub qualification: String,
}
pub fn begin(session_id: &str, modules: &ScanModules) -> ScanObservations {
    ScanObservations {
        id: format!("scan-observations-{session_id}"), session_id: session_id.into(),
        created_at: now(), finished_at: None, status: "running".into(),
        checks: [("system", modules.system), ("startup", modules.startup), ("applications", modules.applications)]
            .into_iter().map(|(name, selected)| ScanCheck { module: name.into(),
                status: if selected { "pending" } else { "notSelected" }.into(),
                started_at: None, finished_at: None, count: None, omissions: 0,
                qualification: if selected { "Waiting for this scan's explicitly selected metadata check." } else { "Not selected for this scan; no collection was requested." }.into(),
            }).collect(),
        system: None, startup_items: vec![], applications: vec![],
        qualification: "Foreground metadata observations from this scan, collected by the app without administrator privileges. Dates and omissions are independent of storage traversal. Content duplicates, similar photos, online updates and system changes are separate reviews.".into(),
    }
}
pub fn save(c: &Connection, report: &ScanObservations) -> Result<()> {
    records::save(
        c,
        KIND,
        &report.id,
        &report.created_at,
        &report.status,
        "Scan metadata checks",
        report,
    )
}
pub fn read(c: &Connection, session: &str) -> Result<Option<ScanObservations>> {
    let value: Option<String> = c
        .query_row(
            "SELECT json FROM records WHERE kind=?1 AND id=?2",
            rusqlite::params![KIND, format!("scan-observations-{session}")],
            |row| row.get(0),
        )
        .optional()?;
    value
        .map(|text| {
            let mut report: ScanObservations = serde_json::from_str(&text)?;
            if report.status == "interrupted" {
                for check in &mut report.checks {
                    if ["pending", "running"].contains(&check.status.as_str()) {
                        check.status = "interrupted".into();
                        check.qualification = "Application closed before this check completed; no completed observation date is recorded.".into();
                    }
                }
            }
            Ok(report)
        })
        .transpose()
}
pub fn finish(report: &mut ScanObservations, cancelled: bool) {
    for check in &mut report.checks {
        if check.status == "pending" || check.status == "running" {
            let attempted = check.started_at.is_some();
            check.status = if cancelled {
                "cancelled"
            } else {
                "unavailable"
            }
            .into();
            check.finished_at = attempted.then(now);
            check.qualification = "No completed collection was established for this check.".into();
        }
    }
    report.status = if cancelled {
        "cancelled"
    } else if report
        .checks
        .iter()
        .any(|c| !["complete", "notSelected"].contains(&c.status.as_str()))
    {
        "partial"
    } else {
        "complete"
    }
    .into();
    report.finished_at = Some(now());
    if let Some(system) = &mut report.system {
        system.startup_items = report.startup_items.clone();
        system.applications = report
            .applications
            .iter()
            .map(|app| InstalledApplication {
                name: app.name.clone(),
                path: app.path.clone(),
            })
            .collect();
    }
}

/// A missing/denied path or a reached bound is an omission, not a completed zero.
pub fn metadata_paths(
    roots: &[PathBuf],
    applications: bool,
    cancel: &AtomicBool,
) -> (Vec<PathBuf>, u32, bool) {
    metadata_paths_excluding(roots, applications, cancel, &[])
}
fn metadata_paths_excluding(
    roots: &[PathBuf],
    applications: bool,
    cancel: &AtomicBool,
    exclusions: &[PathBuf],
) -> (Vec<PathBuf>, u32, bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut paths = vec![];
    let mut omissions = 0u32;
    let mut visited = 0;
    for root in roots {
        if exclusions.iter().any(|excluded| root.starts_with(excluded)) {
            omissions += 1;
            continue;
        }
        if cancel.load(Ordering::SeqCst) {
            return (paths, omissions, false);
        }
        match std::fs::symlink_metadata(root) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => (),
            _ => {
                omissions += 1;
                continue;
            }
        }
        let mut walk = walkdir::WalkDir::new(root)
            .max_depth(if applications { 3 } else { 1 })
            .follow_links(false)
            .into_iter();
        while let Some(entry) = walk.next() {
            if cancel.load(Ordering::SeqCst) {
                return (paths, omissions, false);
            }
            visited += 1;
            if visited > ENTRY_LIMIT || Instant::now() >= deadline {
                return (paths, omissions + 1, false);
            }
            match entry {
                Ok(entry)
                    if exclusions
                        .iter()
                        .any(|excluded| entry.path().starts_with(excluded)) =>
                {
                    if entry.file_type().is_dir() {
                        walk.skip_current_dir();
                    }
                    omissions += 1;
                }
                Ok(entry)
                    if entry
                        .path()
                        .extension()
                        .is_some_and(|e| e == if applications { "app" } else { "plist" }) =>
                {
                    if applications {
                        walk.skip_current_dir();
                    }
                    if entry.file_type().is_symlink()
                        || entry.path().to_str().is_none()
                        || (applications
                            && exclusions.iter().any(|excluded| {
                                entry
                                    .path()
                                    .join("Contents/Info.plist")
                                    .starts_with(excluded)
                            }))
                    {
                        omissions += 1;
                    } else {
                        paths.push(entry.path().to_path_buf());
                    }
                }
                Ok(_) => (),
                Err(_) => omissions += 1,
            }
        }
    }
    paths.sort();
    paths.dedup();
    (paths, omissions, true)
}

fn startup_plist(path: &Path) -> Result<(plist::Value, usize)> {
    use std::{
        io::Read,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::MetadataExt,
        },
    };
    let before = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        before.is_file()
            && !before.file_type().is_symlink()
            && before.len() <= 1024 * 1024
            && !crate::storage::placeholder(path, &before),
        "Unreadable metadata"
    );
    #[cfg(target_os = "macos")]
    let file = {
        let (parent, name) = crate::safe_fs::parent(path)?;
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        anyhow::ensure!(fd >= 0, "Metadata unavailable");
        unsafe { std::fs::File::from_raw_fd(fd) }
    };
    #[cfg(not(target_os = "macos"))]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)?
    };
    let opened = file.metadata()?;
    anyhow::ensure!(
        opened.is_file()
            && opened.dev() == before.dev()
            && opened.ino() == before.ino()
            && !crate::storage::placeholder(path, &opened),
        "Metadata changed"
    );
    let mut data = vec![];
    (&file).take(1024 * 1024 + 1).read_to_end(&mut data)?;
    let after = file.metadata()?;
    anyhow::ensure!(
        data.len() <= 1024 * 1024
            && after.len() == opened.len()
            && after.mtime() == opened.mtime()
            && after.mtime_nsec() == opened.mtime_nsec()
            && after.ctime() == opened.ctime()
            && after.ctime_nsec() == opened.ctime_nsec(),
        "Metadata changed"
    );
    let bytes = data.len();
    Ok((
        plist::Value::from_reader(std::io::Cursor::new(data))?,
        bytes,
    ))
}

pub fn collect(
    report: &mut ScanObservations,
    cancel: &AtomicBool,
    checkpoint: impl FnMut(&ScanObservations) -> bool,
) -> Result<()> {
    collect_excluding(report, cancel, &[], checkpoint)
}
pub fn collect_excluding(
    report: &mut ScanObservations,
    cancel: &AtomicBool,
    excluded: &[String],
    mut checkpoint: impl FnMut(&ScanObservations) -> bool,
) -> Result<()> {
    let user_home = crate::platform::home()?;
    let exclusions = excluded
        .iter()
        .map(|path| {
            crate::volumes::friendly(
                &Path::new(path)
                    .canonicalize()
                    .unwrap_or_else(|_| PathBuf::from(path)),
            )
        })
        .collect::<Vec<_>>();
    for index in 0..report.checks.len() {
        if report.checks[index].status != "pending" {
            continue;
        }
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        report.checks[index].status = "running".into();
        report.checks[index].started_at = Some(now());
        if !checkpoint(report) {
            cancel.store(true, Ordering::SeqCst);
            break;
        }
        let module = report.checks[index].module.clone();
        let result = if module == "system" {
            crate::platform::scan_system(cancel).map(|snapshot| {
                let complete = !snapshot
                    .capabilities
                    .iter()
                    .any(|capability| capability.state == CapabilityState::TemporarilyUnavailable);
                report.system = Some(snapshot);
                (None, if complete { 0 } else { 1 }, complete)
            })
        } else {
            let roots = if module == "startup" {
                vec![
                    user_home.join("Library/LaunchAgents"),
                    PathBuf::from("/Library/LaunchAgents"),
                    PathBuf::from("/Library/LaunchDaemons"),
                ]
            } else {
                vec![
                    PathBuf::from("/Applications"),
                    user_home.join("Applications"),
                    PathBuf::from("/System/Applications"),
                ]
            };
            let (paths, mut omitted, mut finished) =
                metadata_paths_excluding(&roots, module == "applications", cancel, &exclusions);
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut metadata_bytes = 0usize;
            for path in paths {
                if cancel.load(Ordering::SeqCst)
                    || Instant::now() >= deadline
                    || metadata_bytes >= METADATA_BYTES_LIMIT
                {
                    finished = false;
                    break;
                }
                let result = if module == "startup" {
                    startup_plist(&path)
                        .and_then(|(value, bytes)| {
                            metadata_bytes += bytes;
                            anyhow::ensure!(
                                metadata_bytes <= METADATA_BYTES_LIMIT,
                                "Metadata byte limit reached"
                            );
                            let item = crate::platform::startup_from_plist(&path, &value)
                                .context("Launch configuration unavailable")?;
                            anyhow::ensure!(
                                item.name.len() <= 255
                                    && item.path.len() <= 4096
                                    && !item.name.chars().any(char::is_control),
                                "Launch metadata outside limits"
                            );
                            Ok(item)
                        })
                        .map(|item| report.startup_items.push(item))
                } else {
                    crate::applications::info_bytes(&path).and_then(|data| { metadata_bytes += data.len(); anyhow::ensure!(metadata_bytes <= METADATA_BYTES_LIMIT, "Metadata byte limit reached"); Ok(plist::Value::from_reader(std::io::Cursor::new(data))?) }).and_then(|value| {
                        let dictionary = value.as_dictionary().context("Invalid bundle metadata")?;
                        let field = |key| dictionary.get(key).and_then(plist::Value::as_string).unwrap_or("").chars().filter(|c| !c.is_control()).take(255).collect::<String>();
                        let bundle_id = field("CFBundleIdentifier");
                        anyhow::ensure!(!bundle_id.is_empty(), "Missing bundle identity");
                        let name = field("CFBundleDisplayName");
                        report.applications.push(ApplicationFootprint { path: path.to_string_lossy().into_owned(), name: if name.is_empty() { path.file_stem().unwrap_or_default().to_string_lossy().into_owned() } else { name }, bundle_id, version: field("CFBundleShortVersionString"), observed_at: now(), protection: vec!["Scan metadata only; running state and removal eligibility require fresh inspection.".into()], related: vec![], qualification: "Read during the selected scan application check. No signature, updater, running-state, associated-data or removal assessment was performed.".into() });
                        Ok(())
                    })
                };
                if result.is_err() {
                    omitted += 1;
                }
            }
            Ok((
                Some(if module == "startup" {
                    report.startup_items.len()
                } else {
                    report.applications.len()
                } as u32),
                omitted,
                finished,
            ))
        };
        let check = &mut report.checks[index];
        check.finished_at = Some(now());
        match result {
            Ok((count, omissions, finished)) => {
                check.count = count;
                check.omissions = omissions;
                check.status = if cancel.load(Ordering::SeqCst) {
                    "cancelled"
                } else if omissions > 0 || !finished {
                    "partial"
                } else {
                    "complete"
                }
                .into();
                check.qualification = match module.as_str() {
                    "system" => "Current readable system details. Unsupported sensors and hardware fields stay unavailable; this is not a benchmark or firmware audit.",
                    "startup" => "Readable launch plist configurations only, within entry/time/read bounds. Modern registered or managed login items, runtime state and startup delay require manual review. No entries were changed.",
                    _ => "Readable application metadata in system, shared and user Applications folders, up to three levels and within entry/time bounds. Omitted or outside-scope apps remain unknown. No online version check or removal assessment was performed.",
                }.into();
            }
            Err(_) => {
                check.status = if cancel.load(Ordering::SeqCst) {
                    "cancelled"
                } else {
                    "unavailable"
                }
                .into();
                check.qualification = "This metadata check could not complete. Missing readings are unavailable; no healthy/empty conclusion follows.".into();
            }
        }
        if !checkpoint(report) {
            cancel.store(true, Ordering::SeqCst);
            break;
        }
    }
    finish(report, cancel.load(Ordering::SeqCst));
    let _ = checkpoint(report);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    #[test]
    fn bounded_plist_read_rejects_links_oversize_and_nonregular_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("fixture.plist");
        std::fs::write(&path, b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>Label</key><string>fixture</string></dict></plist>").unwrap();
        assert!(startup_plist(&path).is_ok());
        symlink(&path, root.join("linked.plist")).unwrap();
        assert!(startup_plist(&root.join("linked.plist")).is_err());
        assert!(startup_plist(&root).is_err());
        std::fs::write(&path, vec![b'x'; 1024 * 1024 + 1]).unwrap();
        assert!(startup_plist(&path).is_err());
    }
    #[test]
    fn enumeration_treats_bundles_as_leaves_and_stops_on_cancel_and_limit() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = root.join("Example.app");
        std::fs::create_dir_all(app.join("Contents/Nested.app")).unwrap();
        let cancel = AtomicBool::new(false);
        let (paths, omitted, finished) = metadata_paths(std::slice::from_ref(&root), true, &cancel);
        assert_eq!(paths, vec![app.clone()]);
        assert_eq!(omitted, 0);
        assert!(finished);
        let (paths, omitted, finished) = metadata_paths_excluding(
            std::slice::from_ref(&root),
            true,
            &cancel,
            std::slice::from_ref(&app),
        );
        assert!(paths.is_empty());
        assert_eq!(omitted, 1);
        assert!(finished);
        cancel.store(true, Ordering::SeqCst);
        let (paths, _, finished) = metadata_paths(std::slice::from_ref(&root), true, &cancel);
        assert!(paths.is_empty());
        assert!(!finished);
        cancel.store(false, Ordering::SeqCst);
        for n in 0..ENTRY_LIMIT {
            std::fs::write(root.join(format!("entry-{n}")), []).unwrap();
        }
        let (_, omitted, finished) = metadata_paths(&[root], true, &cancel);
        assert!(omitted > 0);
        assert!(!finished);
    }

    #[test]
    fn bundle_metadata_exclusions_cover_contents_subtree_and_exact_plist() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let app = root.join("Fixture.app");
        let contents = app.join("Contents");
        std::fs::create_dir_all(&contents).unwrap();
        let info = contents.join("Info.plist");
        let bytes = b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleIdentifier</key><string>fixture.app</string></dict></plist>";
        std::fs::write(&info, bytes).unwrap();
        let cancel = AtomicBool::new(false);
        let (paths, omitted, finished) = metadata_paths(std::slice::from_ref(&root), true, &cancel);
        assert_eq!(paths, vec![app]);
        assert_eq!(omitted, 0);
        assert!(finished);
        for excluded in [contents, info.clone()] {
            let (paths, omitted, finished) =
                metadata_paths_excluding(std::slice::from_ref(&root), true, &cancel, &[excluded]);
            assert!(paths.is_empty());
            assert_eq!(omitted, 1);
            assert!(finished);
        }
        assert_eq!(std::fs::read(&info).unwrap(), bytes);
    }
}
