//! Read-only developer-output review. No result from this module authorizes Trash.
use crate::{database, model::*, storage};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const LIMIT: usize = 250;
const OUTPUT_LIMIT: u64 = 2 * 1024 * 1024;
const QUALIFICATION: &str = "Review only. Sizes and nested-marker coverage come from the retained scan index, not a fresh recursive traversal or guaranteed recoverable space. Git checks describe the current worktree only; submodule contents, ignored file contents and other worktrees' dirty state are not inspected. A clean Git status or ignored path is not proof that files are disposable. Recheck after changes. Project cleanup is not enabled by this report.";

fn excluded(path: &Path, exclusions: &[PathBuf]) -> bool {
    exclusions
        .iter()
        .any(|e| path.starts_with(e) || e.starts_with(path))
}
fn available(path: &Path, exclusions: &[PathBuf]) -> Result<()> {
    anyhow::ensure!(
        !excluded(path, exclusions),
        "Location intersects an exclusion"
    );
    let m = fs::symlink_metadata(path)?;
    anyhow::ensure!(
        m.is_dir() && m.uid() == unsafe { libc::geteuid() },
        "Not a user-owned directory"
    );
    anyhow::ensure!(
        path.canonicalize()? == path && !storage::placeholder(path, &m),
        "Linked, cloud-only or non-canonical location"
    );
    for a in path.ancestors() {
        anyhow::ensure!(
            !fs::symlink_metadata(a)?.file_type().is_symlink(),
            "Linked ancestor"
        );
    }
    Ok(())
}
fn marker(parent: &Path, name: &str) -> bool {
    fs::symlink_metadata(parent.join(name)).is_ok_and(|m| {
        m.is_file() && !m.file_type().is_symlink() && !storage::placeholder(&parent.join(name), &m)
    })
}
fn identify(path: &Path) -> Option<(&'static str, &'static str, PathBuf, String, &'static str)> {
    let parent = path.parent()?;
    let name = path.file_name()?.to_str()?;
    let (tech, kind, proof, consequence) = match name {
        "target" if marker(parent, "Cargo.toml") => ("Rust", "Build output", "Cargo.toml", "Rebuilding can be expensive. Build scripts may place useful generated files here; a directory name alone does not establish disposability."),
        "node_modules" if marker(parent, "package.json") => ("Node.js", "Dependencies", "package.json", "Dependencies must be restored with the project's package manager. Local edits, linked packages and offline-only dependencies may be lost by removal."),
        ".next" | ".nuxt" | ".svelte-kit" | "dist" if marker(parent, "package.json") => ("Node.js", "Build output", "package.json", "Rebuild with the project's own tooling. Output may include deployed assets or files that cannot be reproduced without external services."),
        ".dart_tool" | "build" if marker(parent, "pubspec.yaml") => ("Dart / Flutter", "Tool / build output", "pubspec.yaml", "Package resolution and a rebuild will be required. Generated sources and offline artifacts require manual review."),
        "build" if marker(parent, "package.json") => ("Node.js", "Build output", "package.json", "Confirm this is generated output rather than project source or published assets before considering removal."),
        ".venv" | "venv" | "__pycache__" => {
            let owner = parent.ancestors().take(12).find_map(|p| ["pyproject.toml", "requirements.txt", "setup.py"].into_iter().find(|n| marker(p,n)).map(|n|(p,n)))?;
            return Some(("Python", if name == "__pycache__" {"Bytecode cache"} else {"Environment"}, owner.0.to_owned(), owner.1.into(), "Recreate the environment with project tooling. Installed packages, local edits, interpreter versions and offline dependencies may not be reproducible."));
        }
        "DerivedData" if path.ends_with("Library/Developer/Xcode/DerivedData") => ("Xcode", "Derived data", "Xcode-style DerivedData path (project association unconfirmed)", "Xcode may need to rebuild and reindex. Archives and custom output paths are not included; association with a particular project is unconfirmed."),
        _ => return None,
    };
    Some((tech, kind, parent.to_owned(), proof.into(), consequence))
}

/// Fixed executable, fixed read-only verbs, no inherited Git overrides, no hooks,
/// fsmonitor, optional index writes, recursive submodules, shell or network.
fn git(path: &Path, args: &[&str], cancel: &AtomicBool) -> Result<Vec<u8>> {
    anyhow::ensure!(
        ["rev-parse", "status", "worktree", "ls-files", "config"].contains(&args[0]),
        "Unsupported Git operation"
    );
    let mut child = Command::new("/usr/bin/git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .args([
            "--no-optional-locks",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "submodule.recurse=false",
            "-C",
        ])
        .arg(path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().context("Git output unavailable")?;
    let reader = std::thread::spawn(move || {
        let mut out = vec![];
        stdout
            .take(OUTPUT_LIMIT + 1)
            .read_to_end(&mut out)
            .map(|_| out)
    });
    let start = Instant::now();
    loop {
        if cancel.load(Ordering::SeqCst) || start.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            anyhow::bail!("Git inspection cancelled or timed out; state is unknown");
        }
        if let Some(status) = child.try_wait()? {
            let output = reader
                .join()
                .map_err(|_| anyhow::anyhow!("Git reader failed"))??;
            anyhow::ensure!(
                status.success(),
                "Git could not validate this repository; state is unknown"
            );
            anyhow::ensure!(
                output.len() as u64 <= OUTPUT_LIMIT,
                "Git output exceeded the inspection limit; state is unknown"
            );
            return Ok(output);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn git_path(path: &Path, arg: &str, cancel: &AtomicBool) -> Result<PathBuf> {
    let bytes = git(path, &["rev-parse", "--path-format=absolute", arg], cancel)?;
    let text = std::str::from_utf8(&bytes)?
        .strip_suffix('\n')
        .context("Malformed Git path")?;
    anyhow::ensure!(!text.is_empty(), "Empty Git path");
    Ok(PathBuf::from(text))
}
pub fn parse_status(bytes: &[u8]) -> Result<GitChanges> {
    anyhow::ensure!(
        bytes.is_empty() || bytes.last() == Some(&0),
        "Truncated Git status"
    );
    let mut changes = GitChanges::default();
    let mut parts = bytes.split(|b| *b == 0).peekable();
    while let Some(p) = parts.next() {
        if p.is_empty() {
            anyhow::ensure!(parts.peek().is_none(), "Malformed Git status");
            break;
        }
        anyhow::ensure!(p.len() >= 4 && p[2] == b' ', "Malformed Git status");
        let (x, y) = (p[0], p[1]);
        anyhow::ensure!(
            b" MADRCUT?!".contains(&x) && b" MADRCUT?!".contains(&y),
            "Unknown Git status"
        );
        if x == b'?' && y == b'?' {
            changes.untracked += 1;
        } else if x == b'U' || y == b'U' || &p[..2] == b"AA" || &p[..2] == b"DD" {
            changes.conflicts += 1;
        } else {
            if x != b' ' {
                changes.staged += 1;
            }
            if y != b' ' {
                changes.modified += 1;
            }
        }
        if [x, y].iter().any(|b| [b'R', b'C'].contains(b)) {
            anyhow::ensure!(
                parts.next().is_some_and(|v| !v.is_empty()),
                "Missing renamed path"
            );
        }
    }
    Ok(changes)
}
pub fn parse_worktrees(bytes: &[u8]) -> Result<Vec<GitWorktree>> {
    anyhow::ensure!(bytes.ends_with(&[0, 0]), "Truncated worktree list");
    let mut result = vec![];
    let mut current: Option<GitWorktree> = None;
    for raw in bytes.split(|b| *b == 0) {
        let v = std::str::from_utf8(raw)?;
        if v.is_empty() {
            if let Some(w) = current.take() {
                result.push(w);
            }
            continue;
        }
        if let Some(path) = v.strip_prefix("worktree ") {
            anyhow::ensure!(
                current.is_none() && Path::new(path).is_absolute(),
                "Malformed worktree path"
            );
            current = Some(GitWorktree {
                path: path.into(),
                head: None,
                branch: None,
                detached: false,
                bare: false,
                locked: false,
                prunable: false,
                present: Path::new(path).is_dir(),
            });
        } else {
            let w = current
                .as_mut()
                .context("Worktree attribute without a path")?;
            if let Some(h) = v.strip_prefix("HEAD ") {
                anyhow::ensure!(
                    [40, 64].contains(&h.len()) && h.bytes().all(|b| b.is_ascii_hexdigit()),
                    "Malformed HEAD"
                );
                if h.bytes().any(|b| b != b'0') {
                    w.head = Some(h.into());
                }
            } else if let Some(b) = v.strip_prefix("branch ") {
                w.branch = Some(b.into());
            } else if v == "detached" {
                w.detached = true;
            } else if v == "bare" {
                w.bare = true;
            } else if v == "locked" || v.starts_with("locked ") {
                w.locked = true;
            } else if v == "prunable" || v.starts_with("prunable ") {
                w.prunable = true;
            } else {
                anyhow::bail!("Unknown worktree attribute");
            }
        }
    }
    anyhow::ensure!(!result.is_empty(), "No registered worktrees reported");
    Ok(result)
}
fn unknown(reason: String) -> GitValidation {
    GitValidation {
        status: "unavailable".into(),
        root: None,
        git_directory: None,
        common_directory: None,
        changes: None,
        worktrees: vec![],
        qualification: reason,
    }
}
// Git status may hash changed tracked files. Refuse it when a bounded,
// metadata-only traversal finds cloud placeholders or incomplete coverage.
fn metadata_guard(root: &Path, cancel: &AtomicBool) -> Result<()> {
    let start = Instant::now();
    for (count, entry) in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .enumerate()
    {
        anyhow::ensure!(
            !cancel.load(Ordering::SeqCst)
                && count < 50_000
                && start.elapsed() < Duration::from_secs(3),
            "Repository metadata guard cancelled or exceeded its inspection limit"
        );
        let entry = entry.context("Repository metadata coverage is incomplete")?;
        let m = entry.metadata()?;
        anyhow::ensure!(
            !storage::placeholder(entry.path(), &m),
            "Repository contains cloud placeholders; Git status was not run"
        );
    }
    Ok(())
}
fn inspect_repository(
    project: &Path,
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
) -> GitValidation {
    let discover = (|| -> Result<Option<PathBuf>> {
        for p in project.ancestors() {
            let dot = p.join(".git");
            match fs::symlink_metadata(&dot) {
                Ok(m) => {
                    anyhow::ensure!(
                        !m.file_type().is_symlink()
                            && (m.is_dir() || m.is_file())
                            && !excluded(&dot, exclusions)
                            && !storage::placeholder(&dot, &m),
                        "Git metadata is linked or excluded"
                    );
                    return Ok(Some(p.to_owned()));
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => anyhow::bail!("Git marker could not be inspected"),
            }
            if [".hg", ".svn", ".jj"]
                .iter()
                .any(|n| fs::symlink_metadata(p.join(n)).is_ok())
            {
                anyhow::bail!(
                    "A non-Git repository was detected; validation requires its native tooling"
                );
            }
            if marker(p, "HEAD") && p.join("objects").is_dir() {
                anyhow::bail!(
                    "Possible bare repository; no output cleanup assessment is available"
                );
            }
        }
        Ok(None)
    })();
    let parent=match discover {
        Ok(Some(p))=>p,
        Ok(None)=>return GitValidation {status:"notDetected".into(),qualification:"No supported repository marker detected in project ancestry. Repository ownership and recovery are unconfirmed.".into(),..unknown(String::new())},
        Err(e)=>return unknown(e.to_string()),
    };
    let result = (|| -> Result<GitValidation> {
        available(&parent, exclusions)?;
        metadata_guard(&parent, cancel)?;
        let root = git_path(&parent, "--show-toplevel", cancel)?;
        anyhow::ensure!(
            root == parent,
            "Git root does not match the discovered project boundary"
        );
        let dir = git_path(&parent, "--absolute-git-dir", cancel)?;
        let common = git_path(&parent, "--git-common-dir", cancel)?;
        available(&dir, exclusions)?;
        available(&common, exclusions)?;
        if !common.starts_with(&parent) {
            metadata_guard(&common, cancel)?;
        }
        let worktrees = parse_worktrees(&git(
            &parent,
            &["worktree", "list", "--porcelain", "-z"],
            cancel,
        )?)?;
        let registered = worktrees
            .iter()
            .find(|w| Path::new(&w.path).canonicalize().ok().as_ref() == Some(&root))
            .context("Current checkout is not a registered worktree")?;
        anyhow::ensure!(
            !registered.bare && !registered.prunable && registered.present,
            "Worktree registration is unavailable or prunable"
        );
        let keys = git(
            &parent,
            &["config", "--null", "--name-only", "--list"],
            cancel,
        )?;
        anyhow::ensure!(
            !keys.split(|b| *b == 0).any(|k| k.starts_with(b"filter.")),
            "Repository content filters require manual Git inspection; no status command was run"
        );
        let changes = parse_status(&git(
            &parent,
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=normal",
                "--ignore-submodules=all",
            ],
            cancel,
        )?)?;
        let clean = changes.staged + changes.modified + changes.untracked + changes.conflicts == 0;
        Ok(GitValidation {status:if clean {"clean"} else {"dirty"}.into(),root:Some(root.to_str().context("Unsupported Git root")?.into()),git_directory:Some(dir.to_str().context("Unsupported Git directory")?.into()),common_directory:Some(common.to_str().context("Unsupported common directory")?.into()),changes:Some(changes),worktrees,qualification:"Read-only Git porcelain inspection. Untracked entries may represent directories. Ignored contents and submodule dirty state are not included. Only the current worktree's status was checked; other registered worktrees are listed, not recursively inspected.".into()})
    })();
    result.unwrap_or_else(|e| unknown(e.to_string()))
}
fn tracked(path: &Path, repo: &GitValidation, cancel: &AtomicBool) -> Result<(u32, u32)> {
    let root = Path::new(
        repo.root
            .as_ref()
            .context("Repository validation unavailable")?,
    );
    let relative = path
        .strip_prefix(root)?
        .to_str()
        .context("Unsupported artifact path")?;
    let out = git(
        root,
        &["ls-files", "--cached", "--stage", "-z", "--", relative],
        cancel,
    )?;
    anyhow::ensure!(
        out.is_empty() || out.last() == Some(&0),
        "Truncated Git index output"
    );
    let mut count = 0;
    let mut links = 0;
    for p in out.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let tab = p
            .iter()
            .position(|b| *b == b'\t')
            .context("Malformed Git index entry")?;
        let meta = std::str::from_utf8(&p[..tab])?
            .split(' ')
            .collect::<Vec<_>>();
        anyhow::ensure!(
            meta.len() == 3 && ["0", "1", "2", "3"].contains(&meta[2]),
            "Malformed Git index stage"
        );
        count += 1;
        if meta[0] == "160000" {
            links += 1;
        }
    }
    Ok((count, links))
}
pub fn save(c: &Connection, report: &DeveloperReport) -> Result<()> {
    c.execute("INSERT INTO developer_reports VALUES(?1,?2,?3,?4) ON CONFLICT(session_id) DO UPDATE SET checked_at=excluded.checked_at,status=excluded.status,json=excluded.json",params![report.session_id,report.checked_at,report.status,serde_json::to_string(report)?])?;
    Ok(())
}
pub fn get(c: &Connection, id: &str) -> Result<Option<DeveloperReport>> {
    let json: Option<String> = c
        .query_row(
            "SELECT json FROM developer_reports WHERE session_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    json.map(|v| Ok(serde_json::from_str(&v)?)).transpose()
}
pub fn analyze(
    c: &Connection,
    session_id: &str,
    exclusions: &[PathBuf],
    cancel: &AtomicBool,
    mut emit: impl FnMut(DeveloperReport) -> bool,
) -> Result<DeveloperReport> {
    let s = database::session(c, session_id)?;
    anyhow::ensure!(s.status != "running", "Wait for the scan to finish");
    let mut report = DeveloperReport {
        session_id: session_id.into(),
        checked_at: now(),
        status: "running".into(),
        examined: 0,
        candidate_count: 0,
        artifacts: vec![],
        errors: vec![],
        qualification: QUALIFICATION.into(),
    };
    let mut q=c.prepare("SELECT path FROM files WHERE session_id=?1 AND directory=1 AND name IN ('target','node_modules','.next','.nuxt','.svelte-kit','dist','build','.dart_tool','.venv','venv','__pycache__','DerivedData') ORDER BY length(path),path LIMIT 2001")?;
    let paths = q
        .query_map([session_id], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    report.candidate_count = paths.len().min(2000) as u32;
    let indexed: u32 = c.query_row(
        "SELECT count(*) FROM files WHERE session_id=?1",
        [session_id],
        |r| r.get(0),
    )?;
    if indexed == 0 && s.file_count > 0 {
        report.errors.push("This scan's file index is no longer retained. Rescan the project folder before checking artifacts.".into());
    }
    if paths.len() > 2000 {
        report.errors.push("Discovery capped at 2,000 indexed output-like directories. Select a narrower scan scope.".into());
    }
    save(c, &report)?;
    let mut connected = emit(report.clone());
    let mut repositories = HashMap::new();
    let start = Instant::now();
    for value in paths.into_iter().take(2000) {
        if cancel.load(Ordering::SeqCst) || !connected {
            report.status = "cancelled".into();
            break;
        }
        if start.elapsed() > Duration::from_secs(120) || report.artifacts.len() >= LIMIT {
            report
                .errors
                .push("Inspection limit reached. Select a narrower scan scope and recheck.".into());
            break;
        }
        report.examined += 1;
        let path = Path::new(&value);
        if report.artifacts.iter().any(|a| path.starts_with(&a.path)) {
            continue;
        }
        if let Err(e) = available(path, exclusions) {
            report.errors.push(format!("Not inspected: {value}: {e}"));
            continue;
        }
        let Some((technology, kind, project, proof, consequences)) = identify(path) else {
            continue;
        };
        if let Err(e) = available(&project, exclusions) {
            report.errors.push(format!("Not inspected: {value}: {e}"));
            continue;
        }
        let prefix = format!("{value}/");
        let (count,logical,allocated):(u32,f64,f64)=c.query_row("SELECT count(*),coalesce(sum(logical),0),coalesce(sum(allocated),0) FROM files WHERE session_id=?1 AND directory=0 AND substr(path,1,length(?2))=?2",params![session_id,prefix],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let nested:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1 AND name IN ('.git','.hg','.svn','.jj','Cargo.toml','package.json','pubspec.yaml','pyproject.toml','pnpm-workspace.yaml','lerna.json') AND substr(path,1,length(?2))=?2)",params![session_id,prefix],|r|r.get(0))?;
        let repo = repositories
            .entry(project.clone())
            .or_insert_with(|| inspect_repository(&project, exclusions, cancel))
            .clone();
        let mut reasons = vec![];
        if repo.status == "unavailable" {
            let gap = format!(
                "Git unavailable for {}: {}",
                project.display(),
                repo.qualification
            );
            if !report.errors.contains(&gap) {
                report.errors.push(gap);
            }
        }
        if repo.status != "clean" {
            reasons.push(format!(
                "Repository status: {}. {}",
                repo.status, repo.qualification
            ));
        }
        let (entries, links) = match tracked(path, &repo, cancel) {
            Ok((a, b)) => (Some(a), Some(b)),
            Err(e) => {
                reasons.push(e.to_string());
                if repo.root.is_some() {
                    report
                        .errors
                        .push(format!("Git index unavailable for {value}: {e}"));
                }
                (None, None)
            }
        };
        if entries.is_some_and(|n| n > 0) {
            reasons.push("Git tracks entries inside this output, including any staged/committed artifacts. Preserve them.".into());
        }
        if nested {
            reasons.push("The scan index contains nested project/repository markers inside this output. Preserve and inspect the nested project.".into());
        }
        if repo
            .worktrees
            .iter()
            .any(|w| w.locked || w.prunable || Path::new(&w.path).starts_with(path))
        {
            reasons.push("Registered worktree relationships include locked/prunable entries or a checkout inside this output. Inspect relationships manually.".into());
        }
        if repo.worktrees.iter().any(|w| w.head.is_none() && !w.bare) {
            reasons.push(
                "A registered worktree has no confirmed commit; recovery is unconfirmed.".into(),
            );
        }
        let assessment = if reasons.is_empty() {
            "Review required"
        } else {
            "Protected"
        }
        .into();
        if reasons.is_empty() {
            reasons.push("No tracked entries or indexed nested markers detected. Review ignored/untracked contents and reproducibility with project tooling; no cleanup approval is granted.".into());
        }
        report.artifacts.push(DeveloperArtifact {
            path: value,
            project_root: project.to_string_lossy().into(),
            technology: technology.into(),
            kind: kind.into(),
            marker: proof,
            indexed_allocated_bytes: allocated,
            indexed_logical_bytes: logical,
            indexed_file_count: count,
            scanned_at: s.finished_at.clone().unwrap_or(s.started_at.clone()),
            checked_at: now(),
            git: repo,
            tracked_entries: entries,
            gitlinks: links,
            nested_markers: nested,
            assessment,
            reasons,
            consequences: consequences.into(),
        });
        save(c, &report)?;
        connected = emit(report.clone());
    }
    if cancel.load(Ordering::SeqCst) || !connected {
        report.status = "cancelled".into();
    } else if report.status == "running" {
        report.status = if report.errors.is_empty() && s.status == "complete" {
            "complete"
        } else {
            "partial"
        }
        .into();
    }
    if s.status != "complete" {
        report.errors.push(
            "Source scan has partial or interrupted coverage; indexed sizes are incomplete.".into(),
        );
    }
    save(c, &report)?;
    let _ = emit(report.clone());
    Ok(report)
}
