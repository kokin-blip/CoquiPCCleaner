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

pub fn init_engine(data_directory: String) -> Result<AppSettings> {
    let dir = PathBuf::from(data_directory);
    anyhow::ensure!(
        dir.is_absolute(),
        "Application data directory must be absolute"
    );
    std::fs::create_dir_all(&dir)?;
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
    let path = dir.join("coqui.sqlite3");
    let c = database::open(&path)?;
    database::recover_interrupted(&c)?;
    let settings = database::settings(&c)?;
    database::retain(&c, settings.retention_days)?;
    *guard = Some(Arc::new(Engine {
        path,
        active: AtomicBool::new(false),
        cancel: AtomicBool::new(false),
        monitoring: AtomicBool::new(false),
        stop_monitor: AtomicBool::new(false),
        cancel_cleanup: AtomicBool::new(false),
        cancel_developer: AtomicBool::new(false),
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
    database::save_settings(&c, &settings)?;
    database::retain(&c, settings.retention_days)
}
pub fn scan_history(offset: u32, limit: u32) -> Result<Vec<DiagnosticSession>> {
    database::history(&database::open(&engine()?.path)?, offset, limit)
}
pub fn get_session(id: String) -> Result<DiagnosticSession> {
    database::session(&database::open(&engine()?.path)?, &id)
}
pub fn storage_entries(query: StorageQuery) -> Result<Vec<FileEntry>> {
    database::files(&database::open(&engine()?.path)?, &query)
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
    let (roots, root_errors) = storage::roots(&request)?;
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
    let mut c = database::open(&e.path)?;
    let mut s = DiagnosticSession {
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
        let summary = storage::walk(
            &roots,
            &excluded,
            &e.cancel,
            |f| {
                let mut b = batch.borrow_mut();
                b.push(f);
                if b.len() >= 250 {
                    database::insert_files(&mut connection.borrow_mut(), &id, &b)?;
                    b.clear();
                }
                Ok(())
            },
            |w| {
                if !batch.borrow().is_empty() {
                    database::insert_files(&mut connection.borrow_mut(), &id, &batch.borrow())?;
                    batch.borrow_mut().clear();
                }
                let mut s = session.borrow_mut();
                s.file_count = w.files;
                s.logical_bytes = w.logical;
                s.allocated_bytes = w.allocated;
                s.skipped_count = w.skipped;
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
        )?;
        s.file_count = summary.files;
        s.logical_bytes = summary.logical;
        s.allocated_bytes = summary.allocated;
        s.skipped_count = summary.skipped;
        s.permissions = summary.permissions;
        s.errors.extend(summary.errors);
        s.categories = summary.categories;
        s.candidates = summary.candidates;
        Ok(())
    })();
    if result.is_err() {
        s.errors.push("Scan stopped because results could not be collected or persisted. Retry after checking available application storage.".into());
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
    let settings = database::settings(&c)?;
    database::retain(&c, settings.retention_days)?;
    let _ = send(ScanEvent { session: s });
    Ok(())
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
