use coqui_engine::{
    cleanup::{self, Policy},
    database,
    model::*,
    storage,
};
use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

struct Fixture {
    _dir: tempfile::TempDir,
    c: Connection,
    home: PathBuf,
    files: Vec<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().canonicalize().unwrap();
        let cache = home.join("Library/Caches/Example");
        fs::create_dir_all(&cache).unwrap();
        let files = vec![cache.join("one.cache"), cache.join("two.cache")];
        for (i, p) in files.iter().enumerate() {
            fs::write(p, format!("useful cached content {i}")).unwrap();
        }
        let mut c = database::open(&home.join("app.sqlite3")).unwrap();
        let mut s = DiagnosticSession {
            id: "scan".into(),
            started_at: now(),
            finished_at: Some(now()),
            status: "complete".into(),
            stage: "Complete".into(),
            scope: home.to_string_lossy().into(),
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
        database::save_session(&c, &s).unwrap();
        let mut rows = vec![];
        let walked = storage::walk(
            &[home.join("Library/Caches")],
            &[],
            &AtomicBool::new(false),
            |r| {
                rows.push(r);
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
        database::insert_files(&mut c, "scan", &rows).unwrap();
        s.file_count = walked.files;
        database::save_session(&c, &s).unwrap();
        Self {
            _dir: dir,
            c,
            home,
            files,
        }
    }
    fn policy(&self) -> Policy {
        Policy {
            home: self.home.clone(),
            exclusions: vec![],
        }
    }
    fn plan(&self) -> CleanupPlan {
        cleanup::preview(
            &self.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths: self
                    .files
                    .iter()
                    .map(|p| p.to_string_lossy().into())
                    .collect(),
            },
            &self.policy(),
        )
        .unwrap()
    }
    fn approval(plan: &CleanupPlan) -> CleanupApproval {
        CleanupApproval {
            plan_id: plan.id.clone(),
            reviewed_at: plan.created_at.clone(),
            applications_closed: true,
            consequences_accepted: true,
        }
    }
    fn trash(&self, p: &Path) -> anyhow::Result<PathBuf> {
        let d = self.home.join("fixture-trash");
        fs::create_dir_all(&d)?;
        let target = d.join(p.file_name().unwrap());
        fs::rename(p, &target)?;
        Ok(target)
    }
}
#[test]
fn preview_is_non_mutating_and_requires_indexed_unique_files() {
    let f = Fixture::new();
    let p = f.plan();
    assert_eq!(p.items.len(), 2);
    assert_eq!(p.status, "review");
    assert!(p.approved_at.is_none());
    assert!(f.files.iter().all(|p| p.exists()));
    for paths in [
        vec![f.files[0].to_string_lossy().into_owned(); 2],
        vec![f
            .home
            .join("Library/Caches/not-indexed")
            .to_string_lossy()
            .into_owned()],
    ] {
        assert!(cleanup::preview(
            &f.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths
            },
            &f.policy()
        )
        .is_err());
    }
}
#[test]
fn candidates_filter_links_exclusions_projects_and_unsafe_roots() {
    let f = Fixture::new();
    let mut policy = f.policy();
    policy.exclusions.push(f.files[0].clone());
    let page = cleanup::candidates(&f.c, "scan", 0, &policy).unwrap();
    assert_eq!(page.files.len(), 1);
    assert_eq!(page.unavailable_count, 1);
    fs::write(
        f.files[0].parent().unwrap().join(".git"),
        "gitdir: /elsewhere",
    )
    .unwrap();
    assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
        .unwrap()
        .files
        .is_empty());
    fs::remove_file(f.files[0].parent().unwrap().join(".git")).unwrap();
    fs::hard_link(&f.files[0], f.home.join("intentional-copy")).unwrap();
    assert_eq!(
        cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .len(),
        1
    );
    fs::remove_file(&f.files[1]).unwrap();
    std::os::unix::fs::symlink(&f.files[0], &f.files[1]).unwrap();
    assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
        .unwrap()
        .files
        .is_empty());
}
#[test]
fn changed_target_is_skipped_and_other_items_continue() {
    let f = Fixture::new();
    let p = f.plan();
    fs::write(&f.files[0], "changed since review").unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |p| f.trash(p),
        |_| true,
    )
    .unwrap();
    assert_eq!(r.status, "partial");
    assert_eq!(r.items[0].status, "skipped");
    assert_eq!(r.items[1].status, "movedToTrash");
    assert!(f.files[0].exists());
    assert!(!f.files[1].exists());
    assert_eq!(
        cleanup::get(&f.c, &p.id).unwrap().items[0].status,
        "skipped"
    );
}
#[test]
fn swapped_parent_and_new_exclusions_invalidate_targets() {
    let f = Fixture::new();
    let p = f.plan();
    let parent = f.files[0].parent().unwrap();
    let old = f.home.join("old");
    fs::rename(parent, &old).unwrap();
    std::os::unix::fs::symlink(&old, parent).unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("must not move through a replaced parent"),
        |_| true,
    )
    .unwrap();
    assert!(r.items.iter().all(|i| i.status == "skipped"));
    fs::remove_file(parent).unwrap();
    fs::rename(&old, parent).unwrap();
    let fresh = f.plan();
    let mut policy = f.policy();
    policy.exclusions.push(parent.to_owned());
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&fresh),
        &policy,
        &AtomicBool::new(false),
        |_| panic!("new exclusions must be honored"),
        |_| true,
    )
    .unwrap();
    assert!(r.items.iter().all(|i| i.status == "skipped"));
}
#[test]
fn approvals_expire_require_acknowledgements_and_cannot_be_replayed() {
    let f = Fixture::new();
    let p = f.plan();
    let mut approval = Fixture::approval(&p);
    approval.applications_closed = false;
    assert!(cleanup::execute(
        &f.c,
        approval,
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!(),
        |_| true
    )
    .is_err());
    let mut approval = Fixture::approval(&p);
    approval.reviewed_at = "different review".into();
    assert!(cleanup::execute(
        &f.c,
        approval,
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!(),
        |_| true
    )
    .is_err());
    cleanup::decline(&f.c, &p.id).unwrap();
    assert!(cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!(),
        |_| true
    )
    .is_err());
    let p = f.plan();
    let json: String =
        f.c.query_row("SELECT json FROM cleanup_plans WHERE id=?1", [&p.id], |r| {
            r.get(0)
        })
        .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["plan"]["expiresAt"] = "2000-01-01T00:00:00Z".into();
    f.c.execute(
        "UPDATE cleanup_plans SET json=?1 WHERE id=?2",
        [v.to_string(), p.id.clone()],
    )
    .unwrap();
    assert!(cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!(),
        |_| true
    )
    .is_err());
    assert!(f.files.iter().all(|p| p.exists()));
    let p = f.plan();
    cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |p| f.trash(p),
        |_| true,
    )
    .unwrap();
    assert!(cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!(),
        |_| true
    )
    .is_err());
}
#[test]
fn native_failure_and_unverified_moves_are_retained_without_false_success() {
    let f = Fixture::new();
    let p = f.plan();
    let mut count = 0;
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| {
            count += 1;
            if count == 1 {
                anyhow::bail!("Permission denied")
            }
            Ok(f.home.join("missing-result"))
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(r.items[0].status, "failed");
    assert_eq!(r.items[1].status, "unverified");
    assert_eq!(r.status, "partial");
    assert!(f.files.iter().all(|p| p.exists()));
    assert_eq!(cleanup::history(&f.c, 0, 10).unwrap().len(), 1);
}
#[test]
fn cancellation_preserves_completed_results_and_skips_remaining_files() {
    let f = Fixture::new();
    let p = f.plan();
    let cancel = AtomicBool::new(false);
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &cancel,
        |p| {
            let moved = f.trash(p)?;
            cancel.store(true, Ordering::SeqCst);
            Ok(moved)
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(r.status, "cancelled");
    assert_eq!(r.items[0].status, "movedToTrash");
    assert_eq!(r.items[1].status, "cancelled");
    assert!(f.files[1].exists());
}
#[test]
fn lost_channel_cancels_before_mutation() {
    let f = Fixture::new();
    let p = f.plan();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("no moves after channel loss"),
        |_| false,
    )
    .unwrap();
    assert_eq!(r.status, "cancelled");
    assert!(f.files.iter().all(|p| p.exists()));
}
#[test]
fn migration_reopen_and_interruption_do_not_retry_uncertain_actions() {
    let f = Fixture::new();
    let p = f.plan();
    let json: String =
        f.c.query_row("SELECT json FROM cleanup_plans WHERE id=?1", [&p.id], |r| {
            r.get(0)
        })
        .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["plan"]["status"] = "running".into();
    v["plan"]["items"][0]["status"] = "moving".into();
    f.c.execute(
        "UPDATE cleanup_plans SET status='running',json=?1 WHERE id=?2",
        [v.to_string(), p.id.clone()],
    )
    .unwrap();
    let reopened = database::open(&f.home.join("app.sqlite3")).unwrap();
    database::recover_interrupted(&reopened).unwrap();
    let r = cleanup::get(&reopened, &p.id).unwrap();
    assert_eq!(r.status, "interrupted");
    assert_eq!(r.items[0].status, "unverified");
    assert_eq!(r.items[1].status, "cancelled");
    assert!(f.files.iter().all(|p| p.exists()));
    assert_eq!(
        reopened
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        4
    );
    database::purge(&reopened).unwrap();
    assert!(cleanup::history(&reopened, 0, 10).unwrap().is_empty());
}
#[test]
fn version_one_database_preserves_settings_and_scans_on_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v1.db");
    let c = database::open(&path).unwrap();
    let mut settings = AppSettings::default();
    settings.appearance = "light".into();
    database::save_settings(&c, &settings).unwrap();
    c.execute_batch("DROP TABLE records; DROP TABLE maintenance; DROP TABLE cleanup_plans; PRAGMA user_version=1;")
        .unwrap();
    drop(c);
    let c = database::open(&path).unwrap();
    assert_eq!(database::settings(&c).unwrap().appearance, "light");
    assert!(cleanup::history(&c, 0, 10).unwrap().is_empty());
}
#[cfg(target_os = "macos")]
#[test]
#[ignore = "Explicit native adapter validation: moves only a newly-created disposable temporary file to Finder Trash and returns it to the fixture"]
fn native_trash_and_fixture_restore() {
    let f = Fixture::new();
    let manifest = f.home.join("package.json");
    fs::write(
        &manifest,
        r#"{"name":"standalone-home-fixture","private":true}"#,
    )
    .unwrap();
    let p = f.plan();
    let result = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        cleanup::native_trash,
        |_| true,
    )
    .unwrap();
    assert_eq!(result.status, "complete");
    for item in &result.items {
        assert_eq!(item.status, "movedToTrash");
        let trashed = Path::new(item.trash_path.as_ref().unwrap());
        assert!(trashed.exists());
        assert!(!Path::new(&item.file.path).exists());
        fs::rename(trashed, &item.file.path).unwrap();
        assert!(fs::read_to_string(&item.file.path)
            .unwrap()
            .starts_with("useful cached content"));
    }
    assert_eq!(cleanup::get(&f.c, &p.id).unwrap().status, "complete");
    assert_eq!(
        fs::read_to_string(manifest).unwrap(),
        r#"{"name":"standalone-home-fixture","private":true}"#
    );
    assert!(result
        .items
        .iter()
        .all(|i| i.file.origin.contains("package.json")));
}

#[test]
fn cannot_mutate_when_audit_record_cannot_be_saved() {
    let f = Fixture::new();
    let p = f.plan();
    f.c.execute_batch("PRAGMA query_only=ON").unwrap();
    assert!(cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("audit persistence must succeed first"),
        |_| true
    )
    .is_err());
    assert!(f.files.iter().all(|p| p.exists()));
}
#[test]
fn directories_and_files_outside_cache_scope_cannot_be_proposed() {
    let mut f = Fixture::new();
    fs::write(f.home.join("package.json"), "{}").unwrap();
    let outside = f.home.join("document.txt");
    fs::write(&outside, "keep this document").unwrap();
    database::insert_files(
        &mut f.c,
        "scan",
        &[FileEntry {
            path: outside.to_string_lossy().into(),
            parent: f.home.to_string_lossy().into(),
            name: "document.txt".into(),
            category: "Caches".into(),
            logical_bytes: 18.,
            allocated_bytes: 4096.,
            is_directory: false,
            modified_at: now(),
            qualification: String::new(),
        }],
    )
    .unwrap();
    for path in [outside, f.files[0].parent().unwrap().to_owned()] {
        assert!(cleanup::preview(
            &f.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths: vec![path.to_string_lossy().into()]
            },
            &f.policy()
        )
        .is_err());
        assert!(path.exists());
    }
}

#[test]
fn project_boundaries_above_caches_are_never_eligible() {
    for (relative, marker) in [
        ("Library", ".git"),
        ("", ".git"),
        ("Library", "Cargo.toml"),
        ("Library", "package.json"),
    ] {
        let f = Fixture::new();
        let marker = f.home.join(relative).join(marker);
        fs::write(&marker, "project marker fixture").unwrap();
        assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .is_empty());
        assert!(cleanup::preview(
            &f.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths: vec![f.files[0].to_string_lossy().into()]
            },
            &f.policy()
        )
        .is_err());
        assert!(f.files.iter().all(|p| p.exists()));
    }
}
#[test]
fn project_ancestor_added_after_preview_invalidates_approval() {
    for relative in ["Library", ""] {
        let f = Fixture::new();
        let plan = f.plan();
        fs::write(
            f.home.join(relative).join(".git"),
            "gitdir: /fixture/worktree-registration",
        )
        .unwrap();
        let result = cleanup::execute(
            &f.c,
            Fixture::approval(&plan),
            &f.policy(),
            &AtomicBool::new(false),
            |_| panic!("Trash must not be called for project-managed descendants"),
            |_| true,
        )
        .unwrap();
        assert_eq!(result.status, "partial");
        assert!(result.items.iter().all(|i| i.status == "skipped"));
        assert!(f.files.iter().all(|p| p.exists()));
    }
}

#[test]
fn standalone_home_package_does_not_own_the_dedicated_cache_namespace() {
    let f = Fixture::new();
    let manifest = f.home.join("package.json");
    let content = r#"{"name":"home-app","private":true,"scripts":{"build":"do not execute this"}}"#;
    fs::write(&manifest, content).unwrap();
    let page = cleanup::candidates(&f.c, "scan", 0, &f.policy()).unwrap();
    assert_eq!(page.files.len(), 2);
    assert_eq!(page.unavailable_count, 0);
    let p = f.plan();
    assert!(p
        .items
        .iter()
        .all(|i| i.file.origin.contains("no workspace declaration")));
    assert!(f.files.iter().all(|p| p.exists()));
    assert_eq!(fs::read_to_string(manifest).unwrap(), content);
}

#[test]
fn cache_local_projects_and_home_repositories_still_block_with_home_manifest() {
    for (relative, markers) in [
        ("", vec![".git", ".hg", ".svn", ".jj"]),
        ("Library", vec![".git", "package.json", "Cargo.toml"]),
        ("Library/Caches", vec![".git", "package.json", "Cargo.toml"]),
        (
            "Library/Caches/Example",
            vec![
                ".git",
                ".hg",
                ".svn",
                ".jj",
                "package.json",
                "Cargo.toml",
                "pubspec.yaml",
                "pyproject.toml",
            ],
        ),
    ] {
        for marker in markers {
            let f = Fixture::new();
            fs::write(
                f.home.join("package.json"),
                r#"{"name":"standalone-home-fixture","private":true}"#,
            )
            .unwrap();
            fs::write(f.home.join(relative).join(marker), "project marker fixture").unwrap();
            assert!(
                cleanup::candidates(&f.c, "scan", 0, &f.policy())
                    .unwrap()
                    .files
                    .is_empty(),
                "{relative}/{marker} was ignored"
            );
            assert!(cleanup::preview(
                &f.c,
                CleanupRequest {
                    session_id: "scan".into(),
                    paths: vec![f.files[0].to_string_lossy().into()]
                },
                &f.policy()
            )
            .is_err());
            assert!(f.files.iter().all(|p| p.exists()));
        }
    }
}

#[test]
fn ambiguous_home_manifests_are_not_treated_as_standalone_files() {
    for kind in ["directory", "symlink", "dangling symlink"] {
        let f = Fixture::new();
        let marker = f.home.join("package.json");
        match kind {
            "directory" => fs::create_dir(&marker).unwrap(),
            "symlink" => std::os::unix::fs::symlink(&f.files[0], &marker).unwrap(),
            _ => std::os::unix::fs::symlink(f.home.join("missing"), &marker).unwrap(),
        }
        assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .is_empty());
        assert!(cleanup::preview(
            &f.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths: vec![f.files[0].to_string_lossy().into()]
            },
            &f.policy()
        )
        .is_err());
    }
}

#[test]
fn newly_detected_project_invalidates_a_home_manifest_review() {
    for (relative, marker) in [
        ("", ".git"),
        ("", ".hg"),
        ("", ".svn"),
        ("", ".jj"),
        ("Library", "package.json"),
        ("Library/Caches", "Cargo.toml"),
        ("Library/Caches/Example", "package.json"),
    ] {
        let f = Fixture::new();
        fs::write(
            f.home.join("package.json"),
            r#"{"name":"standalone-home-fixture","private":true}"#,
        )
        .unwrap();
        let p = f.plan();
        fs::write(f.home.join(relative).join(marker), "new project fixture").unwrap();
        let r = cleanup::execute(
            &f.c,
            Fixture::approval(&p),
            &f.policy(),
            &AtomicBool::new(false),
            |_| panic!("new project boundary must prevent Trash"),
            |_| true,
        )
        .unwrap();
        assert_eq!(r.status, "partial");
        assert!(r
            .items
            .iter()
            .all(|i| i.status == "skipped" && i.detail.contains(marker)));
        assert!(f.files.iter().all(|p| p.exists()));
    }
}

#[test]
fn home_manifest_approval_moves_only_selected_cache_files() {
    let f = Fixture::new();
    let marker = f.home.join("package.json");
    fs::write(
        &marker,
        r#"{"name":"standalone-home-fixture","private":true}"#,
    )
    .unwrap();
    let p = cleanup::preview(
        &f.c,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![f.files[0].to_string_lossy().into()],
        },
        &f.policy(),
    )
    .unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |p| f.trash(p),
        |_| true,
    )
    .unwrap();
    assert_eq!(r.status, "complete");
    assert!(!f.files[0].exists());
    assert!(f.files[1].exists());
    assert_eq!(
        fs::read_to_string(marker).unwrap(),
        r#"{"name":"standalone-home-fixture","private":true}"#
    );
    let retained = cleanup::get(&f.c, &p.id).unwrap();
    assert!(retained.items[0].file.origin.contains("package.json"));
}

#[test]
fn one_cache_local_project_does_not_block_unrelated_cache_files() {
    let mut f = Fixture::new();
    fs::write(
        f.home.join("package.json"),
        r#"{"name":"standalone-home-fixture","private":true}"#,
    )
    .unwrap();
    let project = f.home.join("Library/Caches/Example/project");
    fs::create_dir(&project).unwrap();
    fs::write(project.join(".git"), "gitdir: /fixture/worktree").unwrap();
    let protected = project.join("important.cache");
    fs::write(&protected, "keep project data").unwrap();
    let mut rows = vec![];
    storage::walk(
        &[f.home.join("Library/Caches")],
        &[],
        &AtomicBool::new(false),
        |r| {
            rows.push(r);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    database::insert_files(&mut f.c, "scan", &rows).unwrap();
    let page = cleanup::candidates(&f.c, "scan", 0, &f.policy()).unwrap();
    assert_eq!(page.files.len(), 2);
    assert_eq!(page.unavailable_count, 2); // .git reference and its protected sibling
    assert!(cleanup::preview(
        &f.c,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![protected.to_string_lossy().into()]
        },
        &f.policy()
    )
    .is_err());
    let p = f.plan();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |p| f.trash(p),
        |_| true,
    )
    .unwrap();
    assert_eq!(r.status, "complete");
    assert_eq!(fs::read_to_string(protected).unwrap(), "keep project data");
}

#[test]
fn a_home_manifest_that_becomes_ambiguous_invalidates_the_review() {
    let f = Fixture::new();
    let marker = f.home.join("package.json");
    fs::write(
        &marker,
        r#"{"name":"standalone-home-fixture","private":true}"#,
    )
    .unwrap();
    let p = f.plan();
    fs::remove_file(&marker).unwrap();
    std::os::unix::fs::symlink(&f.files[0], &marker).unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("ambiguous home context must prevent Trash"),
        |_| true,
    )
    .unwrap();
    assert!(r.items.iter().all(|i| i.status == "skipped"));
    assert!(f.files.iter().all(|p| p.exists()));
}

#[test]
fn home_workspace_or_unknown_manifest_scope_remains_protected() {
    for content in [
        r#"{"workspaces":["Library/Caches/*"]}"#,
        r#"{"workspaces":["Projects/*"]}"#,
        r#"{"workspaces":{"packages":["Library/Caches/*"]}}"#,
        r#"{"workspaces":null}"#,
        "[]",
        "not JSON",
        "",
    ] {
        let f = Fixture::new();
        fs::write(f.home.join("package.json"), content).unwrap();
        assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .is_empty());
        assert!(cleanup::preview(
            &f.c,
            CleanupRequest {
                session_id: "scan".into(),
                paths: vec![f.files[0].to_string_lossy().into()]
            },
            &f.policy()
        )
        .is_err());
        assert!(f.files.iter().all(|p| p.exists()));
    }
    for marker in [
        "Cargo.toml",
        "pyproject.toml",
        "pubspec.yaml",
        "pnpm-workspace.yaml",
        "lerna.json",
    ] {
        let f = Fixture::new();
        fs::write(f.home.join("package.json"), "{}").unwrap();
        fs::write(f.home.join(marker), "workspace fixture").unwrap();
        assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .is_empty());
    }
}

#[test]
fn workspace_declaration_added_after_preview_prevents_moves() {
    let f = Fixture::new();
    fs::write(f.home.join("package.json"), "{}").unwrap();
    let p = f.plan();
    fs::write(
        f.home.join("package.json"),
        r#"{"workspaces":["Library/Caches/*"]}"#,
    )
    .unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("workspace declaration must prevent Trash"),
        |_| true,
    )
    .unwrap();
    assert!(r
        .items
        .iter()
        .all(|i| i.status == "skipped" && i.detail.contains("workspace")));
    assert!(f.files.iter().all(|p| p.exists()));
}

#[test]
fn oversized_or_hard_linked_home_package_is_ambiguous() {
    for kind in ["oversized", "hard link"] {
        let f = Fixture::new();
        let manifest = f.home.join("package.json");
        fs::write(&manifest, "{}").unwrap();
        if kind == "oversized" {
            fs::write(&manifest, vec![b' '; 256 * 1024 + 1]).unwrap();
        } else {
            fs::hard_link(&manifest, f.home.join("another-manifest")).unwrap();
        }
        assert!(cleanup::candidates(&f.c, "scan", 0, &f.policy())
            .unwrap()
            .files
            .is_empty());
    }
}

#[test]
fn unreadable_home_package_invalidates_an_existing_review() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return; // Root cannot exercise ordinary user permission denial.
    }
    let f = Fixture::new();
    let manifest = f.home.join("package.json");
    fs::write(&manifest, "{}").unwrap();
    let p = f.plan();
    let permissions = fs::metadata(&manifest).unwrap().permissions();
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0)).unwrap();
    let page = cleanup::candidates(&f.c, "scan", 0, &f.policy()).unwrap();
    let r = cleanup::execute(
        &f.c,
        Fixture::approval(&p),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("unknown manifest scope must prevent Trash"),
        |_| true,
    )
    .unwrap();
    fs::set_permissions(&manifest, permissions).unwrap();
    assert!(page.files.is_empty());
    assert!(r.items.iter().all(|i| i.status == "skipped"));
    assert!(f.files.iter().all(|p| p.exists()));
}
#[test]
fn legacy_cache_plans_without_project_fields_still_open_and_decline_as_files() {
    let f = Fixture::new();
    let plan = f.plan();
    let raw: String =
        f.c.query_row(
            "SELECT json FROM cleanup_plans WHERE id=?1",
            [&plan.id],
            |r| r.get(0),
        )
        .unwrap();
    let mut record: serde_json::Value = serde_json::from_str(&raw).unwrap();
    record["plan"]
        .as_object_mut()
        .unwrap()
        .remove("projectCleanup");
    for identity in record["identities"].as_array_mut().unwrap() {
        identity.as_object_mut().unwrap().remove("project");
    }
    f.c.execute(
        "UPDATE cleanup_plans SET json=?1 WHERE id=?2",
        rusqlite::params![record.to_string(), plan.id],
    )
    .unwrap();
    assert!(!cleanup::get(&f.c, &plan.id).unwrap().project_cleanup);
    let declined = cleanup::decline(&f.c, &plan.id).unwrap();
    assert_eq!(declined.status, "declined");
    assert!(declined.items.iter().all(
        |item| item.status == "declined" && item.detail.contains("Not moved; review declined")
    ));
}
