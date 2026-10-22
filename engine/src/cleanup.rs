//! Approval-bearing cache cleanup. Only indexed regular files beneath this user's
//! Library/Caches are eligible; directories, links and project trees are never targets.
use crate::{database, model::*, storage};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Policy {
    pub home: PathBuf,
    pub exclusions: Vec<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
    size: u64,
    blocks: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
    parent_device: u64,
    parent_inode: u64,
}
#[derive(Serialize, Deserialize)]
struct Record {
    plan: CleanupPlan,
    identities: Vec<Identity>,
}

fn standalone_home_package(path: &Path, expected: &fs::Metadata) -> Result<()> {
    const MAX_BYTES: u64 = 256 * 1024;
    anyhow::ensure!(
        expected.is_file()
            && expected.nlink() == 1
            && expected.uid() == unsafe { libc::geteuid() }
            && !storage::placeholder(path, expected)
            && expected.len() <= MAX_BYTES,
        "Home package.json is linked, cloud-only, oversized or not a user-owned regular file"
    );
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .context("Cannot read home package.json to establish cache scope")?;
    let m = file.metadata()?;
    anyhow::ensure!(
        m.is_file()
            && m.dev() == expected.dev()
            && m.ino() == expected.ino()
            && m.uid() == unsafe { libc::geteuid() }
            && m.nlink() == 1
            && !storage::placeholder(path, &m)
            && m.len() <= MAX_BYTES,
        "Home package.json is changed, linked, oversized or not a user-owned regular file"
    );
    let mut bytes = vec![];
    (&file).take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    anyhow::ensure!(
        after.len() == m.len()
            && after.mtime() == m.mtime()
            && after.mtime_nsec() == m.mtime_nsec()
            && after.ctime() == m.ctime()
            && after.ctime_nsec() == m.ctime_nsec(),
        "Home package.json changed while establishing cache scope"
    );
    anyhow::ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        "Home package.json is oversized"
    );
    // Parse only configuration; never run scripts, resolve packages, follow
    // workspace globs, or retain raw manifest contents in the action record.
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("Cannot establish scope from invalid home package.json"))?;
    let object = value
        .as_object()
        .context("Home package.json is not an object")?;
    anyhow::ensure!(
        !object.contains_key("workspaces"),
        "Home package.json declares workspace configuration; workspace-aware developer analysis is required"
    );
    Ok(())
}

// This context applies only after inspect establishes the dedicated user-cache
// namespace. It never authorizes project-local caches elsewhere in the home.
fn cache_project_context(parent: &Path, home: &Path) -> Result<String> {
    const REPOSITORIES: &[&str] = &[".git", ".hg", ".svn", ".jj"];
    const MANIFESTS: &[&str] = &[
        "Cargo.toml",
        "package.json",
        "pubspec.yaml",
        "pyproject.toml",
        "pnpm-workspace.yaml",
        "lerna.json",
    ];
    let mut home_package = false;
    for ancestor in parent.ancestors().take_while(|p| p.starts_with(home)) {
        for &marker in REPOSITORIES.iter().chain(MANIFESTS) {
            let marker_path = ancestor.join(marker);
            match fs::symlink_metadata(&marker_path) {
                Ok(m) => {
                    // A standalone package directly at home does not establish
                    // ownership of ~/Library/Caches. Repositories and workspace
                    // declarations still protect the entire ancestry.
                    if ancestor == home
                        && marker == "package.json"
                        && m.is_file()
                        && m.uid() == unsafe { libc::geteuid() }
                    {
                        standalone_home_package(&marker_path, &m)?;
                        home_package = true;
                        continue;
                    }
                    anyhow::bail!(
                        "Detected project marker {}. Project-managed files require developer-artifact analysis first",
                        marker_path.display()
                    );
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("Cannot inspect project marker {}", marker_path.display())
                    });
                }
            }
        }
    }
    let mut context = "User Library/Caches · indexed regular file. No supported repository or cache-local project markers detected.".to_string();
    if home_package {
        context.push_str(" Home package.json was parsed with no workspace declaration; it does not establish project ownership of this cache namespace.");
    }
    Ok(context)
}

fn inspect(path: &Path, policy: &Policy) -> Result<(CleanupFile, Identity)> {
    anyhow::ensure!(path.is_absolute(), "An absolute file path is required");
    let home = policy.home.canonicalize()?;
    let base = home.join("Library/Caches");
    anyhow::ensure!(
        path.starts_with(&base) && path != base,
        "Only files in your Library/Caches can be reviewed for this action"
    );
    anyhow::ensure!(
        path.canonicalize()? == path,
        "Linked or non-canonical paths are not eligible"
    );
    anyhow::ensure!(
        !policy
            .exclusions
            .iter()
            .any(|e| path.starts_with(e.canonicalize().unwrap_or(e.clone()))),
        "This location is excluded"
    );
    let m = fs::symlink_metadata(path)?;
    anyhow::ensure!(
        m.is_file() && !m.file_type().is_symlink(),
        "Only regular files are eligible; folders and links are excluded"
    );
    anyhow::ensure!(m.nlink() == 1, "Hard-linked files require separate review");
    anyhow::ensure!(
        m.uid() == unsafe { libc::geteuid() },
        "Only files owned by the current user are eligible"
    );
    anyhow::ensure!(
        !storage::placeholder(path, &m),
        "Cloud-only files are excluded"
    );
    let parent = path.parent().context("File has no parent")?;
    // Reject every symbolic-link ancestor, including a linked cache root.
    for ancestor in parent.ancestors() {
        anyhow::ensure!(
            !fs::symlink_metadata(ancestor)?.file_type().is_symlink(),
            "A parent location became a symbolic link"
        );
    }
    let origin = cache_project_context(parent, &home)?;
    let pm = fs::metadata(parent)?;
    let identity = Identity {
        device: m.dev(),
        inode: m.ino(),
        size: m.len(),
        blocks: m.blocks(),
        modified: m.mtime(),
        modified_ns: m.mtime_nsec(),
        changed: m.ctime(),
        changed_ns: m.ctime_nsec(),
        parent_device: pm.dev(),
        parent_inode: pm.ino(),
    };
    let file = CleanupFile {path: path.to_str().context("This filename cannot be represented")?.into(), allocated_bytes:(m.blocks() as f64)*512., logical_bytes:m.len() as f64, modified_at:m.modified().ok().map(|t|chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()).unwrap_or_default(), origin, consequences:"Close the related application first. Cached data may be useful; the app may need to download or regenerate it and can be slower next time. Moving to Trash does not free disk space until Trash is emptied. Coqui does not empty Trash.".into()};
    Ok((file, identity))
}
fn load(c: &Connection, id: &str) -> Result<Record> {
    let json: String = c
        .query_row("SELECT json FROM cleanup_plans WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .context("Cleanup plan not found")?;
    let record: Record = serde_json::from_str(&json)?;
    anyhow::ensure!(
        record.plan.items.len() == record.identities.len(),
        "This action record is incomplete. No operation can be executed from it."
    );
    Ok(record)
}
fn save(c: &Connection, record: &Record) -> Result<()> {
    c.execute("INSERT INTO cleanup_plans VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET status=excluded.status,json=excluded.json", params![record.plan.id,record.plan.created_at,record.plan.status,serde_json::to_string(record)?])?;
    Ok(())
}
pub fn history(c: &Connection, offset: u32, limit: u32) -> Result<Vec<CleanupPlan>> {
    let mut q =
        c.prepare("SELECT json FROM cleanup_plans ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")?;
    let rows = q.query_map(params![limit.clamp(1, 100), offset], |r| {
        r.get::<_, String>(0)
    })?;
    rows.map(|r| Ok(serde_json::from_str::<Record>(&r?)?.plan))
        .collect()
}
pub fn get(c: &Connection, id: &str) -> Result<CleanupPlan> {
    Ok(load(c, id)?.plan)
}
pub fn candidates(
    c: &Connection,
    session_id: &str,
    offset: u32,
    policy: &Policy,
) -> Result<CleanupFilePage> {
    let s = database::session(c, session_id)?;
    anyhow::ensure!(s.status != "running", "Wait for this scan to finish");
    let rows = database::files(
        c,
        &StorageQuery {
            session_id: session_id.into(),
            parent: None,
            category: Some("Caches".into()),
            sort: "allocated".into(),
            descending: true,
            offset,
            limit: 100,
            files_only: true,
        },
    )?;
    let count = rows.len();
    let files: Vec<_> = rows
        .iter()
        .filter_map(|r| inspect(Path::new(&r.path), policy).ok().map(|v| v.0))
        .collect();
    Ok(CleanupFilePage {
        unavailable_count: (count - files.len()) as u32,
        files,
        next_offset: if count == 100 {
            Some(offset + 100)
        } else {
            None
        },
    })
}
pub fn preview(c: &Connection, request: CleanupRequest, policy: &Policy) -> Result<CleanupPlan> {
    anyhow::ensure!(
        !request.paths.is_empty() && request.paths.len() <= 100,
        "Choose between 1 and 100 cache files"
    );
    let s = database::session(c, &request.session_id)?;
    anyhow::ensure!(
        s.status != "running",
        "Wait for the scan to finish before reviewing cleanup"
    );
    let mut unique = HashSet::new();
    let mut items = vec![];
    let mut identities = vec![];
    for p in request.paths {
        anyhow::ensure!(unique.insert(p.clone()), "A file was selected twice");
        let indexed: Option<bool> = c
            .query_row(
                "SELECT directory FROM files WHERE session_id=?1 AND path=?2",
                params![request.session_id, p],
                |r| r.get(0),
            )
            .optional()?;
        anyhow::ensure!(
            indexed == Some(false),
            "This file is no longer in the selected scan index. Scan again."
        );
        let (file, identity) =
            inspect(Path::new(&p), policy).with_context(|| format!("Cannot review {p}"))?;
        identities.push(identity);
        items.push(CleanupItem {
            file,
            status: "pending".into(),
            detail: "Awaiting your approval".into(),
            trash_path: None,
        });
    }
    let plan = CleanupPlan {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: request.session_id,
        created_at: now(),
        expires_at: (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
        approved_at: None,
        finished_at: None,
        status: "review".into(),
        allocated_bytes: items.iter().map(|i| i.file.allocated_bytes).sum(),
        items,
        verification: "Not executed. Review every exact path and consequence before approving."
            .into(),
    };
    save(
        c,
        &Record {
            plan: plan.clone(),
            identities,
        },
    )?;
    Ok(plan)
}
pub fn decline(c: &Connection, id: &str) -> Result<CleanupPlan> {
    let mut r = load(c, id)?;
    anyhow::ensure!(
        r.plan.status == "review",
        "Only an unexecuted review can be declined"
    );
    r.plan.status = "declined".into();
    r.plan.finished_at = Some(now());
    r.plan.verification = "Declined. No files were changed.".into();
    save(c, &r)?;
    Ok(r.plan)
}
/// Native filesystem mutation is injected for fixture testing. The caller must
/// serialize this operation with scans, purge and any other cleanup execution.
pub fn execute(
    c: &Connection,
    approval: CleanupApproval,
    policy: &Policy,
    cancel: &AtomicBool,
    mut trash: impl FnMut(&Path) -> Result<PathBuf>,
    mut emit: impl FnMut(CleanupPlan) -> bool,
) -> Result<CleanupPlan> {
    anyhow::ensure!(
        approval.applications_closed && approval.consequences_accepted,
        "Confirm that related apps are closed and the consequences were reviewed"
    );
    let mut r = load(c, &approval.plan_id)?;
    anyhow::ensure!(
        r.plan.status == "review",
        "This plan has already been used or declined. Create a new review."
    );
    anyhow::ensure!(
        r.plan.created_at == approval.reviewed_at,
        "The approval does not match this review"
    );
    anyhow::ensure!(
        chrono::DateTime::parse_from_rfc3339(&r.plan.expires_at)? > chrono::Utc::now(),
        "This review expired. Create a fresh preview."
    );
    r.plan.status = "running".into();
    r.plan.approved_at = Some(now());
    // Persist the approval before performing any filesystem mutation.
    save(c, &r)?;
    let mut connected = emit(r.plan.clone());
    for i in 0..r.plan.items.len() {
        if cancel.load(Ordering::SeqCst) || !connected {
            for item in &mut r.plan.items[i..] {
                item.status = "cancelled".into();
                item.detail = "Not attempted after cancellation".into();
            }
            break;
        }
        let path = PathBuf::from(&r.plan.items[i].file.path);
        let validation = (|| -> Result<fs::File> {
            let (_, current) = inspect(&path, policy)?;
            anyhow::ensure!(
                current == r.identities[i],
                "The file or its parent changed since preview. Scan and review it again."
            );
            let parent = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
                .open(path.parent().unwrap())?;
            let m = parent.metadata()?;
            anyhow::ensure!(
                m.dev() == current.parent_device && m.ino() == current.parent_inode,
                "The parent directory changed"
            );
            Ok(parent)
        })();
        match validation {
            Err(e) => {
                r.plan.items[i].status = "skipped".into();
                r.plan.items[i].detail = format!("Not moved: {e:#}");
            }
            Ok(_parent) => {
                r.plan.items[i].status = "moving".into();
                r.plan.items[i].detail = "Approved; native Trash operation started".into();
                save(c, &r)?;
                let outcome = (|| -> Result<PathBuf> {
                    let (_, latest) = inspect(&path, policy)?;
                    anyhow::ensure!(
                        latest == r.identities[i],
                        "Target changed immediately before the native operation"
                    );
                    trash(&path)
                })();
                match outcome {
                    Err(e) => {
                        r.plan.items[i].status = "failed".into();
                        r.plan.items[i].detail=format!("Native Trash operation failed: {e:#}. Inspect the source location before retrying.");
                    }
                    Ok(destination) => {
                        r.plan.items[i].trash_path = destination.to_str().map(String::from);
                        let verified = fs::symlink_metadata(&path)
                            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                            && fs::symlink_metadata(&destination).is_ok_and(|m| {
                                m.is_file()
                                    && m.dev() == r.identities[i].device
                                    && m.ino() == r.identities[i].inode
                                    && m.len() == r.identities[i].size
                            });
                        r.plan.items[i].status = if verified {
                            "movedToTrash"
                        } else {
                            "unverified"
                        }
                        .into();
                        r.plan.items[i].detail=if verified {"Verified: the original path is absent and the same file is present in Trash. Finder may offer Put Back while the item remains available."} else {"The native operation returned, but the move could not be fully verified. Inspect both locations; do not retry automatically."}.into();
                    }
                }
            }
        }
        save(c, &r)?;
        connected = emit(r.plan.clone());
    }
    r.plan.finished_at = Some(now());
    r.plan.status = if r.plan.items.iter().any(|i| i.status == "cancelled") {
        "cancelled"
    } else if r.plan.items.iter().all(|i| i.status == "movedToTrash") {
        "complete"
    } else {
        "partial"
    }
    .into();
    r.plan.verification="Per-file verification only. No performance improvement or reclaimed capacity is claimed. Recovery depends on the item remaining in Trash; Coqui never empties it. Rescan to refresh storage results.".into();
    save(c, &r)?;
    let _ = emit(r.plan.clone());
    Ok(r.plan)
}
pub fn recover_interrupted(c: &Connection) -> Result<()> {
    let mut q = c.prepare("SELECT id FROM cleanup_plans WHERE status='running'")?;
    let ids = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        let mut r = load(c, &id)?;
        r.plan.status = "interrupted".into();
        r.plan.finished_at = Some(now());
        for item in &mut r.plan.items {
            if item.status == "moving" {
                item.status = "unverified".into();
                item.detail="The app closed during this operation. Inspect the source and Trash manually before making another plan.".into();
            } else if item.status == "pending" {
                item.status = "cancelled".into();
                item.detail = "Not attempted before the app closed".into();
            }
        }
        r.plan.verification="Interrupted; no action is automatically retried. Inspect any unverified item manually.".into();
        save(c, &r)?;
    }
    Ok(())
}
#[cfg(target_os = "macos")]
pub fn native_trash(path: &Path) -> Result<PathBuf> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    objc2::rc::autoreleasepool(|_| {
        let url = NSURL::fileURLWithPath(&NSString::from_str(
            path.to_str().context("Unsupported filename")?,
        ));
        let mut result = None;
        NSFileManager::defaultManager()
            .trashItemAtURL_resultingItemURL_error(&url, Some(&mut result))
            .map_err(|e| anyhow::anyhow!("{}", e.localizedDescription()))?;
        let value = result
            .and_then(|u| u.path())
            .context("macOS did not provide the Trash location. Inspect Finder before retrying.")?;
        Ok(PathBuf::from(value.to_string()))
    })
}
#[cfg(not(target_os = "macos"))]
pub fn native_trash(_path: &Path) -> Result<PathBuf> {
    anyhow::bail!("Native Trash is currently supported only on macOS")
}
