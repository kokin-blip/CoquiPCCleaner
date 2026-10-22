use coqui_engine::{database, developer, model::*, storage};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    db: rusqlite::Connection,
}
fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let out = Command::new("/usr/bin/git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "user.name=CoquiFixture",
            "-c",
            "user.email=fixture@invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-C",
        ])
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fixture Git failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir
            .path()
            .canonicalize()
            .unwrap()
            .join("project with spaces");
        fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(
            root.join(".gitignore"),
            "/target/\n/dist/\n/node_modules/\n",
        )
        .unwrap();
        git(&root, &["add", "Cargo.toml", ".gitignore"]);
        git(&root, &["commit", "-m", "fixture"]);
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/cache.bin"), "build fixture").unwrap();
        let db = database::open(&dir.path().join("app.db")).unwrap();
        let mut f = Self {
            _dir: dir,
            root,
            db,
        };
        f.index();
        f
    }
    fn index(&mut self) {
        let mut s = DiagnosticSession {
            id: "scan".into(),
            started_at: now(),
            finished_at: Some(now()),
            status: "complete".into(),
            stage: "Complete".into(),
            scope: self.root.to_string_lossy().into(),
            file_count: 0,
            logical_bytes: 0.,
            allocated_bytes: 0.,
            skipped_count: 0,
            snapshot: None,
            findings: vec![],
            permissions: vec![],
            categories: vec![],
            candidates: vec![],
            errors: vec![],
        };
        database::save_session(&self.db, &s).unwrap();
        let mut rows = vec![];
        let walk = storage::walk(
            &[self.root.clone()],
            &[],
            &AtomicBool::new(false),
            |r| {
                rows.push(r);
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
        database::insert_files(&mut self.db, "scan", &rows).unwrap();
        s.file_count = walk.files;
        database::save_session(&self.db, &s).unwrap();
    }
    fn analyze(&self) -> DeveloperReport {
        developer::analyze(&self.db, "scan", &[], &AtomicBool::new(false), |_| true).unwrap()
    }
}
#[test]
fn real_git_clean_ignored_build_output_has_qualified_review_and_no_writes() {
    let f = Fixture::new();
    let index = fs::read(f.root.join(".git/index")).unwrap();
    let r = f.analyze();
    assert_eq!(r.status, "complete");
    assert_eq!(r.artifacts.len(), 1);
    let a = &r.artifacts[0];
    assert_eq!(a.technology, "Rust");
    assert_eq!(a.git.status, "clean");
    assert_eq!(a.tracked_entries, Some(0));
    assert_eq!(a.assessment, "Review required");
    assert_eq!(a.indexed_file_count, 1);
    assert!(r.qualification.contains("not proof"));
    assert_eq!(fs::read(f.root.join(".git/index")).unwrap(), index);
    assert_eq!(
        fs::read_to_string(f.root.join("target/cache.bin")).unwrap(),
        "build fixture"
    );
    assert_eq!(
        developer::get(&f.db, "scan")
            .unwrap()
            .unwrap()
            .artifacts
            .len(),
        1
    );
}
#[test]
fn dirty_staged_untracked_and_tracked_artifacts_are_protected() {
    let mut f = Fixture::new();
    git(&f.root, &["add", "-f", "target/cache.bin"]);
    git(&f.root, &["commit", "-m", "intentional artifact"]);
    fs::write(f.root.join("Cargo.toml"), "changed fixture").unwrap();
    fs::write(f.root.join("untracked.txt"), "preserve").unwrap();
    fs::write(f.root.join("target/new.bin"), "staged").unwrap();
    git(&f.root, &["add", "-f", "target/new.bin"]);
    f.index();
    let a = f.analyze().artifacts.remove(0);
    let changes = a.git.changes.unwrap();
    assert_eq!(a.git.status, "dirty");
    assert_eq!(changes.modified, 1);
    assert_eq!(changes.staged, 1);
    assert_eq!(changes.untracked, 1);
    assert_eq!(a.tracked_entries, Some(2));
    assert_eq!(a.assessment, "Protected");
}
#[test]
fn linked_detached_locked_and_prunable_worktree_relationships_are_visible() {
    let mut f = Fixture::new();
    let linked = f
        ._dir
        .path()
        .canonicalize()
        .unwrap()
        .join("linked\nworktree");
    git(
        &f.root,
        &["worktree", "add", "--detach", linked.to_str().unwrap()],
    );
    git(&f.root, &["worktree", "lock", linked.to_str().unwrap()]);
    fs::create_dir(linked.join("target")).unwrap();
    fs::write(linked.join("target/keep"), "linked output").unwrap();
    f.root = linked.clone();
    f.index();
    let r = f.analyze();
    let a = &r.artifacts[0];
    assert_eq!(a.git.status, "clean");
    assert_ne!(a.git.git_directory, a.git.common_directory);
    assert_eq!(a.git.worktrees.len(), 2);
    assert!(a
        .git
        .worktrees
        .iter()
        .any(|w| w.path == linked.to_string_lossy() && w.detached && w.locked));
    assert_eq!(a.assessment, "Protected");
    git(&f.root, &["worktree", "unlock", linked.to_str().unwrap()]);
    let original = f
        ._dir
        .path()
        .canonicalize()
        .unwrap()
        .join("project with spaces");
    let moved = f._dir.path().canonicalize().unwrap().join("moved-link");
    fs::rename(&linked, &moved).unwrap();
    f.root = original;
    f.index();
    let r = f.analyze();
    assert!(r.artifacts[0]
        .git
        .worktrees
        .iter()
        .any(|w| w.prunable && !w.present));
    assert_eq!(r.artifacts[0].assessment, "Protected");
}
#[test]
fn nested_projects_and_gitlinks_are_preserved() {
    let mut f = Fixture::new();
    fs::create_dir(f.root.join("target/nested")).unwrap();
    fs::write(
        f.root.join("target/nested/.git"),
        "gitdir: /missing/fixture",
    )
    .unwrap();
    let hash = String::from_utf8(git(&f.root, &["rev-parse", "HEAD"])).unwrap();
    let cacheinfo = format!("160000,{},target/submodule", hash.trim());
    git(
        &f.root,
        &["update-index", "--add", "--cacheinfo", &cacheinfo],
    );
    f.index();
    let a = f.analyze().artifacts.remove(0);
    assert!(a.nested_markers);
    assert_eq!(a.gitlinks, Some(1));
    assert_eq!(a.assessment, "Protected");
}
#[test]
fn broken_git_registration_and_missing_head_remain_unknown_or_protected() {
    let mut f = Fixture::new();
    fs::rename(f.root.join(".git"), f._dir.path().join("original-git")).unwrap();
    fs::write(f.root.join(".git"), "gitdir: /does-not-exist/coqui-fixture").unwrap();
    f.index();
    let a = f.analyze().artifacts.remove(0);
    assert_eq!(a.git.status, "unavailable");
    assert!(a.git.changes.is_none());
    assert_eq!(a.assessment, "Protected");
    fs::remove_file(f.root.join(".git")).unwrap();
    git(&f.root, &["init", "-b", "unborn"]);
    let a = f.analyze().artifacts.remove(0);
    assert_eq!(a.assessment, "Protected");
    assert!(a.git.worktrees.iter().all(|w| w.head.is_none()));
}
#[test]
fn repositories_with_content_filters_do_not_run_status_or_the_filter() {
    let f = Fixture::new();
    let sentinel = f.root.join("filter-was-run");
    git(
        &f.root,
        &[
            "config",
            "filter.fixture.clean",
            &format!("touch '{}'", sentinel.display()),
        ],
    );
    let r = f.analyze();
    assert_eq!(r.artifacts[0].git.status, "unavailable");
    assert!(r.artifacts[0].git.qualification.contains("content filters"));
    assert!(!sentinel.exists());
}
#[test]
fn excluded_disappeared_and_linked_outputs_never_get_reviewed() {
    let f = Fixture::new();
    let r = developer::analyze(
        &f.db,
        "scan",
        &[f.root.join("target")],
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert!(r.artifacts.is_empty());
    assert_eq!(r.status, "partial");
    fs::remove_file(f.root.join("target/cache.bin")).unwrap();
    fs::remove_dir(f.root.join("target")).unwrap();
    std::os::unix::fs::symlink(f._dir.path(), f.root.join("target")).unwrap();
    assert!(f.analyze().artifacts.is_empty());
}
#[test]
fn cancellation_channel_loss_recovery_migration_and_purge() {
    let f = Fixture::new();
    let r = developer::analyze(&f.db, "scan", &[], &AtomicBool::new(false), |_| false).unwrap();
    assert_eq!(r.status, "cancelled");
    assert!(r.artifacts.is_empty());
    let mut interrupted = r;
    interrupted.status = "running".into();
    developer::save(&f.db, &interrupted).unwrap();
    database::recover_interrupted(&f.db).unwrap();
    assert_eq!(
        developer::get(&f.db, "scan").unwrap().unwrap().status,
        "interrupted"
    );
    f.db.execute_batch("DROP TABLE records; DROP TABLE maintenance; DROP TABLE developer_reports; PRAGMA user_version=2;")
        .unwrap();
    let c = database::open(&f._dir.path().join("app.db")).unwrap();
    assert!(developer::get(&c, "scan").unwrap().is_none());
    assert!(database::session(&c, "scan").is_ok());
    developer::save(&c, &interrupted).unwrap();
    database::purge(&c).unwrap();
    assert!(developer::get(&c, "scan").unwrap().is_none());
}
#[test]
fn status_and_worktree_parsers_handle_nul_renames_conflicts_and_reject_truncation() {
    let s = developer::parse_status(
        b"R  new\nname\0old name\0?? untracked directory/\0UU conflict\0 M modified\0",
    )
    .unwrap();
    assert_eq!(s.staged, 1);
    assert_eq!(s.untracked, 1);
    assert_eq!(s.conflicts, 1);
    assert_eq!(s.modified, 1);
    assert!(developer::parse_status(b"R  missing old\0").is_err());
    assert!(developer::parse_status(b"?? truncated").is_err());
    assert!(developer::parse_status(b"ZZ unknown\0").is_err());
    let sha = "a".repeat(40);
    let w=developer::parse_worktrees(format!("worktree /fixture/with\nnewline\0HEAD {sha}\0detached\0locked reason\0prunable reason\0\0").as_bytes()).unwrap();
    assert!(w[0].locked && w[0].detached && w[0].prunable);
    assert!(w[0].path.contains('\n'));
    assert!(developer::parse_worktrees(b"worktree /truncated\0").is_err());
}
#[test]
fn artifact_families_require_markers_and_suppress_nested_dependency_outputs() {
    let mut f = Fixture::new();
    for (dir, proof) in [
        ("node", "package.json"),
        ("dart", "pubspec.yaml"),
        ("python", "pyproject.toml"),
    ] {
        let p = f.root.join(dir);
        fs::create_dir(&p).unwrap();
        fs::write(p.join(proof), "fixture marker").unwrap();
        for child in match dir {
            "node" => vec!["node_modules", "dist"],
            "dart" => vec![".dart_tool", "build"],
            _ => vec![".venv", "__pycache__"],
        } {
            fs::create_dir(p.join(child)).unwrap();
            fs::write(p.join(child).join("file"), "fixture").unwrap();
        }
    }
    fs::create_dir_all(f.root.join("node/node_modules/dependency/dist")).unwrap();
    fs::write(
        f.root.join("node/node_modules/dependency/package.json"),
        "{}",
    )
    .unwrap();
    fs::create_dir(f.root.join("dist")).unwrap();
    f.index();
    let r = f.analyze();
    assert_eq!(r.artifacts.len(), 7);
    assert_eq!(
        r.artifacts
            .iter()
            .filter(|a| a.technology == "Node.js")
            .count(),
        2
    );
    assert!(!r
        .artifacts
        .iter()
        .any(|a| a.path.ends_with("dependency/dist")));
}

#[test]
fn stale_indexes_and_partial_scans_keep_coverage_gaps() {
    let f = Fixture::new();
    f.db.execute("DELETE FROM files WHERE session_id='scan'", [])
        .unwrap();
    let r = f.analyze();
    assert_eq!(r.status, "partial");
    assert!(r.errors.iter().any(|e| e.contains("no longer retained")));
    let mut f = Fixture::new();
    let mut s = database::session(&f.db, "scan").unwrap();
    s.status = "cancelled".into();
    database::save_session(&f.db, &s).unwrap();
    let r = f.analyze();
    assert_eq!(r.status, "partial");
    assert!(r.errors.iter().any(|e| e.contains("incomplete")));
    f.index();
}
#[test]
fn recheck_observes_changed_git_state_and_report_retention_cascades() {
    let f = Fixture::new();
    assert_eq!(f.analyze().artifacts[0].git.status, "clean");
    fs::write(f.root.join("Cargo.toml"), "modified after first review").unwrap();
    let r = f.analyze();
    assert_eq!(r.artifacts[0].git.status, "dirty");
    assert_eq!(r.artifacts[0].assessment, "Protected");
    f.db.execute(
        "UPDATE sessions SET started_at='2000-01-01T00:00:00Z' WHERE id='scan'",
        [],
    )
    .unwrap();
    database::retain(&f.db, 30).unwrap();
    assert!(developer::get(&f.db, "scan").unwrap().is_none());
}
#[test]
fn fsmonitor_configuration_is_disabled_and_repository_metadata_is_unchanged() {
    let f = Fixture::new();
    let sentinel = f.root.join("fsmonitor-was-run");
    git(
        &f.root,
        &[
            "config",
            "core.fsmonitor",
            &format!("touch '{}'", sentinel.display()),
        ],
    );
    let index = fs::read(f.root.join(".git/index")).unwrap();
    let config = fs::read(f.root.join(".git/config")).unwrap();
    assert_eq!(f.analyze().artifacts[0].git.status, "clean");
    assert!(!sentinel.exists());
    assert_eq!(fs::read(f.root.join(".git/index")).unwrap(), index);
    assert_eq!(fs::read(f.root.join(".git/config")).unwrap(), config);
}

#[test]
fn cloud_placeholder_metadata_prevents_git_content_inspection() {
    let f = Fixture::new();
    fs::write(
        f.root.join("remote-only.icloud"),
        "placeholder fixture, do not read",
    )
    .unwrap();
    let r = f.analyze();
    assert_eq!(r.status, "partial");
    assert_eq!(r.artifacts[0].git.status, "unavailable");
    assert!(r.artifacts[0]
        .git
        .qualification
        .contains("cloud placeholders"));
    assert_eq!(r.artifacts[0].assessment, "Protected");
}
