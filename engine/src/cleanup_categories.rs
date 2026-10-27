//! Read-only, file-only category views over the retained index. No cleanup authority.
use crate::{cleanup::Policy, database, model::*, volumes};
use anyhow::Result;
use rusqlite::{params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const XCODE_GUIDE: &str = "https://help.apple.com/xcode/mac/current/en.lproj/deva52afe8a4.html";
pub const BACKUP_GUIDE: &str = "https://support.apple.com/en-us/108809";
pub const SYMBOL_GUIDE: &str =
    "https://developer.apple.com/documentation/xcode/locating-a-missing-debug-symbol-file";
const RULE_DATE: &str = "2026-10-27";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum CleanupCategoryKind {
    UserCaches,
    XcodeCaches,
    UserLogs,
    XcodeDerivedData,
    XcodeArchives,
    XcodeDeviceSupport,
    SimulatorData,
    DeviceBackups,
    Trash,
    SharedCaches,
    SystemLogs,
    SystemCaches,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum CleanupCategoryMode {
    CacheReview,
    DeveloperReview,
    ManualReview,
    Protected,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategory {
    pub kind: CleanupCategoryKind,
    pub title: String,
    pub roots: Vec<String>,
    pub mode: CleanupCategoryMode,
    pub state: String,
    #[ts(type = "number | null")]
    pub file_count: Option<u64>,
    pub logical_bytes: Option<f64>,
    pub allocated_bytes: Option<f64>,
    pub description: String,
    pub consequences: String,
    pub qualification: String,
    pub reference_url: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategoryReport {
    pub session_id: String,
    pub observed_at: Option<String>,
    pub source_status: String,
    pub rules_reviewed_at: String,
    pub categories: Vec<CleanupCategory>,
    pub qualification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupCategoryQuery {
    pub session_id: String,
    pub kind: CleanupCategoryKind,
    pub offset: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategoryPage {
    pub session_id: String,
    pub kind: CleanupCategoryKind,
    pub files: Vec<FileEntry>,
    pub next_offset: Option<u32>,
    pub qualification: String,
}

struct Definition {
    kind: CleanupCategoryKind,
    title: &'static str,
    roots: Vec<PathBuf>,
    omit: Vec<PathBuf>,
    mode: CleanupCategoryMode,
    description: &'static str,
    consequences: &'static str,
    reference: Option<&'static str>,
}
fn definitions(home: &Path) -> Vec<Definition> {
    use CleanupCategoryKind::*;
    use CleanupCategoryMode::*;
    let xcode_cache = home.join("Library/Caches/com.apple.dt.Xcode");
    vec![
        Definition{kind:UserCaches,title:"App caches",roots:vec![home.join("Library/Caches")],omit:vec![xcode_cache.clone()],mode:CacheReview,description:"Indexed files in your Library/Caches, excluding the separate Xcode cache group.",consequences:"Cached data can be useful or expensive to download again. Select individual files for fresh eligibility checks; project-managed, linked, cloud or changed files can be refused.",reference:None},
        Definition{kind:XcodeCaches,title:"Xcode caches",roots:vec![xcode_cache],omit:vec![],mode:CacheReview,description:"Files under Xcode's recognized user-cache location; this does not include all Xcode-generated data.",consequences:"Xcode may need to recreate or download data. Close related tools first. The ordinary individual-cache rules still apply; a path label never establishes eligibility.",reference:None},
        Definition{kind:UserLogs,title:"User logs",roots:vec![home.join("Library/Logs")],omit:vec![],mode:ManualReview,description:"Indexed user application logs and diagnostic files in the standard Logs folder.",consequences:"Logs can be active or needed for troubleshooting. Inspect them with the owning app or Finder. This category adds no log-removal action.",reference:None},
        Definition{kind:XcodeDerivedData,title:"Xcode Derived Data",roots:vec![home.join("Library/Developer/Xcode/DerivedData")],omit:vec![],mode:DeveloperReview,description:"The default DerivedData location can contain indexes, logs and build products. Custom or project-relative locations are outside this group.",consequences:"Rebuilding needs source, dependencies and time. Project association and unique contents are unverified here. Developer review preserves its existing project and Git protections; unassociated Xcode data stays manual.",reference:Some(XCODE_GUIDE)},
        Definition{kind:XcodeArchives,title:"Xcode archives",roots:vec![home.join("Library/Developer/Xcode/Archives")],omit:vec![],mode:ManualReview,description:"The default archive location used by Xcode; custom archive paths are outside this group.",consequences:"Archives may contain distribution artifacts and debug symbols needed for crash reports. Review in Xcode Organizer and retain archives you need. Coqui does not remove them here.",reference:Some(SYMBOL_GUIDE)},
        Definition{kind:XcodeDeviceSupport,title:"Xcode device support",roots:vec![home.join("Library/Developer/Xcode/iOS DeviceSupport"),home.join("Library/Developer/Xcode/watchOS DeviceSupport"),home.join("Library/Developer/Xcode/tvOS DeviceSupport")],omit:vec![],mode:ManualReview,description:"Recognized default device-support paths; installed platform versions and current debugging needs are unverified.",consequences:"Removing support files can affect device debugging or require downloads. Review installed platforms and devices in Xcode. No prune command or removal action is added.",reference:None},
        Definition{kind:SimulatorData,title:"Simulator device data",roots:vec![home.join("Library/Developer/CoreSimulator/Devices")],omit:vec![],mode:ManualReview,description:"Indexed data beneath the standard simulator-device location; this is not a runtime-installation inventory.",consequences:"Simulated devices may contain app state and test data you need. Manage devices in Xcode or Simulator. No reset, erase or automatic deletion is performed.",reference:None},
        Definition{kind:DeviceBackups,title:"iPhone and iPad backups",roots:vec![home.join("Library/Application Support/MobileSync/Backup")],omit:vec![],mode:ManualReview,description:"The standard local device-backup folder. Completeness, encryption and other copies are unverified.",consequences:"A backup may be your only recovery copy. Use Finder's device General → Manage Backups review; keep or archive needed backups. Coqui does not edit or remove backup internals.",reference:Some(BACKUP_GUIDE)},
        Definition{kind:Trash,title:"Trash",roots:vec![home.join(".Trash")],omit:vec![],mode:Protected,description:"Indexed files in your account's standard Trash folder; other volumes' Trash is outside this group.",consequences:"Trash may contain recoverable work. Review it in Finder. Coqui never empties Trash; its indexed allocation is not a promise of available or reclaimable capacity.",reference:None},
        Definition{kind:SharedCaches,title:"Shared caches",roots:vec![PathBuf::from("/Library/Caches")],omit:vec![],mode:Protected,description:"Shared application caches from the retained scan; they are distinct from your user caches.",consequences:"Other users and services may depend on these files. This view is inspection only. Existing separately authorized storage/helper review is not enabled or approved by this category.",reference:None},
        Definition{kind:SystemLogs,title:"System logs",roots:vec![PathBuf::from("/private/var/log")],omit:vec![],mode:Protected,description:"Indexed entries in a recognized system-log location; protected or separately managed logs can be omitted.",consequences:"System and diagnostic services manage these files. Do not change permissions or weaken protections to remove them. This category has no removal action.",reference:None},
        Definition{kind:SystemCaches,title:"System caches",roots:vec![PathBuf::from("/System/Library/Caches")],omit:vec![],mode:Protected,description:"Indexed files in the standard system-cache location; this is not a complete system-data inventory.",consequences:"macOS manages protected system data. Inspection adds no cleanup authority, security change or repair action.",reference:None},
    ]
}

fn aliases(root: &Path) -> Vec<PathBuf> {
    let root = volumes::friendly(root);
    let mut paths = vec![root.clone()];
    if let Ok(relative) = root.strip_prefix("/") {
        let data = Path::new("/System/Volumes/Data").join(relative);
        if data != root && volumes::friendly(&data) == root {
            paths.push(data);
        }
    }
    paths
}
// Each branch keeps an exact primary-key path range. Current exclusions and
// disjoint category subgroups are translated into that branch's published alias.
fn branches(
    definition: &Definition,
    policy: &Policy,
    session: &str,
    values: &mut Vec<Value>,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        policy.exclusions.len() <= 200,
        "At most 200 exclusions are supported by category review; use Storage for this saved index"
    );
    let excluded = policy
        .exclusions
        .iter()
        .chain(&definition.omit)
        .map(|path| volumes::friendly(path))
        .collect::<Vec<_>>();
    let mut queries = vec![];
    for base in &definition.roots {
        let base = volumes::friendly(base);
        if excluded.iter().any(|path| base.starts_with(path)) {
            continue;
        }
        for root in aliases(&base) {
            let name = root
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Category root is not representable"))?;
            let index = values.len() + 1;
            values.extend([
                Value::Text(session.into()),
                Value::Text(format!("{name}/")),
                Value::Text(format!("{name}0")),
            ]);
            let mut condition = format!(
                "session_id=?{index} AND path>=?{} AND path<?{}",
                index + 1,
                index + 2
            );
            for exclusion in &excluded {
                if let Ok(tail) = exclusion.strip_prefix(&base) {
                    let excluded = root.join(tail).to_string_lossy().into_owned();
                    let index = values.len() + 1;
                    condition.push_str(&format!(
                        " AND NOT(path=?{index} OR (path>=?{} AND path<?{}))",
                        index + 1,
                        index + 2
                    ));
                    values.extend([
                        Value::Text(excluded.clone()),
                        Value::Text(format!("{excluded}/")),
                        Value::Text(format!("{excluded}0")),
                    ]);
                }
            }
            queries.push(condition);
        }
    }
    Ok(queries)
}

pub fn report(c: &Connection, session_id: &str, policy: &Policy) -> Result<CleanupCategoryReport> {
    anyhow::ensure!(
        policy.home.is_absolute(),
        "An absolute account home is required"
    );
    let transaction = c.unchecked_transaction()?;
    let session = database::session(&transaction, session_id)?;
    anyhow::ensure!(
        session.status != "running",
        "Wait for the storage scan before reviewing categories"
    );
    let mut categories = vec![];
    for definition in definitions(&policy.home) {
        let mut values = vec![];
        let conditions = branches(&definition, policy, session_id, &mut values)?;
        let mut indexed = false;
        let (mut count, mut logical, mut allocated) = (0u64, 0f64, 0f64);
        if !conditions.is_empty() {
            let query=format!("SELECT COUNT(*),COALESCE(SUM(CASE WHEN directory=0 THEN 1 ELSE 0 END),0),COALESCE(SUM(CASE WHEN directory=0 THEN logical ELSE 0 END),0),COALESCE(SUM(CASE WHEN directory=0 THEN allocated ELSE 0 END),0) FROM ({})",conditions.iter().map(|condition|format!("SELECT directory,logical,allocated FROM files INDEXED BY sqlite_autoindex_files_1 WHERE {condition}")).collect::<Vec<_>>().join(" UNION ALL "));
            let totals = transaction.query_row(&query, params_from_iter(values.iter()), |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    row.get::<_, u64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            })?;
            indexed = totals.0 > 0;
            count = totals.1;
            logical = totals.2;
            allocated = totals.3;
            for root in &definition.roots {
                if policy
                    .exclusions
                    .iter()
                    .any(|excluded| volumes::same_or_below(root, excluded))
                {
                    continue;
                }
                for alias in aliases(root) {
                    indexed |= transaction.query_row("SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1 AND path=?2 AND directory=1)",rusqlite::params![session_id,alias.to_string_lossy()],|row|row.get::<_,bool>(0))?;
                }
            }
        }
        categories.push(CleanupCategory{kind:definition.kind,title:definition.title.into(),roots:definition.roots.iter().map(|path|path.to_string_lossy().into_owned()).collect(),mode:definition.mode,state:if conditions.is_empty(){"excluded"}else if !indexed{"notIndexed"}else if session.status!="complete" || !session.errors.is_empty() || !session.permissions.is_empty(){"partial"}else{"indexed"}.into(),file_count:indexed.then_some(count),logical_bytes:indexed.then_some(logical),allocated_bytes:indexed.then_some(allocated),description:definition.description.into(),consequences:definition.consequences.into(),qualification:"Retained file rows after current exclusions; cumulative directory sizes are excluded. Path recognition does not verify ownership, present contents, age, cleanup eligibility or recoverable space. Custom locations and unindexed data remain unknown.".into(),reference_url:definition.reference.map(str::to_owned)});
    }
    let result=CleanupCategoryReport{session_id:session.id,observed_at:session.finished_at.filter(|date|!date.is_empty()),source_status:session.status,rules_reviewed_at:RULE_DATE.into(),categories,qualification:"Saved metadata only; no new traversal, content read, Git check or removal review was started. Counts are indexed file rows, not removable or unique physical bytes. Coverage omissions, aliases, APFS sharing and live changes remain qualified. Standard paths only; missing groups are not measured zero or proof an app is absent.".into()};
    transaction.commit()?;
    Ok(result)
}

pub fn entries(
    c: &Connection,
    query: CleanupCategoryQuery,
    policy: &Policy,
) -> Result<CleanupCategoryPage> {
    anyhow::ensure!(
        policy.home.is_absolute(),
        "An absolute account home is required"
    );
    let transaction = c.unchecked_transaction()?;
    let session = database::session(&transaction, &query.session_id)?;
    anyhow::ensure!(
        session.status != "running",
        "Wait for the storage scan before reviewing categories"
    );
    let definition = definitions(&policy.home)
        .into_iter()
        .find(|definition| definition.kind == query.kind)
        .unwrap();
    let mut values = vec![];
    let conditions = branches(&definition, policy, &query.session_id, &mut values)?;
    let mut files = vec![];
    if !conditions.is_empty() {
        let index = values.len() + 1;
        values.push(Value::Integer(query.offset.into()));
        let sql=format!("SELECT * FROM ({}) ORDER BY allocated DESC,path LIMIT 101 OFFSET ?{index}",conditions.iter().map(|condition|format!("SELECT path,parent,name,category,logical,allocated,directory,modified,qualification FROM files INDEXED BY sqlite_autoindex_files_1 WHERE {condition} AND directory=0")).collect::<Vec<_>>().join(" UNION ALL "));
        let mut statement = transaction.prepare(&sql)?;
        files = statement
            .query_map(params_from_iter(values.iter()), |row| {
                Ok(FileEntry {
                    path: row.get(0)?,
                    parent: row.get(1)?,
                    name: row.get(2)?,
                    category: row.get(3)?,
                    logical_bytes: row.get(4)?,
                    allocated_bytes: row.get(5)?,
                    is_directory: row.get(6)?,
                    modified_at: row.get(7)?,
                    qualification: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
    }
    let next_offset = if files.len() > 100 {
        files.truncate(100);
        query.offset.checked_add(100)
    } else {
        None
    };
    transaction.commit()?;
    Ok(CleanupCategoryPage{session_id:query.session_id,kind:query.kind,files,next_offset,qualification:"Indexed file page only. Present availability and cleanup eligibility are unverified; a separate fresh review is required for supported cache files. Other categories remain manual or protected.".into()})
}
