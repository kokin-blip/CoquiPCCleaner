use coqui_engine::{
    applications, database,
    removal::{self, Policy, RemovalApproval, RemovalRequest},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
struct Fixture {
    _dir: tempfile::TempDir,
    home: PathBuf,
    c: rusqlite::Connection,
    policy: Policy,
    cancel: AtomicBool,
}
impl Fixture {
    fn new() -> Self {
        let d = tempfile::tempdir().unwrap();
        let home = d.path().canonicalize().unwrap();
        for p in ["Downloads", "Documents", ".Trash", "Applications"] {
            fs::create_dir(home.join(p)).unwrap();
        }
        let c = database::open(&home.join("data.db")).unwrap();
        let policy = Policy {
            home: home.clone(),
            exclusions: vec![],
            self_app: None,
            administrator_uid: None,
        };
        Self {
            _dir: d,
            home,
            c,
            policy,
            cancel: AtomicBool::new(false),
        }
    }
    fn file(&self, n: &str) -> PathBuf {
        let p = self.home.join("Downloads").join(n);
        fs::write(&p, b"user data").unwrap();
        p
    }
    fn preview(&self, paths: &[PathBuf]) -> removal::RemovalPlan {
        removal::preview(
            &self.c,
            RemovalRequest {
                paths: paths.iter().map(|p| p.to_str().unwrap().into()).collect(),
                application: None,
            },
            &self.policy,
            &self.cancel,
        )
        .unwrap()
    }
    fn approval(&self, p: &removal::RemovalPlan) -> RemovalApproval {
        RemovalApproval {
            id: p.id.clone(),
            paths: p.items.iter().map(|i| i.path.clone()).collect(),
            acknowledged: true,
        }
    }
    fn move_to_trash(&self, p: &Path) -> anyhow::Result<PathBuf> {
        let dest = self.home.join(".Trash").join(p.file_name().unwrap());
        fs::rename(p, &dest)?;
        Ok(dest)
    }
}
#[test]
fn explicit_review_stale_approval_replay_and_folder_verification() {
    let f = Fixture::new();
    let p = f.file("one");
    let plan = f.preview(std::slice::from_ref(&p));
    assert!(p.exists());
    fs::write(&p, b"changed").unwrap();
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!("must not move"),
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "blocked");
    assert!(removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true
    )
    .is_err());
    let folder = f.home.join("Documents/folder");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("child"), "kept").unwrap();
    let p = f.preview(std::slice::from_ref(&folder));
    assert_eq!(p.items[0].entry_count, 2);
    let done = removal::execute(
        &f.c,
        f.approval(&p),
        &f.policy,
        &f.cancel,
        |p| f.move_to_trash(p),
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "movedToTrash");
    assert_eq!(
        fs::read(f.home.join(".Trash/folder/child")).unwrap(),
        b"kept"
    );
}
#[test]
fn changed_descendants_exclusions_project_and_cloud_paths_block() {
    let mut f = Fixture::new();
    let dir = f.home.join("Downloads/folder");
    fs::create_dir(&dir).unwrap();
    let plan = f.preview(std::slice::from_ref(&dir));
    fs::write(dir.join("late"), "unique").unwrap();
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "blocked");
    fs::write(dir.join("package.json"), "{}").unwrap();
    assert!(removal::preview(
        &f.c,
        RemovalRequest {
            paths: vec![dir.to_str().unwrap().into()],
            application: None
        },
        &f.policy,
        &f.cancel
    )
    .is_err());
    let p = f.file("cloud.icloud");
    assert!(removal::preview(
        &f.c,
        RemovalRequest {
            paths: vec![p.to_str().unwrap().into()],
            application: None
        },
        &f.policy,
        &f.cancel
    )
    .is_err());
    let p = f.file("excluded");
    let plan = f.preview(std::slice::from_ref(&p));
    f.policy.exclusions.push(p.clone());
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "blocked");
}
#[test]
fn failed_verification_channel_loss_and_interruption_are_never_retried() {
    let f = Fixture::new();
    let p = f.file("one");
    let plan = f.preview(std::slice::from_ref(&p));
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |p| {
            let dest = f.move_to_trash(p)?;
            fs::write(f.file("other"), "different")?;
            fs::remove_file(&dest)?;
            fs::write(&dest, "replacement")?;
            Ok(dest)
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(done.items[0].status, "verificationFailed");
    let p = f.file("cancel");
    let plan = f.preview(std::slice::from_ref(&p));
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| false,
    )
    .unwrap();
    assert_eq!(done.status, "cancelled");
    assert!(p.exists());
    f.cancel.store(false, Ordering::Relaxed);
    let plan = f.preview(std::slice::from_ref(&p));
    f.c.execute("UPDATE removal_plans SET status='running',json=json_set(json,'$.plan.status','running','$.plan.items[0].status','intentRecorded') WHERE id=?1",[&plan.id]).unwrap();
    database::recover_interrupted(&f.c).unwrap();
    let r = removal::history(&f.c, 0)
        .unwrap()
        .into_iter()
        .find(|r| r.id == plan.id)
        .unwrap();
    assert_eq!(r.status, "interrupted");
    assert_eq!(r.items[0].status, "interrupted");
}
#[test]
fn restore_requires_retained_identity_original_parent_and_no_overwrite() {
    let f = Fixture::new();
    let p = f.file("one");
    let plan = f.preview(std::slice::from_ref(&p));
    let done = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |p| f.move_to_trash(p),
        |_| true,
    )
    .unwrap();
    let r = removal::preview_restore(
        &f.c,
        &done.id,
        &[p.to_str().unwrap().into()],
        &f.policy,
        &f.cancel,
    )
    .unwrap();
    fs::write(&p, "new work").unwrap();
    let blocked = removal::execute(
        &f.c,
        f.approval(&r),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true,
    )
    .unwrap();
    assert_eq!(blocked.items[0].status, "blocked");
    assert_eq!(fs::read(&p).unwrap(), b"new work");
    fs::remove_file(&p).unwrap();
    let r = removal::preview_restore(
        &f.c,
        &done.id,
        &[p.to_str().unwrap().into()],
        &f.policy,
        &f.cancel,
    )
    .unwrap();
    let restored = removal::execute(
        &f.c,
        f.approval(&r),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true,
    )
    .unwrap();
    assert_eq!(restored.items[0].status, "restored");
    assert_eq!(fs::read(&p).unwrap(), b"user data");
}
#[test]
fn application_exact_associations_shared_data_and_bundle_links() {
    let f = Fixture::new();
    let app = f.home.join("Applications/Fixture.app");
    fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
    fs::write(app.join("Contents/Info.plist"),r#"<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.coqui.fixture</string><key>CFBundleName</key><string>Fixture</string></dict></plist>"#).unwrap();
    fs::write(app.join("Contents/MacOS/fixture"), "fixture").unwrap();
    std::os::unix::fs::symlink("MacOS/fixture", app.join("Contents/link")).unwrap();
    for path in [
        "Library/Caches/org.coqui.fixture",
        "Library/Containers/org.coqui.fixture",
        "Library/Group Containers/shared",
    ] {
        fs::create_dir_all(f.home.join(path)).unwrap();
    }
    let footprint = applications::inspect(&app, &f.policy).unwrap();
    assert!(footprint
        .related
        .iter()
        .any(|r| r.eligible && r.path.contains("Caches")));
    assert!(footprint
        .related
        .iter()
        .any(|r| !r.eligible && r.path.contains("Containers")));
    assert!(!footprint
        .related
        .iter()
        .any(|r| r.path.contains("Group Containers")));
    let review = removal::preview(
        &f.c,
        RemovalRequest {
            paths: vec![app.to_str().unwrap().into()],
            application: Some(app.to_str().unwrap().into()),
        },
        &f.policy,
        &f.cancel,
    )
    .unwrap();
    let moved = removal::execute(
        &f.c,
        f.approval(&review),
        &f.policy,
        &f.cancel,
        |p| f.move_to_trash(p),
        |_| true,
    )
    .unwrap();
    assert_eq!(moved.items[0].status, "movedToTrash");
    assert!(
        fs::symlink_metadata(f.home.join(".Trash/Fixture.app/Contents/link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[cfg(target_os = "macos")]
#[derive(Default)]
struct NativeFixtureRecovery(Vec<(PathBuf, PathBuf, u64, u64)>);
#[cfg(target_os = "macos")]
impl Drop for NativeFixtureRecovery {
    fn drop(&mut self) {
        use std::os::unix::fs::MetadataExt;
        for (source, trash, dev, ino) in &self.0 {
            if fs::symlink_metadata(source).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                && fs::symlink_metadata(trash).is_ok_and(|m| m.dev() == *dev && m.ino() == *ino)
            {
                if let (Ok(from), Ok(to)) = (
                    std::ffi::CString::new(trash.to_string_lossy().as_bytes()),
                    std::ffi::CString::new(source.to_string_lossy().as_bytes()),
                ) {
                    unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
                }
            }
        }
    }
}
#[test]
#[ignore = "Explicit native Trash/restore validation; creates only uniquely named disposable Downloads fixtures"]
fn native_file_folder_and_application_trash_and_verified_restore() {
    let home = coqui_engine::platform::home()
        .unwrap()
        .canonicalize()
        .unwrap();
    let dir = tempfile::Builder::new()
        .prefix("Coqui-disposable-removal-")
        .tempdir_in(home.join("Downloads"))
        .unwrap();
    let root = dir.path().canonicalize().unwrap();
    let c = database::open(&root.join("journal.sqlite3")).unwrap();
    let policy = Policy {
        home,
        exclusions: vec![],
        self_app: None,
        administrator_uid: None,
    };
    let cancel = AtomicBool::new(false);
    let file = root.join("Disposable file.txt");
    fs::write(&file, "disposable native fixture").unwrap();
    let folder = root.join("Disposable folder");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("child"), "disposable child").unwrap();
    let app = root.join("Coqui disposable fixture.app");
    fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
    fs::write(app.join("Contents/Info.plist"),r#"<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.coqui.disposable.nativefixture</string><key>CFBundleName</key><string>Disposable fixture</string></dict></plist>"#).unwrap();
    fs::write(
        app.join("Contents/MacOS/fixture"),
        "non executable disposable fixture",
    )
    .unwrap();
    std::os::unix::fs::symlink("MacOS/fixture", app.join("Contents/opaque link")).unwrap();
    let mut recovery = NativeFixtureRecovery::default();
    for p in [&file, &folder, &app] {
        use std::os::unix::fs::MetadataExt;
        let original = fs::symlink_metadata(p).unwrap();
        let request = RemovalRequest {
            paths: vec![p.to_str().unwrap().into()],
            application: if p == &app {
                Some(app.to_str().unwrap().into())
            } else {
                None
            },
        };
        let plan = removal::preview(&c, request, &policy, &cancel).unwrap();
        let approval = RemovalApproval {
            id: plan.id.clone(),
            paths: plan.items.iter().map(|i| i.path.clone()).collect(),
            acknowledged: true,
        };
        let moved = removal::native_execute(&c, approval, &policy, &cancel, |_| true).unwrap();
        if let Some(trash) = &moved.items[0].trash_path {
            recovery.0.push((
                p.clone(),
                PathBuf::from(trash),
                original.dev(),
                original.ino(),
            ));
        }
        assert_eq!(
            moved.items[0].status, "movedToTrash",
            "{}",
            moved.items[0].detail
        );
        let mut restored = false;
        for attempt in 0..3 {
            let restore = removal::preview_restore(
                &c,
                &moved.id,
                &[p.to_str().unwrap().into()],
                &policy,
                &cancel,
            )
            .unwrap();
            let approval = RemovalApproval {
                id: restore.id.clone(),
                paths: restore.items.iter().map(|i| i.path.clone()).collect(),
                acknowledged: true,
            };
            let result = removal::native_execute(&c, approval, &policy, &cancel, |_| true).unwrap();
            if result.items[0].status == "restored" {
                restored = true;
                break;
            }
            assert!(
                result.items[0].status == "blocked"
                    && result.items[0]
                        .detail
                        .contains("Trash item or parent changed"),
                "{}: {}",
                p.display(),
                result.items[0].detail
            );
            eprintln!("{}: review {} was invalidated by changed Trash metadata; a new fixture-only review and approval is required",p.file_name().unwrap().to_string_lossy(),attempt+1);
            // This creates a different approval ID. Production never retries an old approval.
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        assert!(
            restored,
            "{} never stabilized for a fresh fixture restore review",
            p.display()
        );
        assert!(p.exists());
    }
    assert_eq!(fs::read(file).unwrap(), b"disposable native fixture");
    assert_eq!(fs::read(folder.join("child")).unwrap(), b"disposable child");
    assert!(fs::symlink_metadata(app.join("Contents/opaque link"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
#[cfg(target_os = "macos")]
fn data_volume_alias_and_visible_path_have_one_review_identity() {
    let f = Fixture::new();
    let file = f.file("alias.txt");
    let alias = PathBuf::from("/System/Volumes/Data").join(file.strip_prefix("/").unwrap());
    if alias.exists() {
        let plan = removal::preview(
            &f.c,
            RemovalRequest {
                paths: vec![alias.to_str().unwrap().into()],
                application: None,
            },
            &f.policy,
            &f.cancel,
        )
        .unwrap();
        assert_eq!(plan.items[0].path, file.to_str().unwrap());
        assert!(removal::preview(
            &f.c,
            RemovalRequest {
                paths: vec![
                    file.to_str().unwrap().into(),
                    alias.to_str().unwrap().into()
                ],
                application: None
            },
            &f.policy,
            &f.cancel
        )
        .is_err());
    }
}
#[test]
fn application_identity_change_blocks_related_data_and_notices_stay_opt_in() {
    let f = Fixture::new();
    let app = f.home.join("Applications/Fixture.app");
    fs::create_dir_all(app.join("Contents")).unwrap();
    let plist = app.join("Contents/Info.plist");
    fs::write(&plist,r#"<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.coqui.relatedfixture</string></dict></plist>"#).unwrap();
    let cache = f.home.join("Library/Caches/org.coqui.relatedfixture");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("data"), "user cache").unwrap();
    let plan = removal::preview(
        &f.c,
        RemovalRequest {
            paths: vec![cache.to_str().unwrap().into()],
            application: Some(app.to_str().unwrap().into()),
        },
        &f.policy,
        &f.cancel,
    )
    .unwrap();
    fs::write(app.join("Contents/new"), "changed bundle").unwrap();
    let blocked = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| panic!(),
        |_| true,
    )
    .unwrap();
    assert_eq!(blocked.items[0].status, "blocked");
    let footprint = applications::inspect(&app, &f.policy).unwrap();
    f.c.execute(
        "INSERT INTO application_observations VALUES(?1,?2,NULL)",
        rusqlite::params![footprint.path, serde_json::to_string(&footprint).unwrap()],
    )
    .unwrap();
    let moved = f.home.join("Applications/Moved.app");
    fs::rename(app, moved).unwrap();
    assert_eq!(applications::notices(&f.c).unwrap(), 0);
    let prefs = applications::ApplicationPreferences {
        protected_paths: vec![],
        disappearance_notices: true,
    };
    applications::save_preferences(&f.c, &prefs).unwrap();
    assert_eq!(applications::notices(&f.c).unwrap(), 1);
    let missing = applications::leftovers(&f.c, 0).unwrap();
    assert_eq!(missing.len(), 1);
    assert!(missing[0].related.iter().all(|r| !r.eligible));
    database::purge(&f.c).unwrap();
    assert_eq!(applications::notices(&f.c).unwrap(), 0);
    assert!(
        applications::preferences(&f.c)
            .unwrap()
            .disappearance_notices
    );
}

#[test]
fn indexed_application_search_protected_pagination_and_schema_five_upgrade() {
    let f = Fixture::new();
    let settings = database::settings(&f.c).unwrap();
    for i in 0..60 {
        let app = applications::ApplicationFootprint {
            path: format!("/fixture/App{i}.app"),
            name: format!("App{i:02}"),
            bundle_id: format!("org.coqui.fixture{i}"),
            version: "1".into(),
            observed_at: coqui_engine::model::now(),
            protection: if i % 2 == 0 {
                vec!["Protected fixture".into()]
            } else {
                vec![]
            },
            related: vec![],
            qualification: "Fixture".into(),
        };
        f.c.execute(
            "INSERT INTO application_observations VALUES(?1,?2,NULL)",
            rusqlite::params![app.path, serde_json::to_string(&app).unwrap()],
        )
        .unwrap();
    }
    let first = applications::indexed_page(&f.c, "", "name", 0, false).unwrap();
    assert_eq!(first.total, 60);
    assert_eq!(first.applications.len(), 50);
    assert_eq!(
        applications::indexed_page(&f.c, "", "name", 50, false)
            .unwrap()
            .applications
            .len(),
        10
    );
    assert_eq!(
        applications::indexed_page(&f.c, "fixture59", "name", 0, false)
            .unwrap()
            .total,
        1
    );
    assert_eq!(
        applications::indexed_page(&f.c, "", "name", 0, true)
            .unwrap()
            .total,
        30
    );
    f.c.execute_batch("DROP TABLE scan_plans;DROP TABLE scan_gaps;DROP TABLE removal_plans;DROP TABLE application_observations;DROP TABLE application_preferences;PRAGMA user_version=5;").unwrap();
    let upgraded = database::open(&f.home.join("data.db")).unwrap();
    assert_eq!(
        database::settings(&upgraded).unwrap().retention_days,
        settings.retention_days
    );
    assert_eq!(
        upgraded
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        6
    );
    assert!(applications::preferences(&upgraded)
        .unwrap()
        .protected_paths
        .is_empty());
}

#[test]
fn multiple_bundle_claimants_never_authorize_shared_data() {
    let f = Fixture::new();
    let mut apps = vec![];
    for name in ["First.app", "Second.app"] {
        let app = f.home.join("Applications").join(name);
        fs::create_dir_all(app.join("Contents")).unwrap();
        fs::write(app.join("Contents/Info.plist"),r#"<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>org.coqui.sharedfixture</string></dict></plist>"#).unwrap();
        apps.push(app);
    }
    let cache = f.home.join("Library/Caches/org.coqui.sharedfixture");
    fs::create_dir_all(&cache).unwrap();
    let footprint = applications::qualify_related(
        applications::inspect(&apps[0], &f.policy).unwrap(),
        &f.policy,
    );
    assert!(footprint.related.iter().all(|r| !r.eligible));
    assert!(removal::preview(
        &f.c,
        RemovalRequest {
            paths: vec![cache.to_str().unwrap().into()],
            application: Some(apps[0].to_str().unwrap().into())
        },
        &f.policy,
        &f.cancel
    )
    .is_err());
    assert!(cache.exists());
}

#[test]
fn reviewed_folder_allocation_counts_hard_linked_objects_once() {
    let f = Fixture::new();
    let dir = f.home.join("Downloads/linked objects");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a"), "one object").unwrap();
    fs::hard_link(dir.join("a"), dir.join("b")).unwrap();
    let p = f.preview(&[dir]);
    assert_eq!(p.items[0].entry_count, 3);
    assert_eq!(p.items[0].logical_bytes, 10.);
}

#[test]
#[cfg(target_os = "macos")]
fn administrator_reviews_use_fd_seals_and_remain_bound_to_exclusions_and_authority() {
    let mut f = Fixture::new();
    f.policy.administrator_uid = Some(unsafe { libc::geteuid() });
    let dir = f.home.join("Downloads/Reviewed folder");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a"), b"fixture").unwrap();
    let plan = f.preview(&[dir.clone()]);
    assert!(plan.administrator);
    f.policy.exclusions.push(dir.join("a"));
    let mut moved = false;
    let result = removal::execute(
        &f.c,
        f.approval(&plan),
        &f.policy,
        &f.cancel,
        |_| {
            moved = true;
            anyhow::bail!("must not move")
        },
        |_| true,
    )
    .unwrap();
    assert!(!moved);
    assert_eq!(result.items[0].status, "blocked");
    f.policy.exclusions.clear();
    let second = f.preview(&[dir.clone()]);
    f.policy.administrator_uid = None;
    assert!(removal::execute(
        &f.c,
        f.approval(&second),
        &f.policy,
        &f.cancel,
        |_| anyhow::bail!("must not move"),
        |_| true
    )
    .is_err());
    assert!(dir.exists());
}
