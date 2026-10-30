use coqui_engine::{
    cache_bulk::{self, *},
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
    home: PathBuf,
    c: Connection,
    files: Vec<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().canonicalize().unwrap();
        let root = home.join("Library/Caches/example.namespace");
        fs::create_dir_all(&root).unwrap();
        let files = vec![root.join("a"), root.join("b")];
        for path in &files {
            fs::write(path, "disposable fixture data").unwrap();
        }
        let mut c = database::open(&home.join("app.sqlite3")).unwrap();
        let s:DiagnosticSession=serde_json::from_value(serde_json::json!({"id":"scan","startedAt":now(),"finishedAt":now(),"status":"partial","stage":"Fixture","scope":home,"fileCount":2,"logicalBytes":40,"allocatedBytes":8192,"skippedCount":0,"snapshot":null,"findings":[],"permissions":[],"categories":[],"candidates":[],"errors":[],"coverage":[],"volumePlan":null})).unwrap();
        database::save_session(&c, &s).unwrap();
        let mut rows = vec![];
        storage::walk(
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
        Self {
            _dir: dir,
            home,
            c,
            files,
        }
    }
    fn policy(&self) -> Policy {
        Policy {
            home: self.home.clone(),
            exclusions: vec![],
        }
    }
    fn query(&self) -> CacheQuery {
        CacheQuery {
            session_id: "scan".into(),
            ..Default::default()
        }
    }
    fn select(&self) -> CacheBulkSummary {
        cache_bulk::select(
            &self.c,
            CacheSelectionRequest {
                query: self.query(),
                revision: None,
                paths: None,
                selected: true,
                clear: false,
            },
            &self.policy(),
            &AtomicBool::new(false),
            |_| true,
        )
        .unwrap()
    }
    fn prepare(&self) -> CacheBulkSummary {
        let s = self.select();
        cache_bulk::prepare(
            &self.c,
            &s.id,
            s.revision,
            &self.policy(),
            &AtomicBool::new(false),
            |_| true,
        )
        .unwrap()
    }
    fn approval(s: &CacheBulkSummary) -> CacheBulkApproval {
        CacheBulkApproval {
            id: s.id.clone(),
            revision: s.revision,
            reviewed_at: s.reviewed_at.clone().unwrap(),
            applications_closed: true,
            consequences_accepted: true,
        }
    }
    fn page(&self, id: &str) -> CacheBulkPage {
        cache_bulk::page(
            &self.c,
            CacheBulkPageQuery {
                id: id.into(),
                cursor: None,
                status: None,
            },
        )
        .unwrap()
    }
    fn trash(&self, path: &Path) -> anyhow::Result<PathBuf> {
        let root = self.home.join("disposable-trash");
        fs::create_dir_all(&root)?;
        let dest = root.join(path.file_name().unwrap());
        fs::rename(path, &dest)?;
        Ok(dest)
    }
}
#[test]
fn frozen_paths_single_use_approval_and_verified_moves() {
    let f = Fixture::new();
    let s = f.prepare();
    assert_eq!(s.eligible_count, 2);
    assert_eq!(s.status, "review");
    let extra = f.files[0].parent().unwrap().join("new-after-review");
    fs::write(&extra, "keep").unwrap();
    let result = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |p| f.trash(p),
        |e| {
            assert!(serde_json::to_string(&e).unwrap().len() < 3000);
            true
        },
    )
    .unwrap();
    assert_eq!(result.moved_count, 2);
    assert_eq!(result.status, "complete");
    assert!(extra.exists());
    assert!(f
        .page(&s.id)
        .items
        .iter()
        .all(|i| i.status == "movedToTrash"));
    assert!(cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("replay"),
        |_| true
    )
    .is_err());
}
#[test]
fn refusals_continue_and_changed_files_are_skipped() {
    let f = Fixture::new();
    fs::remove_file(&f.files[0]).unwrap();
    let s = f.prepare();
    assert_eq!(s.refused_count, 1);
    assert_eq!(s.eligible_count, 1);
    fs::write(&f.files[1], "changed since review").unwrap();
    let r = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("changed file"),
        |_| true,
    )
    .unwrap();
    assert_eq!(r.skipped_count, 1);
    assert_eq!(r.status, "partial");
    assert!(f
        .page(&s.id)
        .items
        .iter()
        .any(|i| i.status == "refused" && !i.detail.is_empty()));
}
#[test]
fn paths_membership_revisions_policy_acknowledgements_and_expiry() {
    let f = Fixture::new();
    let mut q = f.query();
    q.group = Some("wrong.group".into());
    assert!(cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q,
            revision: None,
            paths: Some(vec![f.files[0].to_string_lossy().into()]),
            selected: true,
            clear: false
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true
    )
    .is_err());
    let s = f.select();
    let mut q = f.query();
    q.batch_id = Some(s.id.clone());
    assert!(cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q,
            revision: Some(0),
            paths: None,
            selected: false,
            clear: false
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true
    )
    .is_err());
    let p = Policy {
        home: f.home.clone(),
        exclusions: vec![f.files[0].clone()],
    };
    assert!(
        cache_bulk::prepare(&f.c, &s.id, s.revision, &p, &AtomicBool::new(false), |_| {
            true
        })
        .is_err()
    );
    let s = cache_bulk::prepare(
        &f.c,
        &s.id,
        s.revision,
        &f.policy(),
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    let mut a = Fixture::approval(&s);
    a.consequences_accepted = false;
    assert!(cache_bulk::execute(
        &f.c,
        a,
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("unapproved"),
        |_| true
    )
    .is_err());
    let mut a = Fixture::approval(&s);
    a.revision += 1;
    assert!(cache_bulk::execute(
        &f.c,
        a,
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("wrong revision"),
        |_| true
    )
    .is_err());
    f.c.execute(
        "UPDATE cache_batches SET expires_at='2000-01-01T00:00:00Z' WHERE id=?1",
        [&s.id],
    )
    .unwrap();
    assert!(cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("expired"),
        |_| true
    )
    .is_err());
}
#[test]
fn cancellation_disconnection_and_restart_never_resume() {
    let f = Fixture::new();
    let draft = f.select();
    let s = cache_bulk::prepare(
        &f.c,
        &draft.id,
        draft.revision,
        &f.policy(),
        &AtomicBool::new(true),
        |_| true,
    )
    .unwrap();
    assert_eq!(s.status, "cancelled");
    assert!(s.reviewed_at.is_none());
    let s = f.prepare();
    let r = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("disconnected"),
        |_| false,
    )
    .unwrap();
    assert_eq!(r.status, "cancelled");
    assert_eq!(r.unattempted_count, 2);
    let s = f.prepare();
    f.c.execute(
        "UPDATE cache_batches SET status='running' WHERE id=?1",
        [&s.id],
    )
    .unwrap();
    f.c.execute(
        "UPDATE cache_batch_items SET status='moving' WHERE batch_id=?1 AND path=?2",
        rusqlite::params![s.id, f.files[0].to_string_lossy()],
    )
    .unwrap();
    cache_bulk::recover(&f.c).unwrap();
    let r = cache_bulk::summary(&f.c, &s.id).unwrap();
    assert_eq!(r.status, "interrupted");
    assert_eq!(r.unverified_count, 1);
    assert_eq!(r.unattempted_count, 1);
    assert!(f.files.iter().all(|p| p.exists()));
}
#[test]
fn cancellation_during_move_preserves_verified_outcomes() {
    let f = Fixture::new();
    let s = f.prepare();
    let cancel = AtomicBool::new(false);
    let r = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &cancel,
        |p| {
            let r = f.trash(p);
            cancel.store(true, Ordering::SeqCst);
            r
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(r.status, "cancelled");
    assert_eq!(r.moved_count, 1);
    assert_eq!(r.unattempted_count, 1);
}
#[test]
fn project_markers_links_and_manual_categories_stay_protected() {
    let f = Fixture::new();
    std::os::unix::fs::symlink(&f.files[0], f.files[0].parent().unwrap().join("link")).unwrap();
    fs::write(f.files[0].parent().unwrap().join("package.json"), "{}").unwrap();
    let s = f.prepare();
    assert_eq!(s.eligible_count, 0);
    assert_eq!(s.refused_count, 2);
    assert_eq!(s.status, "empty");
    let mut q = f.query();
    q.kind = Some(coqui_engine::cleanup_categories::CleanupCategoryKind::UserLogs);
    assert!(cache_bulk::groups(&f.c, q, &f.policy()).is_err());
}
#[test]
fn migration_pure_reads_retention_purge_and_legacy_history() {
    let f = Fixture::new();
    let legacy = cleanup::preview(
        &f.c,
        CleanupRequest {
            session_id: "scan".into(),
            paths: vec![f.files[0].to_string_lossy().into()],
        },
        &f.policy(),
    )
    .unwrap();
    f.c.execute_batch(
        "DROP TABLE cache_batch_items;DROP TABLE cache_batches;PRAGMA user_version=7;",
    )
    .unwrap();
    let c = database::open(&f.home.join("app.sqlite3")).unwrap();
    assert_eq!(
        c.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        8
    );
    assert_eq!(cleanup::get(&c, &legacy.id).unwrap().status, "review");
    let s = f.select();
    let before = c.total_changes();
    c.execute_batch("PRAGMA query_only=ON").unwrap();
    cache_bulk::groups(&c, f.query(), &f.policy()).unwrap();
    cache_bulk::entries(&c, f.query(), &f.policy()).unwrap();
    cache_bulk::summary(&c, &s.id).unwrap();
    assert_eq!(c.total_changes(), before);
    c.execute_batch("PRAGMA query_only=OFF").unwrap();
    c.execute(
        "UPDATE cache_batches SET created_at='2000-01-01T00:00:00Z' WHERE id=?1",
        [&s.id],
    )
    .unwrap();
    database::retain(&c, 30).unwrap();
    assert!(cache_bulk::summary(&c, &s.id).is_err());
    let s = f.select();
    database::purge(&c).unwrap();
    assert!(cache_bulk::summary(&c, &s.id).is_err());
    assert!(f.files.iter().all(|p| p.exists()));
}
#[test]
fn over_250000_rows_no_truncation_individual_exceptions_and_keyset_pages() {
    let f = Fixture::new();
    let prefix = f
        .home
        .join("Library/Caches/large.namespace")
        .to_string_lossy()
        .into_owned();
    f.c.execute("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<250000) INSERT INTO files SELECT 'scan',?1||'/'||printf('%07d',x),?1,printf('%07d',x),'Caches',1,4096,0,'','fixture' FROM n",[&prefix]).unwrap();
    let mut q = f.query();
    q.group = Some("large.namespace".into());
    let s = cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q.clone(),
            revision: None,
            paths: None,
            selected: true,
            clear: false,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |e| {
            assert!(serde_json::to_string(&e).unwrap().len() < 3000);
            true
        },
    )
    .unwrap();
    assert_eq!(s.selected_count, 250001);
    q.batch_id = Some(s.id.clone());
    let page = cache_bulk::entries(&f.c, q.clone(), &f.policy()).unwrap();
    assert_eq!(page.items.len(), 100);
    assert!(page.items.iter().all(|i| i.selected));
    let s = cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q.clone(),
            revision: Some(s.revision),
            paths: Some(vec![page.items[0].path.clone()]),
            selected: false,
            clear: false,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert_eq!(s.selected_count, 250000);
    let later = format!("{prefix}/0200000");
    let s = cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q.clone(),
            revision: Some(s.revision),
            paths: Some(vec![later.clone()]),
            selected: false,
            clear: false,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert_eq!(s.selected_count, 249999);
    let s = cache_bulk::select(
        &f.c,
        CacheSelectionRequest {
            query: q.clone(),
            revision: Some(s.revision),
            paths: Some(vec![later]),
            selected: true,
            clear: false,
        },
        &f.policy(),
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert_eq!(s.selected_count, 250000);
    let groups = cache_bulk::groups(&f.c, q.clone(), &f.policy()).unwrap();
    assert_eq!(groups.groups[0].selected_count, 250000);
    q.cursor = page.next_cursor;
    let next = cache_bulk::entries(&f.c, q, &f.policy()).unwrap();
    assert!(next.items[0].path > page.items.last().unwrap().path);
    assert_eq!(s.allocated_bytes, 250000. * 4096.);
}
#[cfg(target_os = "macos")]
#[test]
#[ignore = "Only newly-created disposable cache files: native Trash, verify, restore"]
fn native_bulk_trash_and_fixture_restore() {
    let f = Fixture::new();
    let s = f.prepare();
    let r = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        cleanup::native_trash,
        |_| true,
    )
    .unwrap();
    assert_eq!(r.moved_count, 2);
    for row in f.page(&s.id).items {
        fs::rename(row.trash_path.unwrap(), &row.path).unwrap();
        assert_eq!(
            fs::read_to_string(row.path).unwrap(),
            "disposable fixture data"
        );
    }
}

#[test]
fn alias_deduplication_literal_namespace_and_exclusions() {
    let f = Fixture::new();
    let original = f.files[0].to_string_lossy();
    let data = format!("/System/Volumes/Data{original}");
    if coqui_engine::volumes::friendly(Path::new(&data)) == f.files[0] {
        f.c.execute("INSERT INTO files SELECT session_id,?2,?3,name,category,logical,allocated,directory,modified,qualification FROM files WHERE session_id='scan' AND path=?1",rusqlite::params![original,data,format!("/System/Volumes/Data{}",f.files[0].parent().unwrap().display())]).unwrap();
    }
    let s = f.select();
    assert_eq!(s.selected_count, 2);
    let p = Policy {
        home: f.home.clone(),
        exclusions: vec![f.files[0].clone()],
    };
    assert_eq!(
        cache_bulk::groups(&f.c, f.query(), &p).unwrap().groups[0].file_count,
        1
    );
    let odd = f.home.join("Library/Caches/odd%_namespace");
    fs::create_dir_all(&odd).unwrap();
    f.c.execute("INSERT INTO files SELECT session_id,?2,?3,'keep','Caches',logical,allocated,0,modified,qualification FROM files WHERE session_id='scan' AND path=?1",rusqlite::params![original,odd.join("keep").to_string_lossy(),odd.to_string_lossy()]).unwrap();
    let mut q = f.query();
    q.group = Some("odd%_namespace".into());
    assert_eq!(
        cache_bulk::entries(&f.c, q, &f.policy())
            .unwrap()
            .items
            .len(),
        1
    );
}
#[test]
fn final_review_delivery_failure_is_not_executable_and_frozen_pages_are_bounded() {
    let f = Fixture::new();
    let s = f.select();
    let s = cache_bulk::prepare(
        &f.c,
        &s.id,
        s.revision,
        &f.policy(),
        &AtomicBool::new(false),
        |e| e.status != "review",
    )
    .unwrap();
    assert_eq!(s.status, "cancelled");
    assert!(s.reviewed_at.is_none());
    assert!(s.expires_at.is_none());
    let root = f.home.join("Library/Caches/page.namespace");
    fs::create_dir_all(&root).unwrap();
    for n in 0..105 {
        fs::write(root.join(format!("file-{n:03}")), "fixture").unwrap();
    }
    let mut rows = vec![];
    storage::walk(
        &[root],
        &[],
        &AtomicBool::new(false),
        |r| {
            rows.push(r);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    let mut c = database::open(&f.home.join("app.sqlite3")).unwrap();
    database::insert_files(&mut c, "scan", &rows).unwrap();
    let s = f.prepare();
    let first = f.page(&s.id);
    assert_eq!(first.items.len(), 100);
    assert!(first.next_cursor.is_some());
    let next = cache_bulk::page(
        &f.c,
        CacheBulkPageQuery {
            id: s.id,
            cursor: first.next_cursor,
            status: Some("eligible".into()),
        },
    )
    .unwrap();
    assert_eq!(next.items.len(), 7);
    assert!(next.next_cursor.is_none());
}
#[test]
fn native_failures_and_unverified_returns_never_become_success() {
    let f = Fixture::new();
    let s = f.prepare();
    let mut calls = 0;
    let r = cache_bulk::execute(
        &f.c,
        Fixture::approval(&s),
        &f.policy(),
        &AtomicBool::new(false),
        |p| {
            calls += 1;
            if calls == 1 {
                anyhow::bail!("fixture native failure");
            }
            Ok(p.to_owned())
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(r.failed_count, 1);
    assert_eq!(r.unverified_count, 1);
    assert_eq!(r.moved_count, 0);
    assert_eq!(r.status, "partial");
    assert!(f.files.iter().all(|p| p.exists()));
}
