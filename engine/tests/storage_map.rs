use coqui_engine::{database, model::*, storage};
use std::{
    collections::BTreeMap,
    fs,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

fn session(id: &str) -> DiagnosticSession {
    DiagnosticSession {
        id: id.into(),
        volume_plan: None,
        coverage: vec![],
        started_at: now(),
        finished_at: Some(now()),
        status: "complete".into(),
        stage: "Complete".into(),
        scope: "fixture".into(),
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
    }
}
fn entry(path: &str, parent: &str, allocated: f64, directory: bool) -> FileEntry {
    FileEntry {
        path: path.into(),
        parent: parent.into(),
        name: path.rsplit('/').next().unwrap_or(path).into(),
        category: "Other".into(),
        logical_bytes: allocated,
        allocated_bytes: allocated,
        is_directory: directory,
        modified_at: now(),
        qualification: String::new(),
    }
}
fn query(parent: &str, limit: u32) -> StorageMapQuery {
    StorageMapQuery {
        session_id: "scan".into(),
        parent: parent.into(),
        metric: "allocated".into(),
        limit,
    }
}

#[test]
fn folder_metrics_count_tiny_descendants_and_deduplicate_links() {
    use std::os::unix::fs::MetadataExt;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let project = root.join("project");
    let debug = project.join("target/debug/incremental");
    fs::create_dir_all(&debug).unwrap();
    fs::write(project.join("Cargo.toml"), "[package]\nname='fixture'\n").unwrap();
    for n in 0..256 {
        fs::write(debug.join(format!("object{n:04}")), vec![1; 1024]).unwrap();
    }
    fs::write(project.join("large.bin"), vec![2; 128 * 1024]).unwrap();
    fs::hard_link(project.join("large.bin"), project.join("linked.bin")).unwrap();
    fs::write(project.join("cloud.icloud"), "not downloaded").unwrap();
    std::os::unix::fs::symlink(&root, project.join("cycle")).unwrap();
    let mut indexed = BTreeMap::new();
    let mut gaps = vec![];
    let summary = storage::walk_indexed_with_gaps(
        std::slice::from_ref(&root),
        &[],
        &AtomicBool::new(false),
        |item| {
            indexed.insert(item.entry.path.clone(), item);
            Ok(())
        },
        |_| Ok(()),
        |gap| {
            gaps.push(gap);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(summary.files, 258);
    let stats = indexed[debug.to_str().unwrap()].metrics.as_ref().unwrap();
    assert_eq!(stats.file_count, 256);
    assert_eq!(stats.small_file_count, 256);
    assert_eq!(
        stats.small_allocated_bytes,
        fs::metadata(debug.join("object0000")).unwrap().blocks() as f64 * 512. * 256.
    );
    assert_eq!(
        stats.largest_file_bytes,
        fs::metadata(debug.join("object0000")).unwrap().blocks() as f64 * 512.
    );
    assert_eq!(
        indexed[root.to_str().unwrap()]
            .metrics
            .as_ref()
            .unwrap()
            .file_count,
        258
    );
    assert!(gaps.iter().any(|gap| gap.reason == "cloudPlaceholder"));
    assert!(gaps.iter().any(|gap| gap.reason == "duplicateIdentity"));
    let mut c = database::open(&root.join("results.db")).unwrap();
    let mut scan = session("scan");
    scan.file_count = summary.files;
    scan.categories = summary.categories;
    scan.skipped_count = summary.skipped;
    database::save_session(&c, &scan).unwrap();
    database::insert_indexed_files(&mut c, "scan", &indexed.into_values().collect::<Vec<_>>())
        .unwrap();
    let map =
        database::storage_map(&c, &query(project.join("target").to_str().unwrap(), 60)).unwrap();
    assert_eq!(map.nodes[0].kind, "rustBuild");
    assert_eq!(map.nodes[0].file_count, Some(256.));
    assert_eq!(map.nodes[0].small_file_count, Some(256.));
    assert_eq!(map.file_count, Some(256.));
    let exported = database::indexed_files_after(&c, "scan", "").unwrap();
    assert!(exported
        .iter()
        .any(|item| item.metrics.as_ref().is_some_and(|m| m.file_count == 258)));
}

#[test]
fn map_totals_cover_every_child_and_remainder_not_just_first_page() {
    let fixture = tempfile::tempdir().unwrap();
    let mut c = database::open(&fixture.path().join("map.db")).unwrap();
    database::save_session(&c, &session("scan")).unwrap();
    let rows = (1..=100)
        .map(|n| entry(&format!("/root/f{n:03}"), "/root", n as f64, false))
        .collect::<Vec<_>>();
    database::insert_files(&mut c, "scan", &rows).unwrap();
    let map = database::storage_map(&c, &query("/root", 999)).unwrap();
    assert_eq!(map.nodes.len(), 60);
    assert_eq!(map.total_allocated_bytes, 5050.);
    assert_eq!(map.total_logical_bytes, 5050.);
    assert_eq!(map.file_count, Some(100.));
    assert_eq!(map.remainder_allocated_bytes, 820.);
    assert_eq!(map.remainder_file_count, Some(40.));
    assert_eq!(map.nodes[0].entry.name, "f100");
    let mut q = query("/root", 1);
    q.metric = "allocated;DROP TABLE files".into();
    assert_eq!(database::storage_map(&c, &q).unwrap().nodes.len(), 1);
    assert_eq!(
        database::storage_map(&c, &query("/empty", 20))
            .unwrap()
            .file_count,
        Some(0.)
    );
}

#[test]
fn legacy_folder_counts_remain_unavailable_and_no_analysis_grants_cleanup() {
    let fixture = tempfile::tempdir().unwrap();
    let mut c = database::open(&fixture.path().join("legacy.db")).unwrap();
    database::save_session(&c, &session("scan")).unwrap();
    database::insert_files(
        &mut c,
        "scan",
        &[entry("/project/target/debug", "", 1_000_000_000., true)],
    )
    .unwrap();
    let map = database::storage_map(&c, &query("", 60)).unwrap();
    assert_eq!(map.file_count, None);
    assert_eq!(map.nodes[0].small_file_count, None);
    assert!(map.qualification.contains("unavailable"));
    assert_eq!(map.nodes[0].kind, "rustBuild");
    assert!(map.nodes[0].qualification.contains("unavailable"));
    assert!(map.nodes[0].consequences.contains("validation"));
    assert_eq!(
        storage::category(std::path::Path::new(
            "/Users/test/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"
        )),
        "Virtual machines"
    );
    assert_eq!(
        storage::category(std::path::Path::new("/Users/test/Pictures/camera.raw")),
        "Photos & video"
    );
    let metrics = StorageMetrics {
        file_count: 10_000,
        small_file_count: 9_000,
        small_allocated_bytes: 40_000_000.,
        largest_file_bytes: 32_768.,
    };
    let docker = storage::identify(
        entry(
            "/Users/test/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw",
            "",
            20_000_000_000.,
            false,
        ),
        None,
        None,
    );
    assert_eq!(docker.kind, "virtualMachine");
    assert_eq!(docker.entry.category, "Virtual machines");
    let node = storage::identify(
        entry("/personal/collection", "/personal", 40_000_000., true),
        Some(&metrics),
        None,
    );
    assert_eq!(node.kind, "manySmallFiles");
    assert!(node.consequences.contains("do not establish disposability"));
    let generic = storage::identify(
        entry("/personal/build", "/personal", 100., true),
        Some(&metrics),
        None,
    );
    assert_ne!(generic.kind, "webBuild");
}

#[test]
fn missing_retained_root_index_is_unavailable_rather_than_an_empty_disk() {
    let fixture = tempfile::tempdir().unwrap();
    let c = database::open(&fixture.path().join("pruned.db")).unwrap();
    let mut s = session("scan");
    s.file_count = 100_000;
    s.allocated_bytes = 2_000_000_000.;
    database::save_session(&c, &s).unwrap();
    let map = database::storage_map(&c, &query("", 60)).unwrap();
    assert!(map.nodes.is_empty());
    assert_eq!(map.file_count, None);
    assert_eq!(map.remainder_file_count, None);
    assert!(map.qualification.contains("Folder index unavailable"));
    assert!(map
        .qualification
        .contains("does not mean the disk is empty"));
}

#[test]
fn coverage_keyset_export_walks_every_page_without_offset_or_duplicate_loss() {
    let fixture = tempfile::tempdir().unwrap();
    let c = database::open(&fixture.path().join("coverage.db")).unwrap();
    database::save_session(&c, &session("scan")).unwrap();
    c.execute_batch("WITH RECURSIVE sequence(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM sequence WHERE n<1201) INSERT INTO scan_gaps SELECT 'scan','/fixture','/fixture/gap'||n,'permissionDenied' FROM sequence;").unwrap();
    let mut after = 0;
    let mut total = 0;
    loop {
        let page = database::gaps_after(&c, "scan", after).unwrap();
        if page.gaps.is_empty() {
            break;
        }
        assert!(page.gaps.len() <= 500);
        assert!(page.after > after);
        after = page.after;
        total += page.gaps.len();
    }
    assert_eq!(total, 1201);
}

#[test]
fn current_schema_reads_remain_responsive_while_another_writer_is_held() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("wal.db");
    let mut writer = database::open(&path).unwrap();
    let mut preferences = AppSettings::default();
    preferences.appearance = "light".into();
    preferences.retention_days = 73;
    database::save_settings(&writer, &preferences).unwrap();
    database::save_session(&writer, &session("scan")).unwrap();
    database::insert_files(
        &mut writer,
        "scan",
        &[entry("/fixture/file", "", 42., false)],
    )
    .unwrap();
    writer
        .execute(
            "INSERT INTO removal_plans VALUES('legacy',?1,'blocked','{}')",
            [now()],
        )
        .unwrap();
    // Exercise the real schema-6 upgrade, preserving settings, history and action records.
    writer.execute_batch("DROP TABLE storage_metrics;DROP INDEX session_scope_retention;DROP INDEX scan_gap_export;PRAGMA user_version=6;").unwrap();
    drop(writer);
    let writer = database::open(&path).unwrap();
    assert_eq!(
        writer
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        7
    );
    writer
        .execute_batch("BEGIN IMMEDIATE;UPDATE sessions SET json=json WHERE id='scan';")
        .unwrap();
    let start = Instant::now();
    let reader = database::open(&path).unwrap();
    assert_eq!(database::history(&reader, 0, 10).unwrap().len(), 1);
    assert_eq!(
        database::storage_map(&reader, &query("", 60))
            .unwrap()
            .total_allocated_bytes,
        42.
    );
    assert_eq!(database::settings(&reader).unwrap().appearance, "light");
    assert_eq!(database::settings(&reader).unwrap().retention_days, 73);
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM removal_plans", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
    assert!(
        start.elapsed() < Duration::from_millis(500),
        "A read attempted to acquire the writer lock"
    );
    writer.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn aggregation_cancellation_stops_immediately_and_keeps_unfinished_context_qualified() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    for n in 0..200 {
        let dir = root.join(format!("folder{n}"));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("tiny"), "data").unwrap();
    }
    let cancel = AtomicBool::new(false);
    let mut indexed = BTreeMap::new();
    let summary = storage::walk_indexed_with_gaps(
        &[root],
        &[],
        &cancel,
        |item| {
            indexed.insert(item.entry.path.clone(), item);
            Ok(())
        },
        |summary| {
            if summary.stage == "Aggregating inspected folder allocation" {
                cancel.store(true, Ordering::Relaxed);
            }
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(summary.files, 200);
    let folders = indexed
        .values()
        .filter(|item| item.entry.is_directory)
        .collect::<Vec<_>>();
    assert_eq!(folders.len(), 201);
    assert!(folders
        .iter()
        .all(|item| item.metrics.is_none() && item.entry.qualification.contains("unfinished")));
    assert!(cancel.load(Ordering::Relaxed));
}

#[test]
fn retention_prunes_a_large_old_index_without_losing_summaries_or_coverage() {
    let fixture = tempfile::tempdir().unwrap();
    let c = database::open(&fixture.path().join("retention.db")).unwrap();
    let mut old = session("old");
    old.started_at = "2026-10-22T00:00:00Z".into();
    database::save_session(&c, &old).unwrap();
    database::save_session(&c, &session("new")).unwrap();
    c.execute_batch("WITH RECURSIVE sequence(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM sequence WHERE n<100000) INSERT INTO files SELECT 'old','/fixture/'||n,'/fixture',CAST(n AS TEXT),'Other',1,4096,0,'','' FROM sequence;").unwrap();
    c.execute(
        "INSERT INTO scan_gaps VALUES('old','/fixture','/fixture/protected','permissionDenied')",
        [],
    )
    .unwrap();
    let cancelled = AtomicBool::new(true);
    database::retain_after_scan(&c, 3650, &cancelled).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM files", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        100_000
    );
    database::retain(&c, 3650).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM files", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
    assert_eq!(database::history(&c, 0, 10).unwrap().len(), 2);
    assert_eq!(database::coverage(&c, "old", 0).unwrap().len(), 1);
    let mut after = 0;
    let mut total = 0;
    loop {
        let page = database::gaps_after(&c, "old", after).unwrap();
        if page.gaps.is_empty() {
            break;
        }
        after = page.after;
        total += page.gaps.len();
    }
    assert_eq!(total, 1);
}
