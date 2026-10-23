//! Explicitly selected objects, reviewed metadata seals, journaled native Trash and no-overwrite restore.
//! This does not broaden the cache/project/duplicate rule executor.
use crate::{applications, cleanup, model::now, storage, volumes};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum RemovalKind {
    SelectedFile,
    SelectedFolder,
    AppBundle,
    AssociatedAppData,
    Restore,
    ApprovedCleanupRule,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RemovalRequest {
    pub paths: Vec<String>,
    pub application: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RemovalItem {
    pub path: String,
    pub kind: RemovalKind,
    pub logical_bytes: f64,
    pub allocated_bytes: f64,
    pub entry_count: u32,
    pub association: String,
    pub consequences: String,
    pub status: String,
    pub detail: String,
    pub trash_path: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RemovalPlan {
    #[serde(default)]
    pub administrator: bool,
    pub id: String,
    pub created_at: String,
    pub expires_at: String,
    pub status: String,
    pub items: Vec<RemovalItem>,
    pub finished_at: Option<String>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RemovalApproval {
    pub id: String,
    pub paths: Vec<String>,
    pub acknowledged: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Seal {
    root_changed: i64,
    root_changed_ns: i64,
    dev: u64,
    ino: u64,
    uid: u32,
    parent_dev: u64,
    parent_ino: u64,
    digest: String,
    entries: u32,
    logical: u64,
    allocated: u64,
}
#[derive(Serialize, Deserialize)]
struct Record {
    plan: RemovalPlan,
    seals: Vec<Seal>,
    application: Option<String>,
    #[serde(default)]
    application_seal: Option<Seal>,
    restore_of: Option<String>,
    #[serde(default)]
    restore_parents: Vec<(u64, u64)>,
}
pub struct Policy {
    pub home: PathBuf,
    pub exclusions: Vec<PathBuf>,
    pub self_app: Option<PathBuf>,
    pub administrator_uid: Option<u32>,
}
fn canonical(path: &Path) -> Result<PathBuf> {
    anyhow::ensure!(
        path.is_absolute() && path.to_str().is_some(),
        "An absolute representable path is required"
    );
    let p = path.canonicalize()?;
    anyhow::ensure!(
        volumes::friendly(&p) == volumes::friendly(path),
        "Linked paths are not eligible"
    );
    Ok(volumes::friendly(&p))
}
fn excluded(path: &Path, policy: &Policy) -> bool {
    policy
        .exclusions
        .iter()
        .any(|r| volumes::same_or_below(path, r))
}
pub(crate) fn project_marker(dir: &Path) -> bool {
    [
        ".git",
        ".hg",
        ".svn",
        ".jj",
        "Cargo.toml",
        "package.json",
        "pubspec.yaml",
        "pyproject.toml",
        "pnpm-workspace.yaml",
    ]
    .iter()
    .any(|m| fs::symlink_metadata(dir.join(m)).is_ok())
}
fn scope(path: &Path, app: Option<&str>, policy: &Policy) -> Result<(RemovalKind, String, String)> {
    anyhow::ensure!(!excluded(path, policy), "This location is excluded");
    let home = policy.home.canonicalize()?;
    if policy.administrator_uid.is_some()
        && app.is_none()
        && path.starts_with("/Library/Caches")
        && path != Path::new("/Library/Caches")
    {
        anyhow::ensure!(
            !path
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with("com.apple.")),
            "Apple-managed caches remain protected"
        );
        anyhow::ensure!(
            !path.components().any(|c| [
                ".git",
                ".hg",
                ".svn",
                ".jj",
                "node_modules",
                "target",
                ".dart_tool",
                ".venv",
                "DerivedData"
            ]
            .iter()
            .any(|name| c.as_os_str() == *name)),
            "Developer-managed data requires its existing project/cache workflow"
        );
        for ancestor in path
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .take_while(|p| p.starts_with("/Library/Caches"))
        {
            anyhow::ensure!(
                !project_marker(ancestor),
                "A shared-cache ancestor is project-managed; use developer-artifact review"
            );
        }
        let parent = path.parent().context("Mounted roots are protected")?;
        anyhow::ensure!(
            volumes::mounted_identity(path)? == volumes::mounted_identity(parent)?,
            "Mounted filesystem roots are protected; select their contents"
        );
        return Ok((RemovalKind::ApprovedCleanupRule,"Reviewed shared-cache rule: individual /Library/Caches contents".into(),"Shared cached data may require regeneration or downloading and can affect every user of the application. Close applications first; unique data is not inferred disposable.".into()));
    }
    if let Some(app) = app {
        if policy.administrator_uid.is_some() {
            let a = Path::new(app);
            anyhow::ensure!(a.starts_with("/Applications")||a.starts_with(&home)||a.starts_with("/Volumes"),"Administrator app removal is restricted to shared, personal or explicitly selected volume applications");
        }
        let footprint = applications::inspect(Path::new(app), policy)?;
        anyhow::ensure!(
            footprint.protection.is_empty(),
            "{}",
            footprint.protection.join("; ")
        );
        if path == Path::new(&footprint.path) {
            return Ok((RemovalKind::AppBundle,footprint.bundle_id,"Moves the application bundle only. Services, vendor uninstallers and associated data require separate review. Restoring a bundle cannot guarantee a working installation.".into()));
        }
        let footprint = applications::qualify_related(footprint, policy);
        let related = footprint
            .related
            .iter()
            .find(|r| Path::new(&r.path) == path)
            .context("This location is not confirmed application data")?;
        anyhow::ensure!(
            related.eligible,
            "Shared or unconfirmed data has no cleanup authority"
        );
        return Ok((
            RemovalKind::AssociatedAppData,
            related.association.clone(),
            related.consequences.clone(),
        ));
    }
    let visible = volumes::friendly(path);
    let personal = visible.starts_with(&home)
        && visible != home
        && !visible.starts_with(home.join("Library"))
        && visible
            .strip_prefix(&home)?
            .components()
            .next()
            .is_some_and(|c| !c.as_os_str().to_string_lossy().starts_with('.'));
    let selected_volume = visible.starts_with("/Volumes") || visible.starts_with("/Users/Shared");
    anyhow::ensure!(personal||selected_volume,"Select personal files or individually selected volume contents. Applications, Library data and system locations require their dedicated review.");
    anyhow::ensure!(
        !home.starts_with(&visible),
        "Home and its ancestor folders are protected"
    );
    anyhow::ensure!(
        ![
            "Desktop",
            "Documents",
            "Downloads",
            "Pictures",
            "Movies",
            "Music",
            "Applications"
        ]
        .iter()
        .any(|r| visible == home.join(r)),
        "Keep the personal container folder; select individual contents"
    );
    let metadata = fs::symlink_metadata(path)?;
    let parent_metadata =
        fs::symlink_metadata(path.parent().context("Filesystem roots are protected")?)?;
    anyhow::ensure!(
        metadata.dev() == parent_metadata.dev(),
        "Mounted volume roots are protected; select their contents"
    );
    anyhow::ensure!(
        !path.components().any(|p| [
            ".git",
            ".hg",
            ".svn",
            ".jj",
            "node_modules",
            "target",
            ".Trash",
            ".Trashes",
            "CloudStorage",
            "Mobile Documents"
        ]
        .iter()
        .any(|v| p.as_os_str() == *v)),
        "Project-managed, cloud-provider or recovery data requires its dedicated workflow"
    );
    if path.starts_with(&home) {
        cleanup::cache_project_context_as(
            path.parent().context("Missing parent")?,
            &home,
            policy
                .administrator_uid
                .unwrap_or_else(|| unsafe { libc::geteuid() }),
        )?;
    } else {
        for parent in path.parent().into_iter().flat_map(Path::ancestors) {
            anyhow::ensure!(
                !project_marker(parent),
                "Project data requires developer-artifact review"
            );
        }
    }
    let kind = if path.is_dir() {
        RemovalKind::SelectedFolder
    } else {
        RemovalKind::SelectedFile
    };
    Ok((kind,"Explicit personal-file selection; no disposability inference".into(),"This may be unique work. The entire selected item and every reviewed descendant move to Trash. Close applications using it. Moving to Trash does not immediately reclaim capacity.".into()))
}
fn seal(path: &Path, policy: &Policy, cancel: &AtomicBool, opaque_bundle: bool) -> Result<Seal> {
    let path = canonical(path)?;
    #[cfg(target_os = "macos")]
    if let Some(uid) = policy.administrator_uid {
        return administrator_seal(&path, policy, cancel, opaque_bundle, uid);
    }
    let parent = fs::symlink_metadata(
        path.parent()
            .context("A filesystem root cannot be removed")?,
    )?;
    let root = fs::symlink_metadata(&path)?;
    anyhow::ensure!(root.uid()==unsafe{libc::geteuid()} && (root.is_file()||root.is_dir()) && !root.file_type().is_symlink(),"Only user-owned regular files and folders support portable removal; review administrator access for other items");
    let mut hash = Sha256::new();
    let mut counted = std::collections::HashSet::new();
    let mut entries = 0u32;
    let (mut logical, mut allocated) = (0u64, 0u64);
    let walker = walkdir::WalkDir::new(&path)
        .follow_links(false)
        .max_open(8)
        .sort_by_file_name();
    for next in walker {
        anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Review cancelled");
        let e = next?;
        let p = e.path();
        let m = fs::symlink_metadata(p)?;
        anyhow::ensure!(
            entries < 100_000,
            "This review exceeds 100,000 entries; review smaller folders"
        );
        anyhow::ensure!(!excluded(p, policy), "A descendant is excluded");
        anyhow::ensure!(
            m.dev() == root.dev() && !storage::placeholder(p, &m),
            "Mount boundaries and cloud placeholders are not eligible"
        );
        anyhow::ensure!(m.uid() == root.uid(), "A descendant has a different owner");
        anyhow::ensure!(
            m.is_file() || m.is_dir() || (opaque_bundle && m.file_type().is_symlink()),
            "Links and special objects require separate review"
        );
        if !opaque_bundle && m.is_dir() {
            anyhow::ensure!(
                !project_marker(p),
                "The folder contains project data; use developer-artifact review"
            );
        }
        let relative = p
            .strip_prefix(&path)?
            .to_str()
            .context("A filename cannot be represented")?;
        // Each component is length framed. Symlinks remain opaque: no target read or traversal.
        hash.update((relative.len() as u64).to_le_bytes());
        hash.update(relative.as_bytes());
        for value in [
            m.dev(),
            m.ino(),
            m.mode() as u64,
            m.uid() as u64,
            m.gid() as u64,
            m.len(),
            m.blocks(),
            m.mtime() as u64,
            m.mtime_nsec() as u64,
            if p == path { 0 } else { m.ctime() as u64 },
            if p == path { 0 } else { m.ctime_nsec() as u64 },
            m.nlink(),
        ] {
            hash.update(value.to_le_bytes());
        }
        if m.is_file() && counted.insert((m.dev(), m.ino())) {
            logical = logical.saturating_add(m.len());
            allocated = allocated.saturating_add(m.blocks() * 512);
        }
        entries += 1;
    }
    Ok(Seal {
        root_changed: root.ctime(),
        root_changed_ns: root.ctime_nsec(),
        dev: root.dev(),
        ino: root.ino(),
        uid: root.uid(),
        parent_dev: parent.dev(),
        parent_ino: parent.ino(),
        digest: format!("{:x}", hash.finalize()),
        entries,
        logical,
        allocated,
    })
}
fn save(c: &Connection, r: &Record) -> Result<()> {
    c.execute("INSERT INTO removal_plans VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET status=excluded.status,json=excluded.json",params![r.plan.id,r.plan.created_at,r.plan.status,serde_json::to_string(r)?])?;
    Ok(())
}
fn load(c: &Connection, id: &str) -> Result<Record> {
    let json: String = c.query_row("SELECT json FROM removal_plans WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    let r: Record = serde_json::from_str(&json)?;
    anyhow::ensure!(
        !r.plan.items.is_empty() && r.plan.items.len() == r.seals.len(),
        "Removal journal is inconsistent; inspect locations manually"
    );
    Ok(r)
}
pub fn history(c: &Connection, offset: u32) -> Result<Vec<RemovalPlan>> {
    let mut q =
        c.prepare("SELECT json FROM removal_plans ORDER BY created_at DESC LIMIT 50 OFFSET ?1")?;
    let rows = q.query_map([offset], |r| r.get::<_, String>(0))?;
    rows.map(|r| Ok(serde_json::from_str::<Record>(&r?)?.plan))
        .collect()
}
pub fn preview(
    c: &Connection,
    request: RemovalRequest,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<RemovalPlan> {
    anyhow::ensure!(
        !request.paths.is_empty() && request.paths.len() <= 100,
        "Select between 1 and 100 items"
    );
    let mut paths: Vec<PathBuf> = request
        .paths
        .iter()
        .map(|p| canonical(Path::new(p)))
        .collect::<Result<_>>()?;
    paths.sort();
    for (i, p) in paths.iter().enumerate() {
        anyhow::ensure!(
            !paths[..i].iter().any(|r| p.starts_with(r)),
            "Overlapping or repeated selections must be removed"
        );
    }
    // Related data is handled before the bundle so every item can revalidate the installed identity.
    paths.sort_by_key(|p| {
        request
            .application
            .as_ref()
            .is_some_and(|app| Path::new(app) == p)
    });
    let application_seal = request
        .application
        .as_ref()
        .map(|app| seal(Path::new(app), policy, cancel, true))
        .transpose()?;
    let mut items = vec![];
    let mut seals = vec![];
    for p in paths {
        let (kind, association, consequences) = scope(&p, request.application.as_deref(), policy)?;
        let s = seal(&p, policy, cancel, matches!(kind, RemovalKind::AppBundle))?;
        items.push(RemovalItem {
            path: p.to_str().unwrap().into(),
            kind,
            logical_bytes: s.logical as f64,
            allocated_bytes: s.allocated as f64,
            entry_count: s.entries,
            association,
            consequences,
            status: "pending".into(),
            detail: "Awaiting exact-path approval".into(),
            trash_path: None,
        });
        seals.push(s);
    }
    let plan=RemovalPlan{administrator:policy.administrator_uid.is_some(),id:uuid::Uuid::new_v4().to_string(),created_at:now(),expires_at:(chrono::Utc::now()+chrono::Duration::minutes(10)).to_rfc3339(),status:"review".into(),items,finished_at:None,qualification:"Metadata review only. APFS shared allocation is not guaranteed recoverable space. Coqui never empties Trash. Approval expires after ten minutes; changes require a new review.".into()};
    save(
        c,
        &Record {
            plan: plan.clone(),
            seals,
            application: request.application,
            application_seal,
            restore_of: None,
            restore_parents: vec![],
        },
    )?;
    Ok(plan)
}
fn fresh(p: &RemovalPlan) -> Result<()> {
    anyhow::ensure!(
        p.status == "review"
            && chrono::DateTime::parse_from_rfc3339(&p.expires_at)?.with_timezone(&chrono::Utc)
                > chrono::Utc::now(),
        "This approval was used or expired; review again"
    );
    Ok(())
}
fn same_moved(path: &Path, s: &Seal) -> Result<bool> {
    let m = fs::symlink_metadata(path)?;
    Ok(m.dev() == s.dev && m.ino() == s.ino && m.uid() == s.uid)
}
pub fn execute(
    c: &Connection,
    a: RemovalApproval,
    policy: &Policy,
    cancel: &AtomicBool,
    mut trash: impl FnMut(&Path) -> Result<PathBuf>,
    mut emit: impl FnMut(RemovalPlan) -> bool,
) -> Result<RemovalPlan> {
    let mut r = load(c, &a.id)?;
    anyhow::ensure!(
        r.plan.administrator == policy.administrator_uid.is_some(),
        "This plan requires its original execution authority"
    );
    fresh(&r.plan)?;
    anyhow::ensure!(
        a.acknowledged
            && a.paths
                == r.plan
                    .items
                    .iter()
                    .map(|i| i.path.clone())
                    .collect::<Vec<_>>(),
        "Approve exactly the reviewed paths and consequences"
    );
    r.plan.status = "running".into();
    anyhow::ensure!(
        c.execute(
            "UPDATE removal_plans SET status='running',json=?1 WHERE id=?2 AND status='review'",
            params![serde_json::to_string(&r)?, r.plan.id]
        )? == 1,
        "This plan was already claimed; inspect its retained result"
    );
    if !emit(r.plan.clone()) {
        cancel.store(true, Ordering::Relaxed);
    }
    for i in 0..r.plan.items.len() {
        if cancel.load(Ordering::Relaxed) {
            r.plan.items[i].status = "cancelled".into();
            r.plan.items[i].detail = "No move attempted".into();
            continue;
        }
        r.plan.items[i].status = "intentRecorded".into();
        save(c, &r)?;
        let p = PathBuf::from(&r.plan.items[i].path);
        let attempt = (|| -> Result<PathBuf> {
            if r.restore_of.is_some() {
                let source = PathBuf::from(
                    r.plan.items[i]
                        .trash_path
                        .as_ref()
                        .context("Missing recovery path")?,
                );
                validate_trash(&source, policy)?;
                anyhow::ensure!(
                    fs::symlink_metadata(&p)
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
                    "Restore destination exists or cannot be inspected"
                );
                anyhow::ensure!(!excluded(&p, policy), "Restore destination is excluded");
                let current = seal(
                    &source,
                    policy,
                    cancel,
                    matches!(r.plan.items[i].kind, RemovalKind::Restore),
                )?;
                anyhow::ensure!(
                    current == r.seals[i],
                    "Trash item or parent changed; review restore again"
                );
                let parent = fs::symlink_metadata(p.parent().context("Missing parent")?)?;
                anyhow::ensure!(
                    r.restore_parents.get(i) == Some(&(parent.dev(), parent.ino())),
                    "Original parent identity changed"
                );
                anyhow::ensure!(
                    parent.is_dir() && p.parent().unwrap().canonicalize()? == p.parent().unwrap(),
                    "Original parent changed or is no longer a canonical directory"
                );
                #[cfg(target_os = "macos")]
                if policy.administrator_uid.is_some() {
                    crate::safe_fs::exclusive_move(
                        &source,
                        &p,
                        (r.seals[i].dev, r.seals[i].ino, r.seals[i].uid),
                        r.restore_parents.get(i).copied(),
                    )?;
                } else {
                    native_restore(&source, &p)?;
                }
                #[cfg(not(target_os = "macos"))]
                native_restore(&source, &p)?;
                Ok(p.clone())
            } else {
                if let (Some(app), Some(expected)) = (&r.application, &r.application_seal) {
                    anyhow::ensure!(
                        seal(Path::new(app), policy, cancel, true)? == *expected,
                        "Application identity or contents changed; review again"
                    );
                }
                let (kind, _, _) = scope(&p, r.application.as_deref(), policy)?;
                anyhow::ensure!(
                    seal(&p, policy, cancel, matches!(kind, RemovalKind::AppBundle))? == r.seals[i],
                    "Reviewed item or folder contents changed; review again"
                );
                trash(&p)
            }
        })();
        match attempt {
            Ok(dest) => {
                if r.restore_of.is_none() {
                    r.plan.items[i].trash_path = dest.to_str().map(str::to_owned);
                }
                // Record the returned destination before verification; interrupted moves are never retried automatically.
                r.plan.items[i].status = "verifying".into();
                save(c, &r)?;
                let absent = if r.restore_of.is_some() {
                    r.plan.items[i].trash_path.as_ref().is_some_and(|s| {
                        fs::symlink_metadata(s)
                            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    })
                } else {
                    fs::symlink_metadata(&p)
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                };
                let valid = absent
                    && same_moved(&dest, &r.seals[i]).unwrap_or(false)
                    && seal(&dest, policy, &AtomicBool::new(false), true).is_ok_and(
                        |mut current| {
                            current.parent_dev = r.seals[i].parent_dev;
                            current.parent_ino = r.seals[i].parent_ino;
                            current.root_changed = r.seals[i].root_changed;
                            current.root_changed_ns = r.seals[i].root_changed_ns;
                            current == r.seals[i]
                        },
                    );
                r.plan.items[i].status = if valid {
                    if r.restore_of.is_some() {
                        "restored"
                    } else {
                        "movedToTrash"
                    }
                } else {
                    "verificationFailed"
                }
                .into();
                r.plan.items[i].detail=if valid{if r.restore_of.is_some(){"Trash source absence and the restored object/contents at the original path verified. Full application recovery is not promised."}else{"Original path absence and destination object/contents in Trash verified. No reclaimed capacity or full application recovery is promised."}}else{"The move returned but identity or source absence could not be verified. Inspect both locations; do not retry automatically."}.into();
            }
            Err(e) => {
                r.plan.items[i].status = "blocked".into();
                r.plan.items[i].detail = format!("{e:#}");
            }
        }
        let tx = c.unchecked_transaction()?;
        save(&tx, &r)?;
        mark_restored_source(&tx, &r)?;
        tx.commit()?;
        if !emit(r.plan.clone()) {
            cancel.store(true, Ordering::Relaxed);
        }
    }
    r.plan.status = if r
        .plan
        .items
        .iter()
        .all(|i| ["movedToTrash", "restored"].contains(&i.status.as_str()))
    {
        "complete"
    } else if cancel.load(Ordering::Relaxed) {
        "cancelled"
    } else {
        "partial"
    }
    .into();
    r.plan.finished_at = Some(now());
    save(c, &r)?;
    let _ = emit(r.plan.clone());
    Ok(r.plan)
}
fn validate_trash(path: &Path, policy: &Policy) -> Result<()> {
    let home = policy.home.canonicalize()?;
    let parent = path.parent().context("Missing Trash parent")?;
    let personal = parent == home.join(".Trash");
    let volume = parent.file_name().is_some_and(|n| {
        n == policy
            .administrator_uid
            .unwrap_or_else(|| unsafe { libc::geteuid() })
            .to_string()
            .as_str()
    }) && parent
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|n| n == ".Trashes");
    anyhow::ensure!(
        (personal || volume)
            && parent.canonicalize()? == parent
            && fs::symlink_metadata(parent)?.uid()
                == policy
                    .administrator_uid
                    .unwrap_or_else(|| unsafe { libc::geteuid() }),
        "Only retained items in the requesting user's Trash support restore"
    );
    Ok(())
}
pub fn preview_restore(
    c: &Connection,
    id: &str,
    paths: &[String],
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<RemovalPlan> {
    let original = load(c, id)?;
    anyhow::ensure!(
        original.restore_of.is_none() && !paths.is_empty(),
        "Select retained Trash moves to restore"
    );
    let mut items = vec![];
    let mut seals = vec![];
    let mut parents = vec![];
    for p in paths {
        anyhow::ensure!(
            !excluded(Path::new(p), policy),
            "The restore destination is now excluded or protected"
        );
        let i = original
            .plan
            .items
            .iter()
            .position(|i| &i.path == p && i.status == "movedToTrash")
            .context("Only verified retained Trash moves can be reviewed")?;
        let old = &original.plan.items[i];
        let source = Path::new(
            old.trash_path
                .as_deref()
                .context("No retained Trash path")?,
        );
        validate_trash(source, policy)?;
        anyhow::ensure!(
            same_moved(source, &original.seals[i])?,
            "Trash identity no longer matches"
        );
        anyhow::ensure!(
            fs::symlink_metadata(p).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
            "Original destination already exists or is unavailable"
        );
        let s = seal(
            source,
            policy,
            cancel,
            matches!(old.kind, RemovalKind::AppBundle),
        )?;
        let mut item = old.clone();
        item.kind = RemovalKind::Restore;
        item.status = "pending".into();
        item.detail = "Awaiting restore approval".into();
        item.consequences="Returns only this retained object to the original path without overwriting. App dependencies, registration and subsequent changes are not restored.".into();
        items.push(item);
        seals.push(s);
        parents.push((original.seals[i].parent_dev, original.seals[i].parent_ino));
    }
    let plan=RemovalPlan{administrator:policy.administrator_uid.is_some(),id:uuid::Uuid::new_v4().to_string(),created_at:now(),expires_at:(chrono::Utc::now()+chrono::Duration::minutes(10)).to_rfc3339(),status:"review".into(),items,finished_at:None,qualification:"Restore requires the same Trash identity and an absent original destination. No full application recovery is promised.".into()};
    save(
        c,
        &Record {
            plan: plan.clone(),
            seals,
            application: None,
            application_seal: None,
            restore_of: Some(id.into()),
            restore_parents: parents,
        },
    )?;
    Ok(plan)
}
#[cfg(target_os = "macos")]
fn native_restore(source: &Path, destination: &Path) -> Result<()> {
    use std::ffi::CString;
    let from = CString::new(source.to_str().context("Invalid Trash path")?)?;
    let to = CString::new(destination.to_str().context("Invalid original path")?)?;
    // RENAME_EXCL makes an existing destination an error at the filesystem operation.
    let status = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
    anyhow::ensure!(
        status == 0,
        "Restore failed: {}",
        std::io::Error::last_os_error()
    );
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn native_restore(_source: &Path, _destination: &Path) -> Result<()> {
    anyhow::bail!("Native restore requires macOS")
}
fn mark_restored_source(c: &Connection, r: &Record) -> Result<()> {
    let Some(original) = r.restore_of.as_deref() else {
        return Ok(());
    };
    let mut source = match load(c, original) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let mut changed = false;
    for restored in r.plan.items.iter().filter(|i| i.status == "restored") {
        if let Some(item) = source.plan.items.iter_mut().find(|i| {
            i.path == restored.path
                && i.trash_path == restored.trash_path
                && i.status == "movedToTrash"
        }) {
            item.status = "restoredFromTrash".into();
            item.detail=format!("Verified restore recorded by recovery plan {}. The retained object is no longer offered for Trash recovery.",r.plan.id);
            changed = true;
        }
    }
    if changed {
        save(c, &source)?;
    }
    Ok(())
}
pub fn recover(c: &Connection) -> Result<()> {
    let mut q = c.prepare("SELECT id FROM removal_plans WHERE status='running'")?;
    let ids = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        let mut r = load(c, &id)?;
        r.plan.status = "interrupted".into();
        for item in &mut r.plan.items {
            if ["pending", "intentRecorded", "verifying"].contains(&item.status.as_str()) {
                item.status = "interrupted".into();
                item.detail="The app closed during this operation. Inspect source and Trash manually before another review.".into();
            }
        }
        save(c, &r)?;
    }
    let mut q = c.prepare(
        "SELECT id FROM removal_plans WHERE json_extract(json,'$.restore_of') IS NOT NULL",
    )?;
    let ids = q
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        mark_restored_source(c, &load(c, &id)?)?;
    }
    Ok(())
}
pub fn native_execute(
    c: &Connection,
    a: RemovalApproval,
    policy: &Policy,
    cancel: &AtomicBool,
    emit: impl FnMut(RemovalPlan) -> bool,
) -> Result<RemovalPlan> {
    let retained = load(c, &a.id)?;
    execute(
        c,
        a,
        policy,
        cancel,
        |path| {
            #[cfg(target_os = "macos")]
            if let Some(uid) = policy.administrator_uid {
                let expected = retained
                    .plan
                    .items
                    .iter()
                    .position(|item| Path::new(&item.path) == path)
                    .and_then(|i| retained.seals.get(i))
                    .context("Missing retained move identity")?;
                return administrator_trash(path, uid, expected);
            }
            cleanup::native_trash(path)
        },
        emit,
    )
}

pub fn entries(
    c: &Connection,
    id: &str,
    index: u32,
    offset: u32,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<Vec<crate::model::FileEntry>> {
    let r = load(c, id)?;
    let i = index as usize;
    let item = r.plan.items.get(i).context("Review item not found")?;
    let source = if r.restore_of.is_some() {
        item.trash_path.as_deref().context("No Trash location")?
    } else {
        &item.path
    };
    let path = Path::new(source);
    let current = seal(
        path,
        policy,
        cancel,
        matches!(item.kind, RemovalKind::AppBundle | RemovalKind::Restore),
    )?;
    anyhow::ensure!(
        current == r.seals[i],
        "Reviewed contents changed; close this plan and review again"
    );
    #[cfg(target_os = "macos")]
    if let Some(uid) = policy.administrator_uid {
        return administrator_snapshot(
            path,
            policy,
            cancel,
            matches!(item.kind, RemovalKind::AppBundle | RemovalKind::Restore),
            uid,
        )?
        .into_iter()
        .skip(offset as usize)
        .take(100)
        .map(|(relative, m)| {
            let p = crate::safe_fs::path_of(&relative, path);
            Ok(crate::model::FileEntry {
                path: p.to_string_lossy().into_owned(),
                parent: p.parent().unwrap_or(path).to_string_lossy().into_owned(),
                name: p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                category: if m.st_mode & libc::S_IFMT == libc::S_IFLNK {
                    "Opaque bundle link".into()
                } else {
                    storage::category(&p).into()
                },
                logical_bytes: if m.st_mode & libc::S_IFMT == libc::S_IFREG {
                    m.st_size as f64
                } else {
                    0.
                },
                allocated_bytes: if m.st_mode & libc::S_IFMT == libc::S_IFREG {
                    m.st_blocks as f64 * 512.
                } else {
                    0.
                },
                is_directory: m.st_mode & libc::S_IFMT == libc::S_IFDIR,
                modified_at: String::new(),
                qualification:
                    "Descriptor-relative metadata; links are opaque, contents are not downloaded."
                        .into(),
            })
        })
        .collect();
    }
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .max_open(8)
        .sort_by_file_name()
        .into_iter()
        .skip(offset as usize)
        .take(100)
        .map(|entry| {
            let e = entry?;
            let p = e.path();
            let m = fs::symlink_metadata(p)?;
            Ok(crate::model::FileEntry {
                path: p.to_str().context("Unrepresentable filename")?.into(),
                parent: p.parent().unwrap_or(path).to_string_lossy().into_owned(),
                name: p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                category: if m.file_type().is_symlink() {
                    "Opaque bundle link".into()
                } else {
                    storage::category(p).into()
                },
                logical_bytes: if m.is_file() { m.len() as f64 } else { 0. },
                allocated_bytes: if m.is_file() {
                    m.blocks() as f64 * 512.
                } else {
                    0.
                },
                is_directory: m.is_dir(),
                modified_at: String::new(),
                qualification:
                    "Folder sizes are aggregated in the reviewed item; links are never followed."
                        .into(),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn administrator_snapshot(
    path: &Path,
    policy: &Policy,
    cancel: &AtomicBool,
    opaque: bool,
    uid: u32,
) -> Result<Vec<(String, libc::stat)>> {
    use crate::safe_fs;
    use std::os::fd::AsRawFd;
    fn visit(
        dir: &std::fs::File,
        name: &std::ffi::CStr,
        relative: String,
        base: &Path,
        policy: &Policy,
        cancel: &AtomicBool,
        opaque: bool,
        uid: u32,
        rows: &mut Vec<(String, libc::stat)>,
        name_bytes: &mut usize,
        depth: usize,
        mount: &Option<([libc::c_char; 1024], [libc::c_char; 1024])>,
    ) -> Result<()> {
        anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Review cancelled");
        anyhow::ensure!(
            rows.len() < 20_000 && depth < 128,
            "Administrator review exceeds 20,000 entries or 128 levels; select smaller folders"
        );
        *name_bytes += relative.len();
        anyhow::ensure!(
            *name_bytes <= 8 * 1024 * 1024,
            "Reviewed path data exceeds 8 MiB; select smaller folders"
        );
        let m = safe_fs::stat(dir, name)?;
        let p = safe_fs::path_of(&relative, base);
        anyhow::ensure!(!excluded(&p, policy), "A reviewed descendant is excluded");
        anyhow::ensure!(
            m.st_uid == 0 || m.st_uid == uid,
            "Other users' data is protected from removal"
        );
        anyhow::ensure!(
            m.st_flags & (0x40000000 | 0x80000 | 0x2 | 0x20000) == 0
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(".icloud")),
            "Cloud-only, immutable or system-protected items cannot be removed"
        );
        let kind = m.st_mode & libc::S_IFMT;
        anyhow::ensure!(
            kind == libc::S_IFDIR || kind == libc::S_IFREG || (opaque && kind == libc::S_IFLNK),
            "Links and special objects require separate review"
        );
        rows.push((relative.clone(), m));
        if kind == libc::S_IFDIR {
            let child = safe_fs::open_child(dir, name, &m)?;
            let mut fs: libc::statfs = unsafe { std::mem::zeroed() };
            anyhow::ensure!(
                unsafe { libc::fstatfs(child.as_raw_fd(), &mut fs) } == 0,
                "Mount identity unavailable"
            );
            let identity = (fs.f_mntfromname, fs.f_mntonname);
            anyhow::ensure!(
                mount.as_ref().is_none_or(|root| *root == identity),
                "A reviewed folder crosses a mounted filesystem"
            );
            let root_mount = Some(identity);
            safe_fs::names(&child, |name| {
                let text = name
                    .to_str()
                    .context("Unrepresentable filename cannot be approved")?;
                if !opaque {
                    anyhow::ensure!(
                        ![
                            ".git",
                            ".hg",
                            ".svn",
                            ".jj",
                            "Cargo.toml",
                            "package.json",
                            "pubspec.yaml",
                            "pyproject.toml",
                            "pnpm-workspace.yaml"
                        ]
                        .contains(&text),
                        "Project-managed data requires its existing Git/artifact workflow"
                    );
                }
                let next = if relative.is_empty() {
                    text.to_owned()
                } else {
                    format!("{relative}/{text}")
                };
                visit(
                    &child,
                    &name,
                    next,
                    base,
                    policy,
                    cancel,
                    opaque,
                    uid,
                    rows,
                    name_bytes,
                    depth + 1,
                    &root_mount,
                )
            })?;
        }
        Ok(())
    }
    let (parent, name) = safe_fs::parent(path)?;
    let mut rows = vec![];
    visit(
        &parent,
        &name,
        String::new(),
        path,
        policy,
        cancel,
        opaque,
        uid,
        &mut rows,
        &mut 0,
        0,
        &None,
    )?;
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(rows)
}
#[cfg(target_os = "macos")]
fn administrator_seal(
    path: &Path,
    policy: &Policy,
    cancel: &AtomicBool,
    opaque: bool,
    uid: u32,
) -> Result<Seal> {
    let (parent, name) = crate::safe_fs::parent(path)?;
    let root = crate::safe_fs::stat(&parent, &name)?;
    let parent = parent.metadata()?;
    let mut hash = Sha256::new();
    let mut counted = std::collections::HashSet::new();
    let (mut logical, mut allocated) = (0u64, 0u64);
    let rows = administrator_snapshot(path, policy, cancel, opaque, uid)?;
    for (relative, m) in &rows {
        hash.update((relative.len() as u64).to_le_bytes());
        hash.update(relative.as_bytes());
        for value in [
            m.st_dev as u64,
            m.st_ino,
            m.st_mode as u64,
            m.st_uid as u64,
            m.st_gid as u64,
            m.st_size as u64,
            m.st_blocks as u64,
            m.st_mtime as u64,
            m.st_mtime_nsec as u64,
            if relative.is_empty() {
                0
            } else {
                m.st_ctime as u64
            },
            if relative.is_empty() {
                0
            } else {
                m.st_ctime_nsec as u64
            },
            m.st_nlink as u64,
            m.st_flags as u64,
        ] {
            hash.update(value.to_le_bytes());
        }
        if m.st_mode & libc::S_IFMT == libc::S_IFREG && counted.insert((m.st_dev, m.st_ino)) {
            logical = logical.saturating_add(m.st_size as u64);
            allocated = allocated.saturating_add(m.st_blocks as u64 * 512);
        }
    }
    Ok(Seal {
        root_changed: root.st_ctime,
        root_changed_ns: root.st_ctime_nsec,
        dev: root.st_dev as u64,
        ino: root.st_ino,
        uid: root.st_uid,
        parent_dev: parent.dev(),
        parent_ino: parent.ino(),
        digest: format!("{:x}", hash.finalize()),
        entries: rows.len() as u32,
        logical,
        allocated,
    })
}
#[cfg(target_os = "macos")]
fn administrator_trash(path: &Path, uid: u32, expected: &Seal) -> Result<PathBuf> {
    let trash = crate::privileged::user_trash_directory(path, uid)?;
    let dir = crate::safe_fs::directory(&trash).context(
        "Cannot open the requesting user's Trash. If macOS reports Operation not permitted, review Full Disk Access for /Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper, then prepare a new review",
    )?;
    let m = dir.metadata()?;
    anyhow::ensure!(
        m.uid() == uid && m.mode() & 0o077 == 0,
        "The requesting user's Trash must be private and owned by that user"
    );
    let (source, name) = crate::safe_fs::parent(path)?;
    let before = crate::safe_fs::stat(&source, &name)?;
    anyhow::ensure!(
        m.dev() == before.st_dev as u64,
        "Cross-filesystem Trash copies are unsupported"
    );
    let base = path
        .file_name()
        .context("Missing filename")?
        .to_str()
        .context("Unrepresentable filename")?;
    let mut end = base.len().min(160);
    while !base.is_char_boundary(end) {
        end -= 1;
    }
    let destination = trash.join(format!("{}-Coqui-{}", &base[..end], uuid::Uuid::new_v4()));
    crate::safe_fs::exclusive_move(
        path,
        &destination,
        (expected.dev, expected.ino, expected.uid),
        Some((m.dev(), m.ino())),
    )?;
    Ok(destination)
}
