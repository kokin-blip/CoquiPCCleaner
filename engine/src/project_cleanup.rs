//! Folder-specific eligibility and approval seals. No recursive deletion or tooling execution.
use crate::{cleanup::Policy, developer, model::*, storage};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Stamp {
    device: u64,
    inode: u64,
    size: u64,
    blocks: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
    mode: u32,
    uid: u32,
    links: u64,
}
impl Stamp {
    fn of(m: &fs::Metadata) -> Self {
        Self {
            device: m.dev(),
            inode: m.ino(),
            size: m.len(),
            blocks: m.blocks(),
            modified: m.mtime(),
            modified_ns: m.mtime_nsec(),
            changed: m.ctime(),
            changed_ns: m.ctime_nsec(),
            mode: m.mode(),
            uid: m.uid(),
            links: m.nlink(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Entry {
    relative: String,
    directory: bool,
    stamp: Stamp,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Seal {
    project: String,
    git: String,
    manifests: Vec<(String, Stamp)>,
    tree: Vec<Entry>,
}
const MARKERS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".jj",
    "Cargo.toml",
    "package.json",
    "pubspec.yaml",
    "pyproject.toml",
    "requirements.txt",
    "setup.py",
    "pnpm-workspace.yaml",
    "lerna.json",
    "pyvenv.cfg",
];

fn manifest(path: &Path, root: &Path, cancel: &AtomicBool) -> Result<(Stamp, String)> {
    let before = fs::symlink_metadata(path)?;
    anyhow::ensure!(
        before.is_file()
            && before.uid() == unsafe { libc::geteuid() }
            && before.nlink() == 1
            && !storage::placeholder(path, &before)
            && before.len() <= 4 * 1024 * 1024,
        "Manifest/lockfile is linked, cloud-only, oversized or not user-owned"
    );
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let stamp = Stamp::of(&before);
    anyhow::ensure!(
        Stamp::of(&file.metadata()?) == stamp,
        "Manifest changed before reading"
    );
    let mut value = String::new();
    (&file)
        .take(4 * 1024 * 1024 + 1)
        .read_to_string(&mut value)?;
    anyhow::ensure!(
        !value.trim().is_empty()
            && !value.contains('\0')
            && value.len() <= 4 * 1024 * 1024
            && Stamp::of(&file.metadata()?) == stamp
            && Stamp::of(&fs::symlink_metadata(path)?) == stamp,
        "Manifest changed or is not bounded text"
    );
    let relative = path
        .strip_prefix(root)?
        .to_str()
        .context("Unsupported manifest path")?;
    let output = developer::git(
        root,
        &["ls-files", "--cached", "--stage", "-z", "--", relative],
        cancel,
    )?;
    let records: Vec<_> = output
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .collect();
    anyhow::ensure!(
        records.len() == 1 && records[0].starts_with(b"100644 ")
            || records.len() == 1 && records[0].starts_with(b"100755 "),
        "Manifest/lockfile must be tracked as a regular Git file"
    );
    Ok((stamp, value))
}

fn tree(
    path: &Path,
    exclusions: &[std::path::PathBuf],
    cancel: &AtomicBool,
    bytecode: bool,
) -> Result<Vec<Entry>> {
    let start = Instant::now();
    let root = fs::symlink_metadata(path)?;
    let mut entries = vec![];
    for (count, item) in walkdir::WalkDir::new(path)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .enumerate()
    {
        anyhow::ensure!(
            !cancel.load(Ordering::SeqCst),
            "Project inspection cancelled"
        );
        anyhow::ensure!(count < 20_000 && start.elapsed() < Duration::from_secs(5), "Folder exceeds 20,000-entry/five-second inspection limit; review a smaller output manually");
        let item = item.context("Folder coverage is incomplete")?;
        let p = item.path();
        let m = fs::symlink_metadata(p)?;
        anyhow::ensure!((m.is_dir() || m.is_file()) && m.uid() == unsafe{libc::geteuid()} && m.dev() == root.dev() && !storage::placeholder(p,&m) && (!m.is_file() || m.nlink() == 1), "Folder contains a link, hard link, special file, mount boundary, non-owned or cloud-only entry");
        anyhow::ensure!(
            m.mode() & 0o400 != 0 && (!m.is_dir() || m.mode() & 0o100 != 0),
            "Folder contains an unreadable entry"
        );
        anyhow::ensure!(
            !exclusions
                .iter()
                .any(|e| p.starts_with(e) || e.starts_with(p)),
            "Folder intersects an exclusion"
        );
        let relative = p
            .strip_prefix(path)?
            .to_str()
            .context("Filename cannot be represented")?
            .to_owned();
        if count > 0 {
            anyhow::ensure!(
                !MARKERS.contains(&item.file_name().to_str().unwrap_or("")),
                "Nested project/repository or environment marker detected; preserve this folder"
            );
            if bytecode {
                anyhow::ensure!(m.is_file() && p.extension().is_some_and(|e|e == "pyc"), "Python bytecode cleanup accepts only regular .pyc files directly in __pycache__");
            }
        }
        entries.push(Entry {
            relative,
            directory: m.is_dir(),
            stamp: Stamp::of(&m),
        });
    }
    anyhow::ensure!(
        entries.iter().any(|e| !e.directory),
        "Folder contains no regular files to review"
    );
    Ok(entries)
}

pub(crate) fn inspect(
    path: &Path,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<(CleanupFile, Seal)> {
    let exclusions = policy
        .exclusions
        .iter()
        .map(|e| e.canonicalize().unwrap_or(e.clone()))
        .collect::<Vec<_>>();
    developer::available(path, &exclusions)?;
    let (technology, _, project, marker, consequence) =
        developer::identify(path).context("No supported project marker at this output")?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Unsupported output path")?;
    anyhow::ensure!(matches!((technology,name), ("Rust","target") | ("Node.js",".next"|".nuxt"|".svelte-kit"|"dist"|"build") | ("Dart / Flutter","build"|".dart_tool") | ("Python","__pycache__")), "Analysis only: dependencies, virtual environments and unassociated Xcode data are not supported cleanup targets");
    developer::available(&project, &exclusions)?;
    let repo = developer::inspect_repository(&project, &exclusions, cancel);
    anyhow::ensure!(
        repo.status == "clean",
        "A clean, fully inspected Git checkout is required: {}. {}",
        repo.status,
        repo.qualification
    );
    let root = Path::new(repo.root.as_ref().context("No confirmed Git checkout")?);
    anyhow::ensure!(
        path.starts_with(root) && path != root && project.starts_with(root),
        "Output must be inside its confirmed checkout"
    );
    for w in &repo.worktrees {
        anyhow::ensure!(!w.locked && !w.prunable && w.present && !w.bare && w.head.is_some() && !Path::new(&w.path).starts_with(path), "Locked, missing, prunable, unborn, bare or enclosed worktree relationship requires manual review");
    }
    for metadata in [&repo.git_directory, &repo.common_directory] {
        anyhow::ensure!(
            metadata
                .as_ref()
                .is_some_and(|v| !Path::new(v).starts_with(path)),
            "Output encloses Git metadata"
        );
    }
    anyhow::ensure!(
        developer::tracked(path, &repo, cancel)? == (0, 0),
        "Git tracks output entries or submodules; preserve them"
    );
    let relative = path
        .strip_prefix(root)?
        .to_str()
        .context("Unsupported output path")?;
    developer::git(
        root,
        &["check-ignore", "--no-index", "--", relative],
        cancel,
    )
    .context("The output folder itself must be Git-ignored")?;
    anyhow::ensure!(
        developer::git(
            root,
            &[
                "ls-files",
                "--others",
                "--exclude-standard",
                "-z",
                "--",
                relative
            ],
            cancel
        )?
        .is_empty(),
        "Output contains non-ignored untracked files; preserve them"
    );
    let mut manifests = vec![];
    let manifest_path = project.join(&marker);
    let (stamp, value) = manifest(&manifest_path, root, cancel)?;
    manifests.push((
        manifest_path
            .to_str()
            .context("Unsupported manifest path")?
            .into(),
        stamp,
    ));
    if technology == "Node.js" {
        let config: serde_json::Value =
            serde_json::from_str(&value).context("package.json is invalid JSON")?;
        anyhow::ensure!(config.is_object(), "package.json must be an object");
        let dependency = match name {
            ".next" => Some("next"),
            ".nuxt" => Some("nuxt"),
            ".svelte-kit" => Some("@sveltejs/kit"),
            _ => None,
        };
        if let Some(dep) = dependency {
            anyhow::ensure!(
                ["dependencies", "devDependencies"].iter().any(|key| config
                    .get(key)
                    .and_then(|v| v.get(dep))
                    .is_some_and(|v| v.is_string())),
                "Expected framework dependency is not declared in package.json"
            );
        } else {
            anyhow::ensure!(
                config
                    .get("scripts")
                    .and_then(|s| s.get("build"))
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| !s.trim().is_empty()),
                "A build script must be declared for dist/build review; Coqui never runs it"
            );
        }
        let lock = [
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lock",
        ]
        .iter()
        .find_map(|n| {
            fs::symlink_metadata(project.join(n))
                .is_ok()
                .then(|| project.join(n))
        })
        .context("A tracked package-manager lockfile is required")?;
        let (stamp, _) = manifest(&lock, root, cancel)?;
        manifests.push((
            lock.to_str().context("Unsupported lockfile path")?.into(),
            stamp,
        ));
    }
    let entries = tree(path, &exclusions, cancel, technology == "Python")?;
    // A second independent metadata pass catches directory additions/removals and
    // file edits during inspection. Nothing hashes or reads ignored file contents.
    anyhow::ensure!(
        entries == tree(path, &exclusions, cancel, technology == "Python")?,
        "Folder changed during inspection"
    );
    let file_count = entries.iter().filter(|e| !e.directory).count();
    let file_word = if file_count == 1 { "file" } else { "files" };
    let worktree_word = if repo.worktrees.len() == 1 {
        "worktree"
    } else {
        "worktrees"
    };
    let current = fs::symlink_metadata(path)?;
    let file = CleanupFile {
        path:path.to_str().context("Unsupported output path")?.into(),
        allocated_bytes:entries.iter().filter(|e|!e.directory).map(|e|e.stamp.blocks as f64 * 512.).sum(),
        logical_bytes:entries.iter().filter(|e|!e.directory).map(|e|e.stamp.size as f64).sum(),
        modified_at:current.modified().ok().map(|v|chrono::DateTime::<chrono::Utc>::from(v).to_rfc3339()).unwrap_or_default(),
        origin:format!("{technology} · project {} · tracked {marker} · {file_count} regular {file_word} / {} entries inspected now. Clean confirmed checkout; no tracked output entries; ignored folder; no nested project markers. Git/common directories: {} / {}. {} registered {worktree_word} checked for relationship hazards. Ignored contents were not read; reproducibility requires your review.",project.display(),entries.len(),repo.git_directory.as_deref().unwrap_or(""),repo.common_directory.as_deref().unwrap_or(""),repo.worktrees.len()),
        consequences:format!("{consequence} The entire selected folder and every listed descendant will move to Trash. Close editors, builds, watchers and related tools. Confirm these ignored files contain no unique work and that you can regenerate them. No build/package command is run. Moving to Trash does not immediately free disk space; Coqui never empties Trash."),
    };
    Ok((
        file,
        Seal {
            project: project.to_str().context("Unsupported project path")?.into(),
            git: serde_json::to_string(&repo)?,
            manifests,
            tree: entries,
        },
    ))
}

pub(crate) fn page(path: &str, seal: &Seal, offset: u32) -> ProjectCleanupPage {
    ProjectCleanupPage {
        total: seal.tree.len().saturating_sub(1) as u32,
        entries: seal
            .tree
            .iter()
            .skip(1 + offset as usize)
            .take(50)
            .map(|e| ProjectCleanupEntry {
                path: Path::new(path).join(&e.relative).to_string_lossy().into(),
                directory: e.directory,
                logical_bytes: e.stamp.size as f64,
                allocated_bytes: e.stamp.blocks as f64 * 512.,
            })
            .collect(),
    }
}
pub(crate) fn verify(destination: &Path, seal: &Seal) -> Result<()> {
    // Cancellation stops future moves; once native Trash returns, verify it even
    // when the user has cancelled. Verify metadata and all descendant identities.
    let entries = tree(destination, &[], &AtomicBool::new(false), false)?;
    let mut expected = seal.tree.clone();
    let mut actual = entries;
    // Rename may update the root directory's change timestamp, never its inode.
    for entries in [&mut expected, &mut actual] {
        entries[0].stamp.changed = 0;
        entries[0].stamp.changed_ns = 0;
    }
    anyhow::ensure!(
        expected == actual,
        "Trash folder contents or identity could not be verified"
    );
    Ok(())
}
