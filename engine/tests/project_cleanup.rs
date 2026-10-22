use coqui_engine::{
    cleanup::{self, Policy},
    database, developer,
    model::*,
    storage,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
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
impl Fixture {
    fn policy(&self) -> Policy {
        Policy {
            home: self._dir.path().canonicalize().unwrap(),
            exclusions: vec![],
        }
    }
    fn preview(&self, path: &Path) -> anyhow::Result<CleanupPlan> {
        cleanup::preview_project(
            &self.db,
            CleanupRequest {
                session_id: "scan".into(),
                paths: vec![path.to_str().unwrap().into()],
            },
            &self.policy(),
            &AtomicBool::new(false),
        )
    }
    fn plan(&self) -> CleanupPlan {
        self.analyze();
        self.preview(&self.root.join("target")).unwrap()
    }
    fn execute(&self, plan: &CleanupPlan) -> CleanupPlan {
        cleanup::execute(
            &self.db,
            approval(plan),
            &self.policy(),
            &AtomicBool::new(false),
            |p| {
                let saved = cleanup::get(&self.db, &plan.id).unwrap();
                assert_eq!(saved.status, "running");
                assert_eq!(saved.items[0].status, "moving");
                assert!(saved.project_cleanup);
                let dest = self._dir.path().join("fixture-trash");
                fs::rename(p, &dest)?;
                Ok(dest)
            },
            |_| true,
        )
        .unwrap()
    }
}
fn approval(plan: &CleanupPlan) -> CleanupApproval {
    CleanupApproval {
        plan_id: plan.id.clone(),
        reviewed_at: plan.created_at.clone(),
        applications_closed: true,
        consequences_accepted: true,
    }
}
#[test]
fn fresh_folder_review_is_read_only_paginated_declined_and_retained() {
    let f = Fixture::new();
    for n in 0..61 {
        fs::write(f.root.join(format!("target/file-{n}.cache")), "cache").unwrap();
    }
    let index = fs::read(f.root.join(".git/index")).unwrap();
    let p = f.plan();
    assert!(p.project_cleanup);
    assert!(p.items[0].file.origin.contains("62 regular files"));
    let first = cleanup::project_entries(&f.db, &p.id, 0).unwrap();
    assert_eq!(first.total, 62);
    assert_eq!(first.entries.len(), 50);
    assert_eq!(
        cleanup::project_entries(&f.db, &p.id, 50)
            .unwrap()
            .entries
            .len(),
        12
    );
    assert_eq!(fs::read(f.root.join(".git/index")).unwrap(), index);
    assert_eq!(
        fs::read_to_string(f.root.join("target/cache.bin")).unwrap(),
        "build fixture"
    );
    assert_eq!(cleanup::decline(&f.db, &p.id).unwrap().status, "declined");
    let declined = cleanup::get(&f.db, &p.id).unwrap();
    assert!(declined.project_cleanup);
    assert_eq!(declined.items[0].status, "declined");
    assert!(declined.items[0]
        .detail
        .contains("Not moved; review declined"));
    assert!(cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("declined"),
        |_| true
    )
    .is_err());
}
#[test]
fn approved_folder_move_verifies_every_descendant_and_cannot_replay() {
    let f = Fixture::new();
    fs::create_dir(f.root.join("target/sub dir")).unwrap();
    fs::write(
        f.root.join("target/sub dir/odd\nname.cache"),
        "preserve bytes",
    )
    .unwrap();
    let p = f.plan();
    let done = f.execute(&p);
    assert_eq!(done.status, "complete");
    assert_eq!(done.items[0].status, "movedToTrash");
    assert!(done.items[0]
        .detail
        .contains("all reviewed descendant identities"));
    assert_eq!(
        fs::read_to_string(f._dir.path().join("fixture-trash/sub dir/odd\nname.cache")).unwrap(),
        "preserve bytes"
    );
    assert!(!f.root.join("target").exists());
    assert!(cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("replay"),
        |_| true
    )
    .is_err());
    assert_eq!(git(&f.root, &["status", "--porcelain"]), b"");
}
#[test]
fn stale_contents_manifest_git_state_or_parent_are_never_moved() {
    for change in 0..6 {
        let f = Fixture::new();
        let p = f.plan();
        match change {
            0 => fs::write(f.root.join("target/cache.bin"), "different useful output").unwrap(),
            1 => fs::write(f.root.join("target/new.cache"), "new output").unwrap(),
            2 => {
                fs::write(f.root.join("Cargo.toml"), "changed manifest").unwrap();
                git(&f.root, &["add", "Cargo.toml"]);
                git(&f.root, &["commit", "-m", "manifest changed"]);
            }
            3 => {
                git(&f.root, &["add", "-f", "target/cache.bin"]);
            }
            4 => {
                fs::write(f.root.join("notes.txt"), "unique untracked work").unwrap();
            }
            _ => {
                fs::rename(f.root.join("target"), f.root.join("old-target")).unwrap();
                fs::create_dir(f.root.join("target")).unwrap();
                fs::write(f.root.join("target/cache.bin"), "build fixture").unwrap();
            }
        }
        let done = cleanup::execute(
            &f.db,
            approval(&p),
            &f.policy(),
            &AtomicBool::new(false),
            |_| panic!("changed target {change}"),
            |_| true,
        )
        .unwrap();
        assert_eq!(done.status, "partial");
        assert_eq!(done.items[0].status, "skipped");
    }
}
#[test]
fn nested_markers_links_hard_links_special_files_cloud_and_permissions_block_folders() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    for kind in 0..7 {
        let f = Fixture::new();
        match kind {
            0 => {
                fs::write(f.root.join("target/.git"), "gitdir: other").unwrap();
            }
            1 => {
                fs::write(f.root.join("target/package.json"), "{}").unwrap();
            }
            2 => symlink(f.root.join("Cargo.toml"), f.root.join("target/link")).unwrap(),
            3 => fs::hard_link(
                f.root.join("target/cache.bin"),
                f.root.join("target/second.cache"),
            )
            .unwrap(),
            4 => fs::write(f.root.join("target/.cache.icloud"), "cloud placeholder").unwrap(),
            5 => fs::set_permissions(
                f.root.join("target/cache.bin"),
                fs::Permissions::from_mode(0o000),
            )
            .unwrap(),
            _ => {
                let name =
                    std::ffi::CString::new(f.root.join("target/pipe").to_str().unwrap()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
            }
        }
        f.analyze();
        assert!(f.preview(&f.root.join("target")).is_err(), "kind {kind}");
    }
}
#[test]
fn new_nested_project_or_exclusion_after_preview_prevents_execution() {
    for exclude in [true, false] {
        let f = Fixture::new();
        let p = f.plan();
        let mut policy = f.policy();
        if exclude {
            policy.exclusions.push(f.root.join("target/cache.bin"));
        } else {
            fs::write(f.root.join("target/pyproject.toml"), "[project]").unwrap();
        }
        let done = cleanup::execute(
            &f.db,
            approval(&p),
            &policy,
            &AtomicBool::new(false),
            |_| panic!("protected"),
            |_| true,
        )
        .unwrap();
        assert_eq!(done.items[0].status, "skipped");
    }
}
#[test]
fn tracked_nonignored_and_unborn_outputs_require_manual_review() {
    for kind in 0..3 {
        let f = Fixture::new();
        match kind {
            0 => {
                git(&f.root, &["add", "-f", "target/cache.bin"]);
                git(&f.root, &["commit", "-m", "tracked output"]);
            }
            1 => {
                fs::write(f.root.join(".gitignore"), "").unwrap();
                git(&f.root, &["add", ".gitignore"]);
                git(&f.root, &["commit", "-m", "no ignore"]);
            }
            _ => {
                git(&f.root, &["checkout", "--orphan", "unborn"]);
            }
        }
        f.analyze();
        assert!(f.preview(&f.root.join("target")).is_err());
    }
}
#[test]
fn linked_worktree_is_supported_but_changed_registration_or_lock_blocks_a_review() {
    let mut f = Fixture::new();
    let linked = f
        ._dir
        .path()
        .canonicalize()
        .unwrap()
        .join("linked checkout");
    git(
        &f.root,
        &["worktree", "add", "--detach", linked.to_str().unwrap()],
    );
    fs::create_dir(linked.join("target")).unwrap();
    fs::write(linked.join("target/cache.bin"), "linked output").unwrap();
    let main = f.root.clone();
    f.root = linked.clone();
    f.index();
    let p = f.plan();
    git(&main, &["worktree", "lock", linked.to_str().unwrap()]);
    let done = cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("locked"),
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "skipped");
    git(&main, &["worktree", "unlock", linked.to_str().unwrap()]);
    let p = f.plan();
    assert_eq!(f.execute(&p).status, "complete");
}
#[test]
fn broken_or_filtered_git_never_grants_folder_authority() {
    for kind in 0..2 {
        let f = Fixture::new();
        let p = f.plan();
        if kind == 0 {
            git(
                &f.root,
                &["config", "filter.custom.clean", "touch SHOULD-NOT-RUN"],
            );
        } else {
            fs::rename(f.root.join(".git"), f.root.join(".git-disabled")).unwrap();
            fs::write(f.root.join(".git"), "gitdir: missing").unwrap();
        }
        let done = cleanup::execute(
            &f.db,
            approval(&p),
            &f.policy(),
            &AtomicBool::new(false),
            |_| panic!("unknown Git"),
            |_| true,
        )
        .unwrap();
        assert_eq!(done.items[0].status, "skipped");
        assert!(!f.root.join("SHOULD-NOT-RUN").exists());
    }
}
#[test]
fn supported_node_build_requires_declared_build_tracked_lock_and_unchanged_configuration() {
    let mut f = Fixture::new();
    fs::write(
        f.root.join("package.json"),
        r#"{"scripts":{"build":"tool build"}}"#,
    )
    .unwrap();
    fs::create_dir(f.root.join("dist")).unwrap();
    fs::write(f.root.join("dist/result.js"), "generated").unwrap();
    git(&f.root, &["add", "package.json"]);
    git(&f.root, &["commit", "-m", "node"]);
    f.index();
    f.analyze();
    assert!(f.preview(&f.root.join("dist")).is_err());
    fs::write(f.root.join("package-lock.json"), "{}").unwrap();
    git(&f.root, &["add", "package-lock.json"]);
    git(&f.root, &["commit", "-m", "lock"]);
    let p = f.preview(&f.root.join("dist")).unwrap();
    assert!(p.items[0].file.origin.contains("Node.js"));
    assert_eq!(f.execute(&p).status, "complete");
}
#[test]
fn flutter_build_and_python_bytecode_have_distinct_manifest_and_content_policies() {
    for python in [false, true] {
        let mut f = Fixture::new();
        let (marker, folder, file) = if python {
            ("pyproject.toml", "__pycache__", "source.cpython-313.pyc")
        } else {
            ("pubspec.yaml", "build", "generated.cache")
        };
        fs::write(
            f.root.join(marker),
            if python {
                "[project]\nname='fixture'"
            } else {
                "name: fixture"
            },
        )
        .unwrap();
        fs::write(f.root.join(".gitignore"), format!("/target/\n/{folder}/\n")).unwrap();
        git(&f.root, &["add", marker, ".gitignore"]);
        git(&f.root, &["commit", "-m", "technology fixture"]);
        fs::create_dir(f.root.join(folder)).unwrap();
        fs::write(f.root.join(folder).join(file), "generated").unwrap();
        f.index();
        f.analyze();
        let p = f.preview(&f.root.join(folder)).unwrap();
        if python {
            fs::write(f.root.join(folder).join("unique.py"), "valuable source").unwrap();
            assert_eq!(f.execute(&p).items[0].status, "skipped");
        } else {
            assert_eq!(f.execute(&p).status, "complete");
        }
    }
}
#[test]
fn dependencies_and_environments_remain_analysis_only_even_when_ignored() {
    for (marker, folder) in [
        ("package.json", "node_modules"),
        ("pyproject.toml", ".venv"),
    ] {
        let mut f = Fixture::new();
        fs::write(f.root.join(marker), "{}").unwrap();
        fs::write(f.root.join(".gitignore"), format!("/target/\n/{folder}/\n")).unwrap();
        git(&f.root, &["add", marker, ".gitignore"]);
        git(&f.root, &["commit", "-m", "deps"]);
        fs::create_dir(f.root.join(folder)).unwrap();
        fs::write(f.root.join(folder).join("local-edit"), "unique").unwrap();
        f.index();
        f.analyze();
        assert!(f.preview(&f.root.join(folder)).is_err());
    }
}
#[test]
fn partial_scan_stale_index_and_ambiguous_requests_cannot_create_project_approval() {
    let f = Fixture::new();
    f.analyze();
    let path = f.root.join("target").to_str().unwrap().to_owned();
    assert!(cleanup::preview(
        &f.db,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![path.clone()]
        },
        &f.policy()
    )
    .is_err());
    assert!(cleanup::preview_project(
        &f.db,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![path.clone(), path.clone()]
        },
        &f.policy(),
        &AtomicBool::new(false)
    )
    .is_err());
    let mut session = database::session(&f.db, "scan").unwrap();
    session.status = "partial".into();
    database::save_session(&f.db, &session).unwrap();
    assert!(f.preview(Path::new(&path)).is_err());
    session.status = "complete".into();
    database::save_session(&f.db, &session).unwrap();
    f.db.execute("DELETE FROM files", []).unwrap();
    assert!(f.preview(Path::new(&path)).is_err());
}
#[test]
fn cancellation_channel_loss_and_interruption_never_retry_folder_moves() {
    let f = Fixture::new();
    let p = f.plan();
    let cancel = AtomicBool::new(false);
    let result = cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &cancel,
        |_| panic!("cancel"),
        |_| {
            cancel.store(true, Ordering::SeqCst);
            false
        },
    )
    .unwrap();
    assert_eq!(result.status, "cancelled");
    assert!(cleanup::preview_project(
        &f.db,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![f.root.join("target").to_str().unwrap().into()]
        },
        &f.policy(),
        &AtomicBool::new(true)
    )
    .is_err());
    let p = f.plan();
    let raw: String =
        f.db.query_row("SELECT json FROM cleanup_plans WHERE id=?1", [&p.id], |r| {
            r.get(0)
        })
        .unwrap();
    let mut record: serde_json::Value = serde_json::from_str(&raw).unwrap();
    record["plan"]["status"] = "running".into();
    record["plan"]["items"][0]["status"] = "moving".into();
    f.db.execute(
        "UPDATE cleanup_plans SET status='running',json=?1 WHERE id=?2",
        rusqlite::params![record.to_string(), p.id],
    )
    .unwrap();
    database::recover_interrupted(&f.db).unwrap();
    let retained = cleanup::get(&f.db, &p.id).unwrap();
    assert_eq!(retained.status, "interrupted");
    assert_eq!(retained.items[0].status, "unverified");
    assert!(f.root.join("target/cache.bin").exists());
}
#[test]
fn uncertain_folder_moves_fail_verification_and_native_failures_are_journaled() {
    for fail in [false, true] {
        let f = Fixture::new();
        let p = f.plan();
        let done = cleanup::execute(
            &f.db,
            approval(&p),
            &f.policy(),
            &AtomicBool::new(false),
            |source| {
                if fail {
                    anyhow::bail!("fixture permission failure");
                }
                let dest = f._dir.path().join("fixture-trash");
                fs::rename(source, &dest)?;
                fs::write(dest.join("new output"), "race after native return")?;
                Ok(dest)
            },
            |_| true,
        )
        .unwrap();
        assert_eq!(done.status, "partial");
        assert_eq!(
            done.items[0].status,
            if fail { "failed" } else { "unverified" }
        );
    }
}
#[test]
fn folder_plans_reopen_and_purge_with_app_data() {
    let f = Fixture::new();
    let p = f.plan();
    let db = database::open(&f._dir.path().join("app.db")).unwrap();
    assert!(cleanup::get(&db, &p.id).unwrap().project_cleanup);
    assert_eq!(cleanup::project_entries(&db, &p.id, 0).unwrap().total, 1);
    database::purge(&db).unwrap();
    assert!(cleanup::get(&db, &p.id).is_err());
    assert!(f.root.join("target/cache.bin").exists());
}
#[cfg(target_os = "macos")]
#[test]
#[ignore = "Explicit native validation; moves only a new disposable Git fixture folder to native Trash and restores it"]
fn native_project_folder_trash_verification_and_fixture_restore() {
    let f = Fixture::new();
    let p = f.plan();
    let done = cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        cleanup::native_trash,
        |_| true,
    )
    .unwrap();
    let item = &done.items[0];
    if let Some(dest) = &item.trash_path {
        if Path::new(dest).exists() && !f.root.join("target").exists() {
            fs::rename(dest, f.root.join("target")).unwrap();
        }
    }
    assert_eq!(item.status, "movedToTrash", "{}", item.detail);
    assert_eq!(
        fs::read_to_string(f.root.join("target/cache.bin")).unwrap(),
        "build fixture"
    );
}
#[test]
fn fresh_inspection_has_a_hard_entry_limit() {
    let f = Fixture::new();
    for i in 0..20_000 {
        fs::write(f.root.join(format!("target/{i}.cache")), b"").unwrap();
    }
    f.analyze();
    let err = f.preview(&f.root.join("target")).unwrap_err().to_string();
    assert!(
        err.contains("limit") || err.contains("fully inspected"),
        "{err}"
    );
}
#[test]
fn native_failure_after_cancellation_is_not_labeled_as_an_unattempted_move() {
    let f = Fixture::new();
    let p = f.plan();
    let cancel = AtomicBool::new(false);
    let done = cleanup::execute(
        &f.db,
        approval(&p),
        &f.policy(),
        &cancel,
        |_| {
            cancel.store(true, Ordering::SeqCst);
            anyhow::bail!("fixture native failure after starting")
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "failed");
    assert_eq!(done.status, "partial");
    assert!(done.items[0].detail.contains("Inspect the source location"));
}
