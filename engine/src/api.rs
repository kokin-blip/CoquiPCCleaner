use crate::{cleanup, database, developer, model::*, platform, storage};
use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::Duration,
};

struct Engine {
    path: PathBuf,
    active: AtomicBool,
    cancel: AtomicBool,
    monitoring: AtomicBool,
    stop_monitor: AtomicBool,
    cancel_cleanup: AtomicBool,
    cancel_developer: AtomicBool,
    cancel_work: AtomicBool,
}
static ENGINE: OnceLock<Mutex<Option<Arc<Engine>>>> = OnceLock::new();
fn engine() -> Result<Arc<Engine>> {
    ENGINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| anyhow::anyhow!("Engine lock failed"))?
        .clone()
        .context("Engine not initialized")
}

#[cfg(test)]
#[test]
fn internal_scan_database_is_the_journal_database() {
    let fixture = tempfile::tempdir().unwrap();
    let data = fixture.path().join("state");
    std::fs::create_dir(&data).unwrap();
    let root = fixture.path().join("files");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("disposable.txt"), "fixture").unwrap();
    let path = data.join("diagnostics.sqlite3");
    init_engine_database(path.clone()).unwrap();
    scan(
        ScanRequest {
            mode: "selected".into(),
            roots: vec![root.to_string_lossy().into_owned()],
            excluded_paths: vec![],
            plan_id: None,
        },
        |_| true,
    )
    .unwrap();
    let c = database::open(&path).unwrap();
    let sessions = database::history(&c, 0, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    let count: u32 = c
        .query_row("SELECT count(*) FROM files", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2); // The selected directory and its one file.
    assert!(!data.join("coqui.sqlite3").exists());
    let mut elevated = sessions[0].clone();
    elevated.id = uuid::Uuid::new_v4().to_string();
    save_administrator_session(&c, &elevated).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM administrator_records WHERE kind='scan'",
            [],
            |r| r.get::<_, u32>(0)
        )
        .unwrap(),
        1
    );
    purge_administrator_mirrors(&c).unwrap();
    let kept = database::history(&c, 0, 10).unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].id, sessions[0].id);
}

pub fn init_engine(data_directory: String) -> Result<AppSettings> {
    let dir = PathBuf::from(data_directory);
    anyhow::ensure!(
        dir.is_absolute(),
        "Application data directory must be absolute"
    );
    std::fs::create_dir_all(&dir)?;
    init_engine_database(dir.join("coqui.sqlite3"))
}

// The helper uses the same validated per-user database for scans and journals.
// This is internal Rust API, never a caller-selected IPC database path.
pub(crate) fn init_engine_database(path: PathBuf) -> Result<AppSettings> {
    init_database(path, None)
}

pub(crate) fn init_engine_database_for_scan(
    path: PathBuf,
    cancel: &AtomicBool,
) -> Result<AppSettings> {
    init_database(path, Some(cancel))
}

fn init_database(path: PathBuf, cancel: Option<&AtomicBool>) -> Result<AppSettings> {
    let mut guard = ENGINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| anyhow::anyhow!("Engine lock failed"))?;
    if let Some(existing) = guard.as_ref() {
        anyhow::ensure!(
            !existing.active.load(Ordering::SeqCst) && !existing.monitoring.load(Ordering::SeqCst),
            "Stop scan and monitoring before changing engine location"
        );
    }
    let c = database::open(&path)?;
    database::recover_interrupted(&c)?;
    let settings = database::settings(&c)?;
    if let Some(cancel) = cancel {
        database::retain_after_scan(&c, settings.retention_days, cancel)?;
    } else {
        database::retain(&c, settings.retention_days)?;
    }
    *guard = Some(Arc::new(Engine {
        path,
        active: AtomicBool::new(false),
        cancel: AtomicBool::new(false),
        monitoring: AtomicBool::new(false),
        stop_monitor: AtomicBool::new(false),
        cancel_cleanup: AtomicBool::new(false),
        cancel_developer: AtomicBool::new(false),
        cancel_work: AtomicBool::new(false),
    }));
    Ok(settings)
}
pub fn get_settings() -> Result<AppSettings> {
    database::settings(&database::open(&engine()?.path)?)
}
pub fn update_settings(settings: AppSettings) -> Result<()> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan, validation or cleanup before changing preferences"
    );
    let _guard = ActiveGuard(&e.active);
    let c = database::open(&e.path)?;
    if settings.administrator_enabled && !database::settings(&c)?.administrator_enabled {
        anyhow::ensure!(
            administrator_status()?.compatible,
            "A matching installed administrator helper is required"
        );
        e.cancel_work.store(false, Ordering::SeqCst);
        crate::helper::begin_authorization(&e.cancel_work)?;
    }
    if crate::helper::protocol() == 2
        && (settings.administrator_enabled || database::settings(&c)?.administrator_enabled)
    {
        let preferences = crate::privileged::Preferences {
            excluded_paths: settings.excluded_paths.clone(),
            protected_paths: crate::applications::preferences(&c)?.protected_paths,
            retention_days: settings.retention_days,
        };
        crate::helper::production(
            crate::privileged::Operation::SetPreferences { preferences },
            false,
        )?;
    }
    database::save_settings(&c, &settings)?;
    if !settings.administrator_enabled {
        crate::helper::clear_authorization();
    }
    database::retain(&c, settings.retention_days)
}
pub fn scan_history(offset: u32, limit: u32) -> Result<Vec<DiagnosticSession>> {
    database::history(&database::open(&engine()?.path)?, offset, limit)
}
pub fn get_session(id: String) -> Result<DiagnosticSession> {
    database::session(&database::open(&engine()?.path)?, &id)
}
pub fn storage_entries(query: StorageQuery) -> Result<Vec<FileEntry>> {
    let c = database::open(&engine()?.path)?;
    if database::settings(&c)?.administrator_enabled
        && c.query_row(
            "SELECT EXISTS(SELECT 1 FROM administrator_records WHERE id=?1 AND kind='scan')",
            [&query.session_id],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Ok(serde_json::from_value(crate::helper::production(
            crate::privileged::Operation::Files { query },
            false,
        )?)?);
    }
    database::files(&c, &query)
}
pub fn storage_map(query: StorageMapQuery) -> Result<StorageMap> {
    let c = database::open(&engine()?.path)?;
    if database::settings(&c)?.administrator_enabled
        && c.query_row(
            "SELECT EXISTS(SELECT 1 FROM administrator_records WHERE id=?1 AND kind='scan')",
            [&query.session_id],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Ok(serde_json::from_value(crate::helper::production(
            crate::privileged::Operation::StorageMap { query },
            false,
        )?)?);
    }
    database::storage_map(&c, &query)
}
pub fn clear_diagnostic_data() -> Result<()> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Cancel the active scan before clearing history"
    );
    let _guard = ActiveGuard(&e.active);
    database::purge(&database::open(&e.path)?)
}
pub fn cancel_scan() -> Result<()> {
    engine()?.cancel.store(true, Ordering::SeqCst);
    Ok(())
}
pub fn stop_monitoring() -> Result<()> {
    engine()?.stop_monitor.store(true, Ordering::SeqCst);
    Ok(())
}

struct ActiveGuard<'a>(&'a AtomicBool);
impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

pub fn scan(request: ScanRequest, mut send: impl FnMut(ScanEvent) -> bool) -> Result<()> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "A scan is already running"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel.store(false, Ordering::SeqCst);
    let mut c = database::open(&e.path)?;
    let (roots, root_errors) = if request.mode == "deep" {
        let proposed = crate::volumes::approved_roots(&c, &request)?;
        storage::roots(&ScanRequest {
            mode: "selected".into(),
            roots: proposed
                .iter()
                .map(|r| r.to_string_lossy().into_owned())
                .collect(),
            ..request.clone()
        })?
    } else {
        storage::roots(&request)?
    };
    let mut excluded = request
        .excluded_paths
        .iter()
        .map(|p| {
            let path = PathBuf::from(p);
            path.canonicalize().unwrap_or(path)
        })
        .collect::<Vec<_>>();
    // The app's changing index and database must never inflate its scan results.
    if let Some(parent) = e.path.parent() {
        excluded.push(parent.to_owned());
    }
    let mut s = DiagnosticSession {
        volume_plan: if request.mode == "deep" {
            {
                let mut plan = crate::volumes::get(
                    &c,
                    request.plan_id.as_deref().context("Missing scope plan")?,
                )?;
                for volume in &mut plan.volumes {
                    volume.selected = request.roots.contains(&volume.mount);
                }
                Some(plan)
            }
        } else {
            None
        },
        coverage: vec![],
        id: uuid::Uuid::new_v4().to_string(),
        started_at: now(),
        finished_at: None,
        status: "running".into(),
        stage: "Hardware inventory".into(),
        scope: if roots.is_empty() {
            request.roots.join("; ")
        } else {
            roots
                .iter()
                .map(|r| r.to_string_lossy())
                .collect::<Vec<_>>()
                .join("; ")
        },
        file_count: 0,
        logical_bytes: 0.,
        allocated_bytes: 0.,
        skipped_count: 0,
        snapshot: None,
        findings: vec![],
        permissions: vec![],
        categories: vec![],
        candidates: vec![],
        errors: root_errors,
    };
    database::save_session(&c, &s)?;
    let _ = send(ScanEvent { session: s.clone() });
    let result = (|| -> Result<()> {
        match platform::snapshot() {
            Ok(v) => s.snapshot = Some(v),
            Err(_) => s
                .errors
                .push("Hardware inventory could not be completed.".into()),
        }
        s.stage = "Storage metadata • counting accessible files".into();
        database::save_session(&c, &s)?;
        if !send(ScanEvent { session: s.clone() }) {
            e.cancel.store(true, Ordering::SeqCst);
        }
        let batch = std::cell::RefCell::new(Vec::with_capacity(250));
        let connection = std::cell::RefCell::new(&mut c);
        let session = std::cell::RefCell::new(&mut s);
        let id = session.borrow().id.clone();
        let mut last_event = std::time::Instant::now();
        let gaps = std::cell::RefCell::new(Vec::with_capacity(250));
        let summary = storage::walk_indexed_with_gaps(
            &roots,
            &excluded,
            &e.cancel,
            |f| {
                let mut b = batch.borrow_mut();
                b.push(f);
                if b.len() >= 250 {
                    database::insert_indexed_files(&mut connection.borrow_mut(), &id, &b)?;
                    b.clear();
                }
                Ok(())
            },
            |w| {
                if !batch.borrow().is_empty() {
                    database::insert_indexed_files(
                        &mut connection.borrow_mut(),
                        &id,
                        &batch.borrow(),
                    )?;
                    batch.borrow_mut().clear();
                }
                let mut s = session.borrow_mut();
                s.file_count = w.files;
                s.logical_bytes = w.logical;
                s.allocated_bytes = w.allocated;
                s.skipped_count = w.skipped;
                s.coverage = w.coverage.clone();
                s.permissions = w.permissions.clone();
                // Keep bounded omission context even if a later database write fails.
                for error in &w.errors {
                    if !s.errors.contains(error) && s.errors.len() < 200 {
                        s.errors.push(error.clone());
                    }
                }
                s.stage = w.stage.clone();
                if last_event.elapsed() >= Duration::from_millis(250) || w.files == 0 {
                    database::save_session(&connection.borrow(), &s)?;
                    if !send(ScanEvent {
                        session: (**s).clone(),
                    }) {
                        e.cancel.store(true, Ordering::SeqCst);
                    }
                    last_event = std::time::Instant::now();
                }
                Ok(())
            },
            |g| {
                let mut b = gaps.borrow_mut();
                b.push(g);
                if b.len() >= 250 {
                    database::insert_gaps(&mut connection.borrow_mut(), &id, &b)?;
                    b.clear();
                }
                Ok(())
            },
        )?;
        if !batch.borrow().is_empty() {
            database::insert_indexed_files(&mut connection.borrow_mut(), &id, &batch.borrow())?;
        }
        if !gaps.borrow().is_empty() {
            database::insert_gaps(&mut connection.borrow_mut(), &id, &gaps.borrow())?;
        }
        s.coverage = summary.coverage;
        s.file_count = summary.files;
        s.logical_bytes = summary.logical;
        s.allocated_bytes = summary.allocated;
        s.skipped_count = summary.skipped;
        s.permissions = summary.permissions;
        for error in summary.errors {
            if !s.errors.contains(&error) {
                s.errors.push(error);
            }
        }
        s.categories = summary.categories;
        s.candidates = summary.candidates;
        Ok(())
    })();
    if let Err(error) = &result {
        s.errors.push(format!("Scan stopped because collection or persistence failed: {}. Retained results may be incomplete; check available application storage before retrying.",scan_error(error)));
        for coverage in &mut s.coverage {
            if coverage.state == "running" {
                coverage.state = "interrupted".into();
            }
        }
    }
    s.status = if e.cancel.load(Ordering::SeqCst) {
        "cancelled"
    } else if result.is_err() {
        "failed"
    } else if !s.permissions.is_empty() || !s.errors.is_empty() {
        "partial"
    } else {
        "complete"
    }
    .into();
    s.finished_at = Some(now());
    s.stage = match s.status.as_str() {
        "complete" => "Scan complete",
        "partial" => "Scan complete with coverage gaps",
        "cancelled" => "Scan cancelled; partial results retained",
        _ => "Scan failed; available results retained",
    }
    .into();
    s.findings = findings(&s);
    database::save_session(&c, &s)?;
    // Publish terminal collection before potentially substantial index retention.
    let _ = send(ScanEvent { session: s.clone() });
    let settings = database::settings(&c)?;
    if let Err(error) = database::retain_after_scan(&c, settings.retention_days, &e.cancel) {
        s.errors.push(format!("Scan results were saved, but older index retention did not finish: {}. Retention will retry on a later app start.",scan_error(&error)));
        database::save_session(&c, &s)?;
    }
    let _ = send(ScanEvent { session: s });
    Ok(())
}

fn scan_error(error: &anyhow::Error) -> String {
    // Keep actionable SQLite/I/O classes without persisting raw paths or command output.
    for cause in error.chain() {
        if let Some(sql) = cause.downcast_ref::<rusqlite::Error>() {
            if let rusqlite::Error::SqliteFailure(code, _) = sql {
                return format!("SQLite {:?} (code {})", code.code, code.extended_code);
            }
        }
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            return format!("filesystem {:?}", io.kind());
        }
    }
    "collection or persistence error (detail unavailable)".into()
}

fn findings(s: &DiagnosticSession) -> Vec<DiagnosticFinding> {
    let mut result = vec![];
    if !s.permissions.is_empty() || !s.errors.is_empty() {
        result.push(DiagnosticFinding{id:"coverage".into(),title:"Some locations could not be inspected".into(),explanation:"Results describe the accessible scan scope, not the entire machine. Review skipped locations and permission guidance before drawing conclusions.".into(),severity:"attention".into(),confidence:"Observed".into(),kind:"coverage".into(),evidence:vec![DiagnosticEvidence::observed("Reported coverage gaps",(s.permissions.len()+s.errors.len()).to_string(),"reported entries (capped)","Storage walker")]});
    }
    if let Some(machine) = &s.snapshot {
        for d in machine.disks.iter().filter(|d| {
            d.mount != "/"
                || !machine
                    .disks
                    .iter()
                    .any(|other| other.mount == "/System/Volumes/Data")
        }) {
            if d.total_bytes > 0. && d.available_bytes / d.total_bytes < 0.10 {
                result.push(DiagnosticFinding{id:format!("capacity:{}",d.mount),title:format!("Review available space on {}",d.mount),explanation:"Less than 10% of this filesystem's reported capacity is available. This is a capacity observation, not proof of a performance problem. APFS volumes can share container space.".into(),severity:"attention".into(),confidence:"Observed".into(),kind:"storage".into(),evidence:vec![DiagnosticEvidence::observed("Available capacity",d.available_bytes.to_string(),"bytes","macOS filesystem capacity")]});
            }
        }
        result.push(DiagnosticFinding{id:"startup".into(),title:format!("{} startup configurations detected",machine.startup_items.len()),explanation:"Configured entries are not measured startup delays. Review mechanisms and use Login Items in System Settings to inspect managed registrations.".into(),severity:"info".into(),confidence:"Partial coverage".into(),kind:"startup".into(),evidence:vec![]});
    }
    if !s.candidates.is_empty() {
        result.push(DiagnosticFinding{id:"caches".into(),title:"Cache locations are available for inspection".into(),explanation:"These are known cache directories, not confirmed junk. Review their origin and consequences. Individual eligible files can be proposed for a separate approved Trash operation.".into(),severity:"info".into(),confidence:"Rule-based candidate".into(),kind:"storage".into(),evidence:vec![]});
    }
    result
}

pub fn monitor(mut send: impl FnMut(LiveMetrics) -> bool) -> Result<()> {
    let e = engine()?;
    anyhow::ensure!(
        e.monitoring
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Monitoring already active"
    );
    let _guard = ActiveGuard(&e.monitoring);
    e.stop_monitor.store(false, Ordering::SeqCst);
    let mut sys = sysinfo::System::new_all();
    while !e.stop_monitor.load(Ordering::SeqCst) {
        for _ in 0..20 {
            if e.stop_monitor.load(Ordering::SeqCst) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if !send(platform::metrics(&mut sys)) {
            break;
        }
    }
    Ok(())
}

fn cleanup_policy(e: &Engine) -> Result<cleanup::Policy> {
    let settings = database::settings(&database::open(&e.path)?)?;
    let mut exclusions = settings
        .excluded_paths
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if let Some(parent) = e.path.parent() {
        exclusions.push(parent.to_owned());
    }
    Ok(cleanup::Policy {
        home: platform::home()?,
        exclusions,
    })
}
pub fn cleanup_files(session_id: String, offset: u32) -> Result<CleanupFilePage> {
    let e = engine()?;
    cleanup::candidates(
        &database::open(&e.path)?,
        &session_id,
        offset,
        &cleanup_policy(&e)?,
    )
}
pub fn preview_cleanup(request: CleanupRequest) -> Result<CleanupPlan> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan or cleanup to finish"
    );
    let _guard = ActiveGuard(&e.active);
    cleanup::preview(&database::open(&e.path)?, request, &cleanup_policy(&e)?)
}
pub fn preview_project_cleanup(request: CleanupRequest) -> Result<CleanupPlan> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan, validation or cleanup to finish"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel_cleanup.store(false, Ordering::SeqCst);
    cleanup::preview_project(
        &database::open(&e.path)?,
        request,
        &cleanup_policy(&e)?,
        &e.cancel_cleanup,
    )
}
pub fn project_cleanup_entries(id: String, offset: u32) -> Result<ProjectCleanupPage> {
    cleanup::project_entries(&database::open(&engine()?.path)?, &id, offset)
}
pub fn cleanup_history(offset: u32, limit: u32) -> Result<Vec<CleanupPlan>> {
    cleanup::history(&database::open(&engine()?.path)?, offset, limit)
}
pub fn get_cleanup_plan(id: String) -> Result<CleanupPlan> {
    cleanup::get(&database::open(&engine()?.path)?, &id)
}
pub fn decline_cleanup(id: String) -> Result<CleanupPlan> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active operation to finish"
    );
    let _guard = ActiveGuard(&e.active);
    cleanup::decline(&database::open(&e.path)?, &id)
}
pub fn execute_cleanup(
    approval: CleanupApproval,
    send: impl FnMut(CleanupPlan) -> bool,
) -> Result<CleanupPlan> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan or cleanup to finish"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel_cleanup.store(false, Ordering::SeqCst);
    cleanup::execute(
        &database::open(&e.path)?,
        approval,
        &cleanup_policy(&e)?,
        &e.cancel_cleanup,
        cleanup::native_trash,
        send,
    )
}
pub fn cancel_cleanup() -> Result<()> {
    engine()?.cancel_cleanup.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn developer_report(session_id: String) -> Result<Option<DeveloperReport>> {
    developer::get(&database::open(&engine()?.path)?, &session_id)
}
pub fn analyze_developer(
    session_id: String,
    send: impl FnMut(DeveloperReport) -> bool,
) -> Result<DeveloperReport> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan, validation or cleanup to finish"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel_developer.store(false, Ordering::SeqCst);
    let policy = cleanup_policy(&e)?;
    developer::analyze(
        &database::open(&e.path)?,
        &session_id,
        &policy.exclusions,
        &e.cancel_developer,
        send,
    )
}
pub fn cancel_developer() -> Result<()> {
    engine()?.cancel_developer.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn cancel_work() -> Result<()> {
    engine()?.cancel_work.store(true, Ordering::SeqCst);
    Ok(())
}
fn with_work<T>(action: impl FnOnce(&Engine, &rusqlite::Connection) -> Result<T>) -> Result<T> {
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active scan, analysis or cleanup"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel_work.store(false, Ordering::SeqCst);
    action(&e, &database::open(&e.path)?)
}
pub fn analyze_duplicates(
    session_id: String,
    send: impl FnMut(DuplicateReport) -> bool,
) -> Result<DuplicateReport> {
    with_work(|e, c| {
        crate::duplicates::analyze(c, &session_id, &cleanup_policy(e)?, &e.cancel_work, send)
    })
}
pub fn preview_duplicate_cleanup(request: DuplicateCleanupRequest) -> Result<CleanupPlan> {
    with_work(|e, c| cleanup::preview_duplicates(c, request, &cleanup_policy(e)?, &e.cancel_work))
}
pub fn capture_baseline(
    request: BaselineRequest,
    send: impl FnMut(Baseline) -> bool,
) -> Result<Baseline> {
    with_work(|e, c| crate::baseline::capture(c, request, &e.cancel_work, send))
}
pub fn compare_baselines(before: String, after: String) -> Result<BaselineComparison> {
    crate::baseline::compare(&database::open(&engine()?.path)?, &before, &after)
}
pub fn create_manual_plan(kind: String, requested: String) -> Result<ManualPlan> {
    with_work(|_, c| crate::baseline::plan(c, &kind, &requested))
}
pub fn verify_manual_plan(id: String, performed: bool) -> Result<ManualPlan> {
    with_work(|_, c| crate::baseline::verify_plan(c, &id, performed))
}
pub fn ask_coqui(
    id: Option<String>,
    session_id: Option<String>,
    text: String,
) -> Result<Investigation> {
    with_work(|_, c| crate::assistant::ask(c, id.as_deref(), session_id.as_deref(), &text))
}
pub fn assess_workload(request: AdviceRequest) -> Result<AdviceReport> {
    with_work(|_, c| crate::advice::assess(c, request))
}
pub fn analyze_resources(session_id: String) -> Result<ResourceReview> {
    with_work(|e, c| {
        crate::resources::analyze(
            c,
            &session_id,
            &cleanup_policy(e)?.exclusions,
            &e.cancel_work,
        )
    })
}
pub fn activity_records(kind: Option<String>, offset: u32) -> Result<Vec<ActivityRecord>> {
    crate::records::list(&database::open(&engine()?.path)?, kind.as_deref(), offset)
}
pub fn get_record(kind: String, id: String) -> Result<RetainedRecord> {
    let c = database::open(&engine()?.path)?;
    Ok(match kind.as_str() {
        "firmwareComparison" => {
            RetainedRecord::FirmwareComparison(crate::records::get(&c, &kind, &id)?)
        }
        "firmware" => RetainedRecord::Firmware(crate::records::get(&c, &kind, &id)?),
        "firmwarePlan" => RetainedRecord::FirmwarePlan(crate::records::get(&c, &kind, &id)?),
        "duplicates" => RetainedRecord::Duplicates(crate::records::get(&c, &kind, &id)?),
        "baseline" => RetainedRecord::Baseline(crate::records::get(&c, &kind, &id)?),
        "manualPlan" => RetainedRecord::ManualPlan(crate::records::get(&c, &kind, &id)?),
        "maintenance" => RetainedRecord::Maintenance(crate::records::get(&c, &kind, &id)?),
        "investigation" => RetainedRecord::Investigation(crate::records::get(&c, &kind, &id)?),
        "advice" => RetainedRecord::Advice(crate::records::get(&c, &kind, &id)?),
        "resources" => RetainedRecord::Resources(crate::records::get(&c, &kind, &id)?),
        _ => anyhow::bail!("Unknown record type"),
    })
}
pub fn maintenance_rule() -> Result<Option<MaintenanceRule>> {
    crate::maintenance::get(&database::open(&engine()?.path)?)
}
pub fn save_maintenance(rule: MaintenanceRule) -> Result<()> {
    with_work(|_, c| crate::maintenance::save(c, &rule))
}
pub fn maintenance_tick() -> Result<Option<MaintenanceEvent>> {
    let e = engine()?;
    if e.active
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(None);
    }
    let guard = ActiveGuard(&e.active);
    let mut c = database::open(&e.path)?;
    let Some((mut rule, event)) = crate::maintenance::due(&mut c)? else {
        return Ok(None);
    };
    let prefs = database::settings(&c)?;
    for p in prefs.excluded_paths {
        if !rule.scope.excluded_paths.contains(&p) {
            rule.scope.excluded_paths.push(p);
        }
    }
    drop(guard);
    if rule.mode == "reminder" {
        return Ok(Some(event));
    }
    let mut result = None;
    let scan = scan(rule.scope, |e| {
        result = Some(e.session);
        true
    });
    let (status,id,message)=match (scan,result){(Ok(()),Some(s))=>{let alerts=s.snapshot.as_ref().map(|m|m.disks.iter().filter(|d|!d.removable && d.total_bytes>0. && (1.-d.available_bytes/d.total_bytes)*100.>=rule.capacity_threshold_percent as f64).map(|d|format!("{} meets the {}% capacity alert threshold",d.mount,rule.capacity_threshold_percent)).collect::<Vec<_>>().join("; ")).unwrap_or_default();(s.status.clone(),Some(s.id),format!("Scheduled read-only scan: {}. {} No cleanup executed.",s.status,alerts))},(Err(error),_)=> ("failed".into(),None,format!("Scheduled scan did not finish: {error}. No automatic retry until the next due occurrence.")),_=> ("failed".into(),None,"No terminal scan result was retained".into())};
    Ok(Some(crate::maintenance::finish(
        &c, event, &status, id, message,
    )?))
}

pub fn audit_firmware(
    request: FirmwareRequest,
    send: impl FnMut(FirmwareReport) -> bool,
) -> Result<FirmwareReport> {
    with_work(|e, c| crate::firmware::audit(c, request, &e.cancel_work, send))
}
pub fn firmware_plan(report_id: String, kind: FirmwarePlanKind) -> Result<FirmwarePlan> {
    with_work(|_, c| crate::firmware::plan(c, &report_id, kind))
}
pub fn acknowledge_firmware_plan(id: String, accepted: bool) -> Result<FirmwarePlan> {
    with_work(|_, c| crate::firmware::acknowledge(c, &id, accepted))
}
pub fn observe_firmware_plan(
    id: String,
    performed: bool,
    send: impl FnMut(FirmwareReport) -> bool,
) -> Result<FirmwarePlan> {
    with_work(|e, c| crate::firmware::observe(c, &id, performed, &e.cancel_work, send))
}

pub fn compare_firmware(before: String, after: String) -> Result<FirmwareComparison> {
    with_work(|_, c| crate::firmware_review::compare(c, &before, &after))
}
pub fn firmware_center() -> Result<FirmwareCenter> {
    crate::firmware_review::center(&database::open(&engine()?.path)?)
}
pub fn save_firmware_reminder(enabled: bool, days: u32) -> Result<FirmwareCenter> {
    with_work(|_, c| crate::firmware_review::save_reminder(c, enabled, days))
}
pub fn dismiss_firmware_notice(id: String) -> Result<FirmwareCenter> {
    with_work(|_, c| crate::firmware_review::dismiss(c, &id))
}

pub fn disk_scan_plan() -> Result<DiskScanPlan> {
    crate::volumes::plan(&database::open(&engine()?.path)?)
}
pub fn scan_coverage(id: String, offset: u32) -> Result<Vec<CoverageGap>> {
    let c = database::open(&engine()?.path)?;
    if database::settings(&c)?.administrator_enabled
        && c.query_row(
            "SELECT EXISTS(SELECT 1 FROM administrator_records WHERE id=?1 AND kind='scan')",
            [&id],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Ok(serde_json::from_value(crate::helper::production(
            crate::privileged::Operation::Coverage { id, offset },
            false,
        )?)?);
    }
    database::coverage(&c, &id, offset)
}

fn removal_policy(e: &Engine) -> Result<crate::removal::Policy> {
    let mut p = cleanup_policy(e)?;
    p.exclusions.extend(
        crate::applications::preferences(&database::open(&e.path)?)?
            .protected_paths
            .iter()
            .map(PathBuf::from),
    );
    let exe = std::env::current_exe()?;
    let self_app = exe
        .ancestors()
        .find(|p| p.extension().is_some_and(|s| s == "app"))
        .map(std::path::Path::to_owned);
    Ok(crate::removal::Policy {
        home: p.home,
        exclusions: p.exclusions,
        self_app,
        administrator_uid: None,
    })
}
pub fn application_page(
    query: String,
    sort: String,
    offset: u32,
    protected_only: bool,
) -> Result<crate::applications::ApplicationPage> {
    crate::applications::indexed_page(
        &database::open(&engine()?.path)?,
        &query,
        &sort,
        offset,
        protected_only,
    )
}
pub fn refresh_application_inventory() -> Result<()> {
    with_work(|e, c| {
        crate::applications::page(
            c,
            &removal_policy(e)?,
            &crate::applications::preferences(c)?,
            "",
            "name",
            0,
        )?;
        Ok(())
    })
}
pub fn inspect_application(path: String) -> Result<crate::applications::ApplicationFootprint> {
    with_work(|e, _| {
        let policy = removal_policy(e)?;
        Ok(crate::applications::qualify_related(
            crate::applications::inspect(std::path::Path::new(&path), &policy)?,
            &policy,
        ))
    })
}
pub fn application_preferences() -> Result<crate::applications::ApplicationPreferences> {
    crate::applications::preferences(&database::open(&engine()?.path)?)
}
pub fn save_application_preferences(p: crate::applications::ApplicationPreferences) -> Result<()> {
    with_work(|_, c| {
        let s = database::settings(c)?;
        if s.administrator_enabled && crate::helper::protocol() == 2 {
            crate::helper::production(
                crate::privileged::Operation::SetPreferences {
                    preferences: crate::privileged::Preferences {
                        excluded_paths: s.excluded_paths,
                        protected_paths: p.protected_paths.clone(),
                        retention_days: s.retention_days,
                    },
                },
                false,
            )?;
        }
        crate::applications::save_preferences(c, &p)
    })
}
pub fn preview_removal(r: crate::removal::RemovalRequest) -> Result<crate::removal::RemovalPlan> {
    with_work(|e, c| crate::removal::preview(c, r, &removal_policy(e)?, &e.cancel_work))
}
pub fn preview_restore(id: String, paths: Vec<String>) -> Result<crate::removal::RemovalPlan> {
    with_work(|e, c| {
        if is_administrator_plan(c, &id)? {
            let preferences = administrator_preferences(e)?;
            let job = wait_administrator(
                crate::privileged::Operation::PreviewRestore {
                    id,
                    paths,
                    preferences,
                },
                &e.cancel_work,
                |_| true,
            )?;
            let p = job.plan.context("No administrator restore review")?;
            mirror_administrator(c, &p)?;
            return Ok(p);
        }
        crate::removal::preview_restore(c, &id, &paths, &removal_policy(e)?, &e.cancel_work)
    })
}
pub fn execute_removal(
    a: crate::removal::RemovalApproval,
    mut send: impl FnMut(crate::removal::RemovalPlan) -> bool,
) -> Result<crate::removal::RemovalPlan> {
    with_work(|e, c| {
        if is_administrator_plan(c, &a.id)? {
            let preferences = administrator_preferences(e)?;
            let job = wait_administrator(
                crate::privileged::Operation::Execute {
                    approval: a,
                    preferences,
                },
                &e.cancel_work,
                |j| {
                    j.plan
                        .as_ref()
                        .is_none_or(|p| mirror_administrator(c, p).is_ok() && send(p.clone()))
                },
            )?;
            let p = job.plan.context("No administrator removal result")?;
            mirror_administrator(c, &p)?;
            // Refresh authoritative public records so restored originals cannot retain stale offers.
            let mut offset = 0;
            loop {
                let plans: Vec<crate::removal::RemovalPlan> =
                    serde_json::from_value(crate::helper::production(
                        crate::privileged::Operation::History { offset },
                        false,
                    )?)?;
                if plans.is_empty() {
                    break;
                }
                for p in &plans {
                    mirror_administrator(c, p)?;
                }
                offset += plans.len() as u32;
            }
            return Ok(p);
        }
        crate::removal::native_execute(c, a, &removal_policy(e)?, &e.cancel_work, send)
    })
}
pub fn removal_history(offset: u32) -> Result<Vec<crate::removal::RemovalPlan>> {
    let e = engine()?;
    let c = database::open(&e.path)?;
    if database::settings(&c)?.administrator_enabled {
        // Refresh retained private records as well as plans started by this app.
        // A connection failure must not erase previously imported recovery data.
        for page in (0..=offset / 50).map(|page| page * 50) {
            let value = crate::helper::production(
                crate::privileged::Operation::History { offset: page },
                false,
            )?;
            let records: Vec<crate::removal::RemovalPlan> = serde_json::from_value(value)?;
            let count = records.len();
            for record in records {
                mirror_administrator(&c, &record)?;
            }
            if count < 50 {
                break;
            }
        }
    }
    let mut q=c.prepare("SELECT payload FROM (SELECT json_extract(json,'$.plan') payload,created_at FROM removal_plans UNION ALL SELECT json payload,created_at FROM administrator_records WHERE kind='removal') ORDER BY created_at DESC LIMIT 50 OFFSET ?1")?;
    let rows = q.query_map([offset], |r| r.get::<_, String>(0))?;
    rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
}

pub fn removal_entries(id: String, index: u32, offset: u32) -> Result<Vec<FileEntry>> {
    with_work(|e, c| {
        if is_administrator_plan(c, &id)? {
            let _ = administrator_preferences(e)?;
            return Ok(serde_json::from_value(crate::helper::production(
                crate::privileged::Operation::Entries { id, index, offset },
                false,
            )?)?);
        }
        crate::removal::entries(c, &id, index, offset, &removal_policy(e)?, &e.cancel_work)
    })
}

pub fn application_leftovers(
    offset: u32,
) -> Result<Vec<crate::applications::ApplicationFootprint>> {
    with_work(|_, c| crate::applications::leftovers(c, offset))
}
pub fn application_notices() -> Result<u32> {
    crate::applications::notices(&database::open(&engine()?.path)?)
}

pub fn administrator_status() -> Result<crate::privileged::HelperStatus> {
    use crate::privileged::{HelperStatus, Operation};
    if crate::helper::protocol() != 2 {
        return Ok(HelperStatus{authorization_seconds:0,installed:std::path::Path::new("/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper").exists(),compatible:false,version:String::new(),protocol:crate::helper::protocol(),qualification:"Install the matching optional Coqui app-and-helper package. Portable builds and the health gate do not provide production administrator access.".into()});
    }
    let mut status: HelperStatus =
        serde_json::from_value(crate::helper::production(Operation::Health, false)?)?;
    status.authorization_seconds = crate::helper::authorization_remaining();
    Ok(status)
}
pub fn administrator_lock() -> Result<()> {
    with_work(|_, _| {
        crate::helper::clear_authorization();
        Ok(())
    })
}
fn administrator_preferences(e: &Engine) -> Result<crate::privileged::Preferences> {
    let c = database::open(&e.path)?;
    let s = database::settings(&c)?;
    anyhow::ensure!(
        s.administrator_enabled,
        "Enable the optional matching administrator helper in Settings first"
    );
    Ok(crate::privileged::Preferences {
        excluded_paths: s.excluded_paths,
        protected_paths: crate::applications::preferences(&c)?.protected_paths,
        retention_days: s.retention_days,
    })
}
fn wait_administrator(
    operation: crate::privileged::Operation,
    cancel: &AtomicBool,
    mut event: impl FnMut(&crate::privileged::Job) -> bool,
) -> Result<crate::privileged::Job> {
    use crate::privileged::{Job, Operation};
    anyhow::ensure!(
        !cancel.load(Ordering::SeqCst),
        "Operation cancelled before authorization"
    );
    let mut job: Job = serde_json::from_value(crate::helper::production_with_cancel(
        operation, true, cancel,
    )?)?;
    if job.status == "authorized" {
        if cancel.load(Ordering::SeqCst) {
            crate::helper::production(Operation::Cancel { id: job.id.clone() }, false)?;
            anyhow::bail!("Operation cancelled before start");
        }
        job = serde_json::from_value(crate::helper::production(
            Operation::Start { id: job.id.clone() },
            false,
        )?)?;
    }
    let mut cancelled = false;
    loop {
        if !event(&job) {
            cancel.store(true, Ordering::SeqCst);
        }
        if job.status != "running" {
            break;
        }
        if cancel.load(Ordering::SeqCst) && !cancelled {
            crate::helper::production(Operation::Cancel { id: job.id.clone() }, false)?;
            cancelled = true;
        }
        std::thread::sleep(Duration::from_millis(350));
        match crate::helper::production(Operation::Status { id: job.id.clone() }, false) {
            Ok(value) => job = serde_json::from_value(value)?,
            Err(error) => {
                let _ = crate::helper::production(Operation::Cancel { id: job.id.clone() }, false);
                return Err(error.context("Administrator transport failed; inspect retained records before another review"));
            }
        }
    }
    if let Some(error) = &job.error {
        anyhow::bail!("{error}");
    }
    Ok(job)
}
fn mirror_administrator(c: &rusqlite::Connection, p: &crate::removal::RemovalPlan) -> Result<()> {
    anyhow::ensure!(p.administrator, "Expected an administrator journal result");
    c.execute("INSERT INTO administrator_records VALUES(?1,'removal',?2,?3) ON CONFLICT(id) DO UPDATE SET json=excluded.json",rusqlite::params![p.id,p.created_at,serde_json::to_string(p)?])?;
    Ok(())
}
fn save_administrator_session(c: &rusqlite::Connection, s: &DiagnosticSession) -> Result<()> {
    let tx = c.unchecked_transaction()?;
    database::save_session(&tx, s)?;
    tx.execute(
        "INSERT INTO administrator_records VALUES(?1,'scan',?2,'{}') ON CONFLICT(id) DO NOTHING",
        rusqlite::params![s.id, s.started_at],
    )?;
    tx.commit()?;
    Ok(())
}
fn purge_administrator_mirrors(c: &rusqlite::Connection) -> Result<()> {
    let tx = c.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM sessions WHERE id IN (SELECT id FROM administrator_records WHERE kind='scan')",
        [],
    )?;
    tx.execute("DELETE FROM administrator_records", [])?;
    tx.commit()?;
    Ok(())
}
fn is_administrator_plan(c: &rusqlite::Connection, id: &str) -> Result<bool> {
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM administrator_records WHERE id=?1 AND kind='removal')",
        [id],
        |r| r.get(0),
    )?)
}
pub fn administrator_inspect_application(
    path: String,
) -> Result<crate::applications::ApplicationFootprint> {
    with_work(|e, _| {
        let preferences = administrator_preferences(e)?;
        let job = wait_administrator(
            crate::privileged::Operation::InspectApplication { path, preferences },
            &e.cancel_work,
            |_| true,
        )?;
        job.application
            .context("No administrator application result")
    })
}
pub fn administrator_preview_removal(
    request: crate::removal::RemovalRequest,
) -> Result<crate::removal::RemovalPlan> {
    with_work(|e, c| {
        let preferences = administrator_preferences(e)?;
        let job = wait_administrator(
            crate::privileged::Operation::PreviewRemoval {
                request,
                preferences,
            },
            &e.cancel_work,
            |_| true,
        )?;
        let p = job.plan.context("No administrator preview")?;
        mirror_administrator(c, &p)?;
        Ok(p)
    })
}
pub fn administrator_lifecycle(action: crate::privileged::Lifecycle) -> Result<()> {
    with_work(|e, c| {
        crate::helper::production_with_cancel(
            crate::privileged::Operation::Lifecycle { action },
            true,
            &e.cancel_work,
        )?;
        if matches!(action, crate::privileged::Lifecycle::Purge) {
            purge_administrator_mirrors(c)?;
        }
        if matches!(
            action,
            crate::privileged::Lifecycle::Stop | crate::privileged::Lifecycle::Uninstall
        ) {
            let mut s = database::settings(c)?;
            s.administrator_enabled = false;
            database::save_settings(c, &s)?;
        }
        Ok(())
    })
}
pub fn administrator_scan(
    request: ScanRequest,
    plan: DiskScanPlan,
    mut send: impl FnMut(ScanEvent) -> bool,
) -> Result<()> {
    use crate::privileged::Operation;
    let e = engine()?;
    anyhow::ensure!(
        e.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "Wait for the active operation"
    );
    let _guard = ActiveGuard(&e.active);
    e.cancel.store(false, Ordering::SeqCst);
    let preferences = administrator_preferences(&e)?;
    let mut c = database::open(&e.path)?;
    let mut latest = None;
    let job = wait_administrator(
        Operation::Scan {
            request,
            plan,
            preferences,
        },
        &e.cancel,
        |j| {
            let Some(session) = &j.session else {
                return true;
            };
            let mut s = session.clone();
            if s.status != "running" {
                s.status = "running".into();
                s.stage = "Importing administrator scan results".into();
            }
            if save_administrator_session(&c, &s).is_err() {
                return false;
            }
            latest = Some(session.clone());
            // A bounded current page supplies real partial results while traversal continues.
            let q = StorageQuery {
                session_id: s.id.clone(),
                parent: None,
                category: None,
                search: None,
                sort: "size".into(),
                descending: true,
                offset: 0,
                limit: 100,
                files_only: false,
            };
            if let Ok(value) = crate::helper::production(Operation::Files { query: q }, false) {
                if let Ok(files) = serde_json::from_value::<Vec<FileEntry>>(value) {
                    let _ = database::insert_files(&mut c, &s.id, &files);
                }
            }
            send(ScanEvent { session: s })
        },
    );
    let job = match job {
        Ok(j) => j,
        Err(error) => {
            if let Some(mut s) = latest {
                s.status = "partial".into();
                s.stage = "Administrator scan interrupted".into();
                s.finished_at = Some(now());
                s.errors.push(format!("{error:#}"));
                save_administrator_session(&c, &s)?;
                let _ = send(ScanEvent { session: s });
            }
            return Err(error);
        }
    };
    let mut s = job
        .session
        .or(latest)
        .context("Administrator scan returned no session")?;
    // Cancellation covers collection and both import loops. Retain bounded mirrored
    // context, but never turn one cancel into an uninterruptible second workload.
    let mut imported_files = 0u32;
    let import = (|| -> Result<()> {
        let mut after = String::new();
        while !e.cancel.load(Ordering::SeqCst) {
            let entries: Vec<IndexedStorageEntry> =
                serde_json::from_value(crate::helper::production(
                    Operation::ExportStorage {
                        id: s.id.clone(),
                        after: after.clone(),
                    },
                    false,
                )?)?;
            if entries.is_empty() {
                break;
            }
            imported_files += entries.iter().filter(|f| !f.entry.is_directory).count() as u32;
            after = entries.last().unwrap().entry.path.clone();
            database::insert_indexed_files(&mut c, &s.id, &entries)?;
            let mut progress = s.clone();
            progress.status = "running".into();
            progress.stage = format!(
                "Saving administrator metadata: {imported_files} of {} files",
                s.file_count
            );
            if !send(ScanEvent { session: progress }) {
                e.cancel.store(true, Ordering::SeqCst);
            }
        }
        if e.cancel.load(Ordering::SeqCst) {
            return Ok(());
        }
        if imported_files != s.file_count {
            s.status = "partial".into();
            s.errors.push("The helper index changed or was unavailable during import. Retained imported results are partial; review scope and retry.".into());
        }
        c.execute("DELETE FROM scan_gaps WHERE session_id=?1", [&s.id])?;
        let mut after = 0;
        while !e.cancel.load(Ordering::SeqCst) {
            let page: database::CoverageExport =
                serde_json::from_value(crate::helper::production(
                    Operation::ExportCoverage {
                        id: s.id.clone(),
                        after,
                    },
                    false,
                )?)?;
            if page.gaps.is_empty() {
                break;
            }
            after = page.after;
            database::insert_gaps(&mut c, &s.id, &page.gaps)?;
            let mut progress = s.clone();
            progress.status = "running".into();
            progress.stage = "Saving administrator coverage evidence".into();
            if !send(ScanEvent { session: progress }) {
                e.cancel.store(true, Ordering::SeqCst);
            }
        }
        Ok(())
    })();
    if e.cancel.load(Ordering::SeqCst) {
        s.status = "cancelled".into();
        s.stage = "Administrator scan/import cancelled".into();
        s.finished_at = Some(now());
        s.errors.push("Local index import is incomplete after cancellation; saved coverage and bytes describe collection, while local folder/file results may contain only a subset. The helper retains its index until retention or explicit purge.".into());
    }
    if let Err(error) = import {
        s.status = "partial".into();
        s.stage = "Administrator index import interrupted".into();
        s.finished_at = Some(now());
        s.errors.push(format!("Administrator result import failed: {error}. Local file and coverage results contain only the imported subset; the helper retains its index."));
    }
    s.stage = format!(
        "{} • administrator metadata; TCC/SIP access gaps remain qualified",
        s.stage
    );
    save_administrator_session(&c, &s)?;
    if let Err(error) =
        database::retain_after_scan(&c, database::settings(&c)?.retention_days, &e.cancel)
    {
        s.errors.push(format!(
            "Administrator results were saved, but older index retention did not finish: {}.",
            scan_error(&error)
        ));
        save_administrator_session(&c, &s)?;
    }
    let _ = send(ScanEvent { session: s });
    Ok(())
}
