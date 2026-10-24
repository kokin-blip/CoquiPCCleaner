//! Production helper protocol: independently journaled, per-user and per-operation authorized.
use crate::{
    applications::ApplicationFootprint,
    database,
    model::*,
    removal::{self, RemovalApproval, RemovalPlan, RemovalRequest},
};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
};
const STATE: &str = "/Library/Application Support/CoquiPCCleaner/Helper/State";
const SELF_APP: &str = "/Applications/CoquiPCCleaner.app";
static ACTIVE: AtomicBool = AtomicBool::new(false);
static CONTROL: Mutex<()> = Mutex::new(());
static STOPPING: AtomicBool = AtomicBool::new(false);
struct ActiveJob {
    uid: u32,
    id: String,
    pid: i32,
    start: u64,
    cancel: Arc<AtomicBool>,
    seen: Arc<AtomicU64>,
    ack: Arc<AtomicU64>,
}
static CANCEL: OnceLock<Mutex<Option<ActiveJob>>> = OnceLock::new();
fn tick() -> u64 {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as u64
}
static RECOVERED: OnceLock<Mutex<std::collections::HashSet<u32>>> = OnceLock::new();
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub excluded_paths: Vec<String>,
    pub protected_paths: Vec<String>,
    pub retention_days: u32,
}
#[derive(Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct HelperStatus {
    #[serde(default)]
    pub authorization_seconds: u32,
    pub installed: bool,
    pub compatible: bool,
    pub version: String,
    pub protocol: u32,
    pub qualification: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum Operation {
    Health,
    SetPreferences {
        preferences: Preferences,
    },
    Challenge {
        request: Box<Operation>,
    },
    InspectApplication {
        path: String,
        preferences: Preferences,
    },
    PreviewRemoval {
        request: RemovalRequest,
        preferences: Preferences,
    },
    PreviewRestore {
        id: String,
        paths: Vec<String>,
        preferences: Preferences,
    },
    Execute {
        approval: RemovalApproval,
        preferences: Preferences,
    },
    Scan {
        request: ScanRequest,
        plan: DiskScanPlan,
        preferences: Preferences,
    },
    Start {
        id: String,
    },
    Status {
        id: String,
    },
    Files {
        query: StorageQuery,
    },
    StorageMap {
        query: StorageMapQuery,
    },
    ExportFiles {
        id: String,
        after: String,
    },
    ExportStorage {
        id: String,
        after: String,
    },
    Coverage {
        id: String,
        offset: u32,
    },
    ExportCoverage {
        id: String,
        after: u64,
    },
    Entries {
        id: String,
        index: u32,
        offset: u32,
    },
    History {
        offset: u32,
    },
    Cancel {
        id: String,
    },
    Lifecycle {
        action: Lifecycle,
    },
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Lifecycle {
    Purge,
    Stop,
    Uninstall,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub protocol: u32,
    pub version: String,
    pub request: Operation,
    #[serde(default)]
    pub challenge: Option<String>,
    #[serde(default)]
    pub authorization: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    #[serde(default)]
    pub revision: u64,
    pub id: String,
    pub status: String,
    pub kind: String,
    pub session: Option<DiagnosticSession>,
    pub plan: Option<RemovalPlan>,
    pub application: Option<ApplicationFootprint>,
    pub error: Option<String>,
}
impl Operation {
    fn authorized(&self) -> bool {
        matches!(
            self,
            Self::InspectApplication { .. }
                | Self::PreviewRemoval { .. }
                | Self::PreviewRestore { .. }
                | Self::Execute { .. }
                | Self::Scan { .. }
                | Self::Lifecycle { .. }
        )
    }
}
fn validate_preferences(p: &Preferences) -> Result<()> {
    anyhow::ensure!(
        (1..=3650).contains(&p.retention_days)
            && p.excluded_paths.len() + p.protected_paths.len() <= 2000,
        "Invalid helper preferences"
    );
    for text in p.excluded_paths.iter().chain(&p.protected_paths) {
        let path = Path::new(text);
        anyhow::ensure!(
            path.is_absolute()
                && text.len() <= 4096
                && !path.components().any(|c| matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )),
            "Invalid exclusion/protection path"
        );
    }
    Ok(())
}
pub fn account(uid: u32) -> Result<(PathBuf, u32)> {
    anyhow::ensure!(uid >= 501, "Interactive requesting user required");
    let mut record: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result = std::ptr::null_mut();
    let mut bytes = vec![0u8; 65536];
    anyhow::ensure!(
        unsafe {
            libc::getpwuid_r(
                uid,
                &mut record,
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                &mut result,
            )
        } == 0
            && !result.is_null(),
        "Requesting account unavailable"
    );
    let home = PathBuf::from(unsafe { std::ffi::CStr::from_ptr(record.pw_dir) }.to_str()?);
    anyhow::ensure!(
        home.is_absolute() && home != Path::new("/var/root") && home != Path::new("/"),
        "Invalid requesting home"
    );
    let m = std::fs::symlink_metadata(&home)?;
    anyhow::ensure!(
        m.is_dir() && !m.file_type().is_symlink() && m.uid() == uid,
        "Requesting home ownership changed"
    );
    Ok((home, record.pw_gid))
}
fn policy(uid: u32, p: &Preferences) -> Result<removal::Policy> {
    validate_preferences(p)?;
    let (home, _) = account(uid)?;
    let mut exclusions = p
        .excluded_paths
        .iter()
        .chain(&p.protected_paths)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    exclusions.push(home.join("Library/Application Support/CoquiPCCleaner"));
    exclusions.push(PathBuf::from("/Library/Application Support/CoquiPCCleaner"));
    Ok(removal::Policy {
        home,
        exclusions,
        self_app: Some(SELF_APP.into()),
        administrator_uid: Some(uid),
    })
}
fn protected_directory(path: &Path) -> Result<()> {
    let m = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        m.is_dir() && !m.file_type().is_symlink() && m.uid() == 0 && m.mode() & 0o022 == 0,
        "Helper state directory ownership or permissions invalid"
    );
    #[cfg(target_os = "macos")]
    {
        let _ = crate::safe_fs::directory(path)?;
    }
    Ok(())
}
fn connection(uid: u32) -> Result<Connection> {
    anyhow::ensure!(
        unsafe { libc::geteuid() } == 0,
        "Helper state requires root installation"
    );
    protected_directory(Path::new(STATE).parent().unwrap())?;
    for dir in [
        PathBuf::from(STATE),
        PathBuf::from(STATE).join(uid.to_string()),
    ] {
        if !dir.exists() {
            std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
        }
        protected_directory(&dir)?;
    }
    let file = PathBuf::from(STATE)
        .join(uid.to_string())
        .join("diagnostics.sqlite3");
    if !file.exists() {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&file)?;
    }
    let m = std::fs::symlink_metadata(&file)?;
    anyhow::ensure!(
        m.is_file()
            && !m.file_type().is_symlink()
            && m.uid() == 0
            && m.nlink() == 1
            && m.mode() & 0o077 == 0,
        "Helper database ownership or permissions invalid"
    );
    let c = database::open(&file)?;
    c.execute_batch("PRAGMA synchronous=FULL;")?;
    let mut recovered = RECOVERED
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| anyhow::anyhow!("Helper recovery lock unavailable"))?;
    if !recovered.contains(&uid) {
        initialize(&c)?;
        database::recover_interrupted(&c)?;
        c.execute("UPDATE helper_jobs SET status='interrupted',json=json_set(json,'$.status','interrupted','$.error','Helper restarted; inspect retained results, no automatic retry') WHERE status IN ('running','authorized')",[])?;
        c.execute("DELETE FROM helper_pending", [])?;
        expire_requests(&c)?;
        recovered.insert(uid);
    }
    // Status, file pages and map requests are pure reads after first connection.
    // They must remain available while the scan writer finalizes or retains an index.
    Ok(c)
}
fn expire_requests(c: &Connection) -> Result<()> {
    let pending_cutoff = (chrono::Utc::now() - chrono::Duration::minutes(2)).to_rfc3339();
    c.execute("UPDATE helper_jobs SET status='expired',json=json_set(json,'$.status','expired','$.error','Prepared authorization expired; obtain a new review') WHERE id IN (SELECT id FROM helper_pending WHERE created_at<?1)",[&pending_cutoff])?;
    c.execute(
        "DELETE FROM helper_pending WHERE created_at<?1",
        [&pending_cutoff],
    )?;
    c.execute(
        "DELETE FROM helper_challenges WHERE expires_at < ?1",
        [crate::model::now()],
    )?;
    Ok(())
}
fn initialize(c: &Connection) -> Result<()> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS helper_pending(id TEXT PRIMARY KEY,pid INTEGER NOT NULL,start INTEGER NOT NULL,created_at TEXT NOT NULL,json TEXT NOT NULL);CREATE TABLE IF NOT EXISTS helper_challenges(id TEXT PRIMARY KEY,uid INTEGER NOT NULL,digest TEXT NOT NULL,expires_at TEXT NOT NULL);CREATE TABLE IF NOT EXISTS helper_jobs(id TEXT PRIMARY KEY,created_at TEXT NOT NULL,status TEXT NOT NULL,json TEXT NOT NULL);CREATE TABLE IF NOT EXISTS helper_policy(id TEXT PRIMARY KEY,json TEXT NOT NULL);")?;
    Ok(())
}
fn digest(op: &Operation) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(op)?)))
}
fn challenge(c: &Connection, uid: u32, op: &Operation) -> Result<String> {
    anyhow::ensure!(
        op.authorized(),
        "This operation does not accept an authorization challenge"
    );
    expire_requests(c)?;
    let count: u32 = c.query_row("SELECT count(*) FROM helper_challenges", [], |r| r.get(0))?;
    anyhow::ensure!(
        count < 100,
        "Too many pending authorization requests; wait for expiry"
    );
    let id = uuid::Uuid::new_v4().to_string();
    let expires = (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339();
    c.execute(
        "INSERT INTO helper_challenges VALUES(?1,?2,?3,?4)",
        params![id, uid, digest(op)?, expires],
    )?;
    Ok(id)
}
fn consume(c: &Connection, uid: u32, id: &str, op: &Operation) -> Result<()> {
    let found:Option<String>=c.query_row("DELETE FROM helper_challenges WHERE id=?1 AND uid=?2 AND expires_at>?3 RETURNING digest",params![id,uid,crate::model::now()],|r|r.get(0)).optional()?;
    anyhow::ensure!(
        found.as_deref() == Some(digest(op)?.as_str()),
        "Authorization challenge expired, mismatched or already consumed"
    );
    Ok(())
}
fn authorize(c: &Connection, uid: u32, e: &Envelope) -> Result<()> {
    consume(
        c,
        uid,
        e.challenge
            .as_deref()
            .context("Fresh per-operation authorization required")?,
        &e.request,
    )?;
    let form = std::ffi::CString::new(
        e.authorization
            .as_deref()
            .context("Administrator authorization required")?,
    )?;
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn coqui_authorization_valid(form: *const libc::c_char) -> i32;
        }
        anyhow::ensure!(
            unsafe { coqui_authorization_valid(form.as_ptr()) } == 1,
            "Administrator authorization rejected"
        );
    }
    #[cfg(not(target_os = "macos"))]
    anyhow::bail!("Administrator helper requires macOS");
    Ok(())
}
fn save(c: &Connection, j: &Job) -> Result<()> {
    c.execute("INSERT INTO helper_jobs VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET status=excluded.status,json=excluded.json",params![j.id,crate::model::now(),j.status,serde_json::to_string(j)?])?;
    Ok(())
}
fn job(c: &Connection, id: &str) -> Result<Job> {
    Ok(serde_json::from_str(&c.query_row(
        "SELECT json FROM helper_jobs WHERE id=?1",
        [id],
        |r| r.get::<_, String>(0),
    )?)?)
}
fn save_policy(c: &Connection, id: &str, p: &Preferences) -> Result<()> {
    c.execute(
        "INSERT INTO helper_policy VALUES(?1,?2)",
        params![id, serde_json::to_string(p)?],
    )?;
    Ok(())
}
fn set_current_preferences(c: &Connection, p: &Preferences) -> Result<()> {
    validate_preferences(p)?;
    c.execute("INSERT INTO helper_policy VALUES('$current',?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(p)?])?;
    let mut settings = database::settings(c)?;
    settings.retention_days = p.retention_days;
    database::save_settings(c, &settings)?;
    database::retain(c, p.retention_days)?;
    c.execute("DELETE FROM helper_policy WHERE id!='$current' AND id NOT IN (SELECT id FROM removal_plans)", [])?;
    let cutoff =
        (chrono::Utc::now() - chrono::Duration::days(p.retention_days.into())).to_rfc3339();
    c.execute(
        "DELETE FROM helper_jobs WHERE created_at<?1 AND status!='running'",
        [cutoff],
    )?;
    Ok(())
}
fn check_policy(c: &Connection, id: &str, p: &Preferences) -> Result<()> {
    validate_preferences(p)?;
    let current: String = c.query_row(
        "SELECT json FROM helper_policy WHERE id='$current'",
        [],
        |r| r.get(0),
    )?;
    anyhow::ensure!(
        serde_json::from_str::<Preferences>(&current)? == *p,
        "Helper preferences changed; obtain a new review"
    );
    let old: Preferences = serde_json::from_str(&c.query_row(
        "SELECT json FROM helper_policy WHERE id=?1",
        [id],
        |r| r.get::<_, String>(0),
    )?)?;
    anyhow::ensure!(
        &old == p,
        "Exclusions, protections or retention changed; obtain a new review"
    );
    Ok(())
}
fn check_job_preferences(c: &Connection, operation: &Operation) -> Result<()> {
    let p = match operation {
        Operation::InspectApplication { preferences, .. }
        | Operation::PreviewRemoval { preferences, .. }
        | Operation::PreviewRestore { preferences, .. }
        | Operation::Scan { preferences, .. } => preferences,
        _ => return Ok(()),
    };
    let current: String = c.query_row(
        "SELECT json FROM helper_policy WHERE id='$current'",
        [],
        |r| r.get(0),
    )?;
    anyhow::ensure!(
        serde_json::from_str::<Preferences>(&current)? == *p,
        "Helper preferences changed; refresh Settings and obtain a new review"
    );
    Ok(())
}
fn process_start(pid: i32) -> u64 {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn coqui_process_start(pid: i32) -> u64;
        }
        unsafe { coqui_process_start(pid) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        0
    }
}
fn prepare(c: &Connection, pid: i32, op: &Operation) -> Result<Job> {
    check_job_preferences(c, op)?;
    let count: u32 = c.query_row("SELECT count(*) FROM helper_pending", [], |r| r.get(0))?;
    anyhow::ensure!(count < 100, "Too many pending authorized operations");
    let id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO helper_pending VALUES(?1,?2,?3,?4,?5)",
        params![
            id,
            pid,
            process_start(pid),
            crate::model::now(),
            serde_json::to_string(op)?
        ],
    )?;
    let j = Job {
        revision: 0,
        id,
        status: "authorized".into(),
        kind: "prepared".into(),
        session: None,
        plan: None,
        application: None,
        error: None,
    };
    save(c, &j)?;
    Ok(j)
}
fn start_prepared(c: &Connection, uid: u32, pid: i32, id: &str) -> Result<Job> {
    let _control = CONTROL
        .lock()
        .map_err(|_| anyhow::anyhow!("Helper lifecycle lock unavailable"))?;
    let record: Option<(i32, u64, String)> = c
        .query_row(
            "DELETE FROM helper_pending WHERE id=?1 AND created_at>?2 RETURNING pid,start,json",
            params![
                id,
                (chrono::Utc::now() - chrono::Duration::minutes(2)).to_rfc3339()
            ],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (owner, start, operation) =
        record.context("Prepared operation expired or was already started")?;
    anyhow::ensure!(
        owner == pid && start != 0 && process_start(pid) == start,
        "Only the original authenticated process can start this authorization"
    );
    let operation = serde_json::from_str(&operation)?;
    check_job_preferences(c, &operation)?;
    begin(uid, pid, operation, id.into())
}
fn wait_for_consumer(uid: u32, id: &str, revision: u64, cancel: &AtomicBool) -> bool {
    let ack = {
        let Ok(guard) = CANCEL.get_or_init(Default::default).lock() else {
            return false;
        };
        let Some(active) = guard.as_ref().filter(|a| a.uid == uid && a.id == id) else {
            return false;
        };
        active.ack.clone()
    };
    let started = std::time::Instant::now();
    while ack.load(Ordering::SeqCst) < revision && !cancel.load(Ordering::SeqCst) {
        if started.elapsed() > std::time::Duration::from_secs(12) {
            cancel.store(true, Ordering::SeqCst);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    !cancel.load(Ordering::SeqCst)
}
fn begin(uid: u32, pid: i32, op: Operation, id: String) -> Result<Job> {
    anyhow::ensure!(
        !STOPPING.load(Ordering::SeqCst),
        "Helper is stopping; retry explicitly after restart"
    );
    anyhow::ensure!(
        ACTIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok(),
        "The administrator helper is busy; finish or cancel its current operation"
    );
    struct Release;
    impl Drop for Release {
        fn drop(&mut self) {
            ACTIVE.store(false, Ordering::SeqCst);
        }
    }
    let release = Release;
    let c = connection(uid)?;
    let j = Job {
        revision: 0,
        id: id.clone(),
        status: "running".into(),
        kind: match &op {
            Operation::Scan { .. } => "scan",
            Operation::InspectApplication { .. } => "application",
            Operation::Execute { .. } => "execute",
            _ => "preview",
        }
        .into(),
        session: None,
        plan: None,
        application: None,
        error: None,
    };
    save(&c, &j)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let seen = Arc::new(AtomicU64::new(tick()));
    *CANCEL
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| anyhow::anyhow!("Helper cancellation unavailable"))? = Some(ActiveJob {
        uid,
        id: id.clone(),
        pid,
        start: process_start(pid),
        cancel: cancel.clone(),
        seen: seen.clone(),
        ack: Arc::new(AtomicU64::new(0)),
    });
    let done = Arc::new(AtomicBool::new(false));
    let watch_done = done.clone();
    let watch_cancel = cancel.clone();
    let watch_id = id.clone();
    let start = process_start(pid);
    std::thread::spawn(move || {
        while !watch_done.load(Ordering::SeqCst) {
            if start == 0
                || process_start(pid) != start
                || tick().saturating_sub(seen.load(Ordering::SeqCst)) > 10_000
            {
                watch_cancel.store(true, Ordering::SeqCst);
                if let Ok(guard) = CANCEL.get_or_init(Default::default).lock() {
                    if !watch_done.load(Ordering::SeqCst)
                        && guard
                            .as_ref()
                            .is_some_and(|a| a.uid == uid && a.id == watch_id)
                    {
                        let _ = crate::api::cancel_scan();
                    }
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    });
    let mut working = j.clone();
    std::thread::spawn(move || {
        let _release = release;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run(uid, op, &cancel, &mut working)
        }));
        if let Err(error) =
            result.unwrap_or_else(|_| Err(anyhow::anyhow!("Helper operation interrupted")))
        {
            working.error = Some(format!("{error:#}"));
            working.status = if cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "failed"
            }
            .into();
        } else {
            working.status = if cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "complete"
            }
            .into();
        }
        if let Ok(c) = connection(uid) {
            let _ = save(&c, &working);
        }
        done.store(true, Ordering::SeqCst);
    });
    Ok(j)
}
fn run(uid: u32, op: Operation, cancel: &AtomicBool, j: &mut Job) -> Result<()> {
    let c = connection(uid)?;
    check_job_preferences(&c, &op)?;
    match op {
        Operation::InspectApplication { path, preferences } => {
            let p = policy(uid, &preferences)?;
            let path = Path::new(&path);
            anyhow::ensure!(
                path.starts_with("/Applications")
                    || path.starts_with(&p.home)
                    || path.starts_with("/Volumes")
                    || path.starts_with("/System/Applications"),
                "This app location is outside supported inspection scope"
            );
            j.application = Some(crate::applications::qualify_related(
                crate::applications::inspect(path, &p)?,
                &p,
            ));
        }
        Operation::PreviewRemoval {
            request,
            preferences,
        } => {
            let plan = removal::preview(&c, request, &policy(uid, &preferences)?, cancel)?;
            save_policy(&c, &plan.id, &preferences)?;
            j.plan = Some(plan);
        }
        Operation::PreviewRestore {
            id,
            paths,
            preferences,
        } => {
            let plan =
                removal::preview_restore(&c, &id, &paths, &policy(uid, &preferences)?, cancel)?;
            save_policy(&c, &plan.id, &preferences)?;
            j.plan = Some(plan);
        }
        Operation::Execute {
            approval,
            preferences,
        } => {
            check_policy(&c, &approval.id, &preferences)?;
            let p = policy(uid, &preferences)?;
            j.plan = Some(removal::native_execute(&c, approval, &p, cancel, |plan| {
                j.plan = Some(plan);
                j.revision += 1;
                save(&c, j).is_ok() && wait_for_consumer(uid, &j.id, j.revision, cancel)
            })?);
        }
        Operation::Scan {
            mut request,
            plan,
            preferences,
        } => {
            validate_preferences(&preferences)?;
            anyhow::ensure!(
                request.mode == "deep",
                "Administrator scans require reviewed whole-internal scope"
            );
            c.execute(
                "INSERT OR REPLACE INTO scan_plans VALUES(?1,?2,?3)",
                params![plan.id, plan.created_at, serde_json::to_string(&plan)?],
            )?;
            request.plan_id = Some(plan.id.clone());
            crate::volumes::approved_roots(&c, &request)?;
            request.excluded_paths = preferences.excluded_paths.clone();
            request.excluded_paths.push(STATE.into());
            request.excluded_paths.push(
                account(uid)?
                    .0
                    .join("Library/Application Support/CoquiPCCleaner")
                    .to_string_lossy()
                    .into_owned(),
            );
            let data = PathBuf::from(STATE).join(uid.to_string());
            let mut settings = crate::api::init_engine_database_for_scan(
                data.join("diagnostics.sqlite3"),
                cancel,
            )?;
            anyhow::ensure!(
                !cancel.load(Ordering::SeqCst),
                "Scan cancelled before collection"
            );
            settings.retention_days = preferences.retention_days;
            // The helper already validated/saved policy; avoid a second uncancellable
            // retention pass through the general GUI preference operation.
            database::save_settings(&c, &settings)?;
            crate::api::scan(request, |event| {
                j.session = Some(event.session);
                save(&c, j).is_ok() && !cancel.load(Ordering::SeqCst)
            })?;
        }
        _ => anyhow::bail!("This operation is not an executable helper job"),
    }
    Ok(())
}
pub fn dispatch(text: &str, uid: u32, pid: i32) -> Result<serde_json::Value> {
    anyhow::ensure!(uid >= 501, "Interactive requesting user required");
    let e: Envelope = serde_json::from_str(text)?;
    anyhow::ensure!(
        e.protocol == 2 && e.version == env!("CARGO_PKG_VERSION"),
        "Helper protocol/build version mismatch; install matching app and helper"
    );
    if matches!(e.request, Operation::Health) {
        return Ok(
            serde_json::json!({"protocol":2,"version":env!("CARGO_PKG_VERSION"),"result":HelperStatus{authorization_seconds:0,installed:true,compatible:true,version:env!("CARGO_PKG_VERSION").into(),protocol:2,qualification:"Authenticated matching installed build. Elevated operations require separate authorization; TCC/SIP protections still apply.".into()}}),
        );
    }
    // Renew a validated client's lease and accept active cancellation before any
    // database work. A busy journal must not prevent the user from stopping a scan.
    match &e.request {
        Operation::Status { id } => {
            signal_active(uid, pid, id, false)?;
        }
        Operation::Cancel { id } if signal_active(uid, pid, id, true)? => {
            return Ok(
                serde_json::json!({"protocol":2,"version":env!("CARGO_PKG_VERSION"),"result":true}),
            );
        }
        _ => {}
    }
    let c = connection(uid)?;
    if e.request.authorized() {
        authorize(&c, uid, &e)?;
    }
    let result = match e.request {
        Operation::Health => unreachable!(),
        Operation::SetPreferences { preferences } => {
            let _control = CONTROL
                .lock()
                .map_err(|_| anyhow::anyhow!("Helper lifecycle lock unavailable"))?;
            anyhow::ensure!(
                !ACTIVE.load(Ordering::SeqCst),
                "Finish or cancel helper work before changing preferences"
            );
            set_current_preferences(&c, &preferences)?;
            serde_json::json!(true)
        }
        Operation::Challenge { request } => serde_json::json!(challenge(&c, uid, &request)?),
        Operation::Start { id } => serde_json::to_value(start_prepared(&c, uid, pid, &id)?)?,
        Operation::Status { id } => {
            let j = job(&c, &id)?;
            if let Some(active) = CANCEL
                .get_or_init(Default::default)
                .lock()
                .map_err(|_| anyhow::anyhow!("Helper lease unavailable"))?
                .as_ref()
                .filter(|a| {
                    a.uid == uid && a.id == id && a.pid == pid && process_start(pid) == a.start
                })
            {
                active.seen.store(tick(), Ordering::SeqCst);
                active.ack.fetch_max(j.revision, Ordering::SeqCst);
            }
            serde_json::to_value(j)?
        }
        Operation::History { offset } => serde_json::to_value(removal::history(&c, offset)?)?,
        Operation::Files { query } => serde_json::to_value(database::files(&c, &query)?)?,
        Operation::StorageMap { query } => {
            serde_json::to_value(database::storage_map(&c, &query)?)?
        }
        Operation::ExportFiles { id, after } => {
            serde_json::to_value(database::files_after(&c, &id, &after)?)?
        }
        Operation::ExportStorage { id, after } => {
            serde_json::to_value(database::indexed_files_after(&c, &id, &after)?)?
        }
        Operation::Coverage { id, offset } => {
            serde_json::to_value(database::coverage(&c, &id, offset)?)?
        }
        Operation::ExportCoverage { id, after } => {
            serde_json::to_value(database::gaps_after(&c, &id, after)?)?
        }
        Operation::Entries { id, index, offset } => {
            let preferences: Preferences = serde_json::from_str(&c.query_row(
                "SELECT json FROM helper_policy WHERE id=?1",
                [&id],
                |r| r.get::<_, String>(0),
            )?)?;
            check_policy(&c, &id, &preferences)?;
            serde_json::to_value(removal::entries(
                &c,
                &id,
                index,
                offset,
                &policy(uid, &preferences)?,
                &AtomicBool::new(false),
            )?)?
        }
        Operation::Cancel { id } => {
            if c.execute(
                "DELETE FROM helper_pending WHERE id=?1 AND pid=?2 AND start=?3",
                params![id, pid, process_start(pid)],
            )? > 0
            {
                let mut j = job(&c, &id)?;
                j.status = "cancelled".into();
                save(&c, &j)?;
            }
            serde_json::json!(true)
        }
        Operation::Lifecycle { action } => {
            let _control = CONTROL
                .lock()
                .map_err(|_| anyhow::anyhow!("Helper lifecycle lock unavailable"))?;
            anyhow::ensure!(
                !ACTIVE.load(Ordering::SeqCst),
                "Cancel active helper work before lifecycle changes"
            );
            match action {
                Lifecycle::Purge => {
                    database::purge(&c)?;
                    c.execute_batch("DELETE FROM helper_jobs;DELETE FROM helper_policy;DELETE FROM helper_challenges;DELETE FROM helper_pending;")?;
                }
                Lifecycle::Stop => schedule_stop(false)?,
                Lifecycle::Uninstall => schedule_stop(true)?,
            };
            serde_json::json!(true)
        }
        operation => serde_json::to_value(prepare(&c, pid, &operation)?)?,
    };
    Ok(serde_json::json!({"protocol":2,"version":env!("CARGO_PKG_VERSION"),"result":result}))
}

fn signal_active(uid: u32, pid: i32, id: &str, cancel: bool) -> Result<bool> {
    let guard = CANCEL
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| anyhow::anyhow!("Helper control unavailable"))?;
    let Some(active) = guard.as_ref().filter(|a| a.id == id) else {
        return Ok(false);
    };
    signal_record(active, uid, pid, process_start(pid), cancel)?;
    if cancel {
        let _ = crate::api::cancel_scan();
    }
    Ok(true)
}

fn signal_record(active: &ActiveJob, uid: u32, pid: i32, start: u64, cancel: bool) -> Result<()> {
    anyhow::ensure!(
        active.uid == uid && active.pid == pid && active.start != 0 && start == active.start,
        "Only the original authenticated process can control this operation"
    );
    active.seen.store(tick(), Ordering::SeqCst);
    if cancel {
        active.cancel.store(true, Ordering::SeqCst);
    }
    Ok(())
}
fn schedule_stop(uninstall: bool) -> Result<()> {
    STOPPING.store(true, Ordering::SeqCst);
    if uninstall {
        for path in [
            "/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist",
            "/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper",
            "/Library/Application Support/CoquiPCCleaner/Helper/pairing.json",
        ] {
            let m = std::fs::symlink_metadata(path)?;
            anyhow::ensure!(
                m.is_file() && !m.file_type().is_symlink() && m.uid() == 0,
                "Helper lifecycle target ownership changed"
            );
        }
        for path in [
            "/Library/LaunchDaemons/com.coqui.CoquiPCCleaner.Helper.plist",
            "/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper",
            "/Library/Application Support/CoquiPCCleaner/Helper/pairing.json",
        ] {
            std::fs::remove_file(path)?;
        }
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        if uninstall {
            let _ = std::process::Command::new("/bin/launchctl")
                .args(["bootout", "system/com.coqui.CoquiPCCleaner.Helper"])
                .status();
        }
        std::process::exit(0);
    });
    Ok(())
}
pub fn user_trash_directory(path: &Path, uid: u32) -> Result<PathBuf> {
    anyhow::ensure!(
        unsafe { libc::geteuid() } == 0,
        "Administrator Trash resolution requires installed root helper"
    );
    let helper = std::env::current_exe()?;
    anyhow::ensure!(
        helper == Path::new("/Library/PrivilegedHelperTools/com.coqui.CoquiPCCleaner.Helper"),
        "Trash resolution requires the installed helper"
    );
    let (home, _) = account(uid)?;
    let mut command = std::process::Command::new(helper);
    command.arg("--user-trash-directory").env("HOME", &home);
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .context("User Trash pipe unavailable")?
        .write_all(serde_json::to_string(&(uid, path))?.as_bytes())?;
    let start = std::time::Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("User Trash resolution timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let output = child.wait_with_output()?;
    anyhow::ensure!(
        output.status.success() && output.stdout.len() < 16384,
        "User Trash directory unavailable"
    );
    let trash: PathBuf = serde_json::from_slice(&output.stdout)?;
    let (home, _) = account(uid)?;
    let personal = trash == home.join(".Trash");
    let volume = trash
        .file_name()
        .is_some_and(|n| n == uid.to_string().as_str())
        && trash
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|n| n == ".Trashes");
    anyhow::ensure!(
        personal || volume,
        "Native Trash directory is not the requesting user's location"
    );
    Ok(trash)
}
pub fn trash_directory_worker() -> Result<()> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().take(16385).read_to_string(&mut input)?;
    anyhow::ensure!(input.len() <= 16384, "Invalid Trash resolution request");
    let (uid, path): (u32, PathBuf) = serde_json::from_str(&input)?;
    let (_, gid) = account(uid)?;
    // This fixed worker is a separate, single-threaded process. Drop all root
    // supplementary groups as well as real/effective IDs before Foundation.
    if unsafe { libc::geteuid() } == 0 {
        anyhow::ensure!(
            unsafe { libc::setgroups(0, std::ptr::null()) } == 0
                && unsafe { libc::setgid(gid) } == 0
                && unsafe { libc::setuid(uid) } == 0,
            "Unable to drop Trash worker privileges"
        );
    }
    anyhow::ensure!(
        unsafe { libc::getuid() } == uid && unsafe { libc::geteuid() } == uid,
        "Trash worker must run as the requesting user"
    );
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn coqui_user_trash_directory(path: *const libc::c_char) -> *mut libc::c_char;
        }
        let path = std::ffi::CString::new(path.to_str().context("Invalid Trash source")?)?;
        let raw = unsafe { coqui_user_trash_directory(path.as_ptr()) };
        anyhow::ensure!(!raw.is_null(), "Native user Trash lookup failed");
        let text = unsafe { std::ffi::CStr::from_ptr(raw) }
            .to_str()?
            .to_owned();
        unsafe { libc::free(raw.cast()) };
        println!("{}", serde_json::to_string(&PathBuf::from(text))?);
    }
    Ok(())
}
use std::os::unix::fs::DirBuilderExt;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn challenges_are_bound_to_uid_operation_and_single_use() {
        let c = Connection::open_in_memory().unwrap();
        initialize(&c).unwrap();
        let op = Operation::Lifecycle {
            action: Lifecycle::Purge,
        };
        let id = challenge(&c, 501, &op).unwrap();
        assert!(consume(&c, 502, &id, &op).is_err());
        assert!(consume(&c, 501, &id, &op).is_ok());
        assert!(consume(&c, 501, &id, &op).is_err());
        let id = challenge(&c, 501, &op).unwrap();
        assert!(consume(
            &c,
            501,
            &id,
            &Operation::Lifecycle {
                action: Lifecycle::Stop
            }
        )
        .is_err());
        assert!(consume(&c, 501, &id, &op).is_err());
    }
    #[test]
    fn expired_grants_and_changed_policy_do_not_reuse_approval_authority() {
        let d = tempfile::tempdir().unwrap();
        let c = database::open(&d.path().join("helper.sqlite3")).unwrap();
        initialize(&c).unwrap();
        let original = Preferences {
            excluded_paths: vec![],
            protected_paths: vec![],
            retention_days: 30,
        };
        set_current_preferences(&c, &original).unwrap();
        c.execute(
            "INSERT INTO removal_plans VALUES('plan',?1,'review','{}')",
            [crate::model::now()],
        )
        .unwrap();
        save_policy(&c, "plan", &original).unwrap();
        assert!(check_policy(&c, "plan", &original).is_ok());
        let mut changed = original.clone();
        changed
            .excluded_paths
            .push("/fixture/newly-excluded".into());
        set_current_preferences(&c, &changed).unwrap();
        assert!(check_policy(&c, "plan", &original).is_err());
        assert!(check_policy(&c, "plan", &changed).is_err());
        let op = Operation::Lifecycle {
            action: Lifecycle::Stop,
        };
        let id = challenge(&c, 501, &op).unwrap();
        c.execute(
            "UPDATE helper_challenges SET expires_at='2000-01-01T00:00:00Z'",
            [],
        )
        .unwrap();
        assert!(consume(&c, 501, &id, &op).is_err());
    }
    #[test]
    fn legacy_settings_default_off_and_local_mirrors_purge_with_retention() {
        let d = tempfile::tempdir().unwrap();
        let c = database::open(&d.path().join("helper.sqlite3")).unwrap();
        initialize(&c).unwrap();
        let old = serde_json::to_value(AppSettings::default()).unwrap();
        let mut old = old.as_object().unwrap().clone();
        old.remove("administratorEnabled");
        let settings: AppSettings = serde_json::from_value(serde_json::Value::Object(old)).unwrap();
        assert!(!settings.administrator_enabled);
        database::save_settings(&c, &settings).unwrap();
        c.execute("INSERT INTO administrator_records VALUES('fixture','removal','2000-01-01T00:00:00Z','{}')",[]).unwrap();
        database::retain(&c, 30).unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM administrator_records", [], |r| r
                .get::<_, u32>(0))
                .unwrap(),
            0
        );
        c.execute(
            "INSERT INTO administrator_records VALUES('fixture','removal',?1,'{}')",
            [crate::model::now()],
        )
        .unwrap();
        database::purge(&c).unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM administrator_records", [], |r| r
                .get::<_, u32>(0))
                .unwrap(),
            0
        );
        assert!(!database::settings(&c).unwrap().administrator_enabled);
        save_policy(
            &c,
            "expired-plan",
            &Preferences {
                excluded_paths: vec!["/disposable/old".into()],
                protected_paths: vec![],
                retention_days: 30,
            },
        )
        .unwrap();
        set_current_preferences(
            &c,
            &Preferences {
                excluded_paths: vec![],
                protected_paths: vec![],
                retention_days: 30,
            },
        )
        .unwrap();
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM helper_policy WHERE id='expired-plan'",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
            0
        );
    }
    #[test]
    #[cfg(target_os = "macos")]
    fn prepared_authorization_does_not_touch_files_and_cannot_start_from_another_process() {
        let d = tempfile::tempdir().unwrap();
        let c = database::open(&d.path().join("helper.sqlite3")).unwrap();
        initialize(&c).unwrap();
        let file = d.path().join("untouched fixture");
        std::fs::write(&file, "fixture").unwrap();
        let preferences = Preferences {
            excluded_paths: vec![],
            protected_paths: vec![],
            retention_days: 30,
        };
        set_current_preferences(&c, &preferences).unwrap();
        let op = Operation::PreviewRemoval {
            request: RemovalRequest {
                paths: vec![file.to_string_lossy().into_owned()],
                application: None,
            },
            preferences: Preferences {
                excluded_paths: vec![],
                protected_paths: vec![],
                retention_days: 30,
            },
        };
        let pid = std::process::id() as i32;
        let j = prepare(&c, pid, &op).unwrap();
        assert_eq!(j.status, "authorized");
        assert!(j.plan.is_none());
        assert_eq!(
            c.query_row("SELECT count(*) FROM removal_plans", [], |r| r
                .get::<_, u32>(0))
                .unwrap(),
            0
        );
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "fixture");
        assert!(start_prepared(&c, unsafe { libc::getuid() }, pid + 1, &j.id).is_err());
        assert!(start_prepared(&c, unsafe { libc::getuid() }, pid, &j.id).is_err());
        let stale = prepare(&c, pid, &op).unwrap();
        let mut changed = preferences;
        changed
            .excluded_paths
            .push(file.to_string_lossy().into_owned());
        set_current_preferences(&c, &changed).unwrap();
        assert!(
            start_prepared(&c, unsafe { libc::getuid() }, pid, &stale.id)
                .err()
                .unwrap()
                .to_string()
                .contains("preferences changed")
        );
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "fixture");
    }
    #[test]
    fn status_lease_and_active_cancellation_do_not_wait_for_a_journal_writer() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("helper.sqlite3");
        let writer = database::open(&path).unwrap();
        initialize(&writer).unwrap();
        save(
            &writer,
            &Job {
                id: "fixture-job".into(),
                revision: 1,
                status: "running".into(),
                kind: "scan".into(),
                session: None,
                plan: None,
                application: None,
                error: None,
            },
        )
        .unwrap();
        writer
            .execute_batch("BEGIN IMMEDIATE;UPDATE helper_jobs SET status=status;")
            .unwrap();
        let active = ActiveJob {
            uid: 501,
            id: "fixture-job".into(),
            pid: 123,
            start: 42,
            cancel: Arc::new(AtomicBool::new(false)),
            seen: Arc::new(AtomicU64::new(u64::MAX)),
            ack: Arc::new(AtomicU64::new(0)),
        };
        assert!(signal_record(&active, 502, 123, 42, true).is_err());
        assert!(signal_record(&active, 501, 124, 42, true).is_err());
        assert!(signal_record(&active, 501, 123, 43, true).is_err());
        assert!(!active.cancel.load(Ordering::SeqCst));
        let start = std::time::Instant::now();
        signal_record(&active, 501, 123, 42, false).unwrap();
        let reader = database::open(&path).unwrap();
        assert_eq!(job(&reader, "fixture-job").unwrap().status, "running");
        assert_ne!(active.seen.load(Ordering::SeqCst), u64::MAX);
        signal_record(&active, 501, 123, 42, true).unwrap();
        assert!(active.cancel.load(Ordering::SeqCst));
        assert!(start.elapsed() < std::time::Duration::from_millis(500));
        writer.execute_batch("ROLLBACK").unwrap();
    }
    #[test]
    fn unknown_commands_versions_and_ambiguous_preferences_are_rejected() {
        assert!(serde_json::from_str::<Envelope>(
            r#"{"protocol":2,"version":"0.8.0","request":{"operation":"command","command":"rm"}}"#
        )
        .is_err());
        assert!(dispatch(
            r#"{"protocol":1,"version":"0.8.0","request":{"operation":"health"}}"#,
            501,
            0
        )
        .is_err());
        assert!(dispatch(
            r#"{"protocol":2,"version":"wrong","request":{"operation":"health"}}"#,
            501,
            0
        )
        .is_err());
        assert!(validate_preferences(&Preferences {
            excluded_paths: vec!["/tmp/../System".into()],
            protected_paths: vec![],
            retention_days: 30
        })
        .is_err());
    }
}
