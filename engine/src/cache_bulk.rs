//! Disk-backed, explicit user-cache selection and single-use bulk review.
//! Group names are namespaces, never application ownership or folder authority.
use crate::{
    cleanup::{self, Policy},
    cleanup_categories::CleanupCategoryKind,
    database,
    model::now,
};
use anyhow::{Context, Result};
use rusqlite::{params, params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheQuery {
    pub session_id: String,
    pub batch_id: Option<String>,
    pub kind: Option<CleanupCategoryKind>,
    pub group: Option<String>,
    pub cursor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CacheGroup {
    pub key: String,
    pub title: String,
    #[ts(type = "number")]
    pub file_count: u64,
    pub allocated_bytes: f64,
    #[ts(type = "number")]
    pub selected_count: u64,
    #[ts(type = "number")]
    pub eligible_count: u64,
    #[ts(type = "number")]
    pub refused_count: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CacheGroupPage {
    pub groups: Vec<CacheGroup>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CacheBulkItem {
    pub path: String,
    pub group: String,
    pub allocated_bytes: f64,
    pub logical_bytes: f64,
    pub modified_at: String,
    pub selected: bool,
    pub eligible: bool,
    pub status: String,
    pub detail: String,
    pub trash_path: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CacheBulkPage {
    pub items: Vec<CacheBulkItem>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheSelectionRequest {
    pub query: CacheQuery,
    pub revision: Option<u32>,
    /// None selects the complete group/category; Some selects only these indexed paths.
    pub paths: Option<Vec<String>>,
    pub selected: bool,
    pub clear: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CacheBulkSummary {
    pub id: String,
    pub session_id: String,
    pub revision: u32,
    pub status: String,
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub expires_at: Option<String>,
    #[ts(type = "number")]
    pub selected_count: u64,
    #[ts(type = "number")]
    pub processed_count: u64,
    #[ts(type = "number")]
    pub eligible_count: u64,
    #[ts(type = "number")]
    pub refused_count: u64,
    #[ts(type = "number")]
    pub moved_count: u64,
    #[ts(type = "number")]
    pub skipped_count: u64,
    #[ts(type = "number")]
    pub failed_count: u64,
    #[ts(type = "number")]
    pub unverified_count: u64,
    #[ts(type = "number")]
    pub unattempted_count: u64,
    pub allocated_bytes: f64,
    pub reviewed_bytes: f64,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheBulkApproval {
    pub id: String,
    pub revision: u32,
    pub reviewed_at: String,
    pub applications_closed: bool,
    pub consequences_accepted: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheBulkPageQuery {
    pub id: String,
    pub cursor: Option<String>,
    pub status: Option<String>,
}

fn policy_key(policy: &Policy) -> Result<String> {
    Ok(serde_json::to_string(&(
        policy.home.to_string_lossy(),
        &policy.exclusions,
    ))?)
}
fn check_batch(c: &Connection, id: &str, session: &str, policy: &Policy) -> Result<String> {
    let (source, key, status): (String, String, String) = c
        .query_row(
            "SELECT session_id,policy,status FROM cache_batches WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .context("Cache selection was not found. Start a new selection.")?;
    anyhow::ensure!(
        source == session && key == policy_key(policy)?,
        "The source scan or exclusions changed. Clear this selection and start again."
    );
    Ok(status)
}
// All paths originate in fixed retained account-cache ranges. The friendly Data
// alias is normalized before grouping/deduplication; no LIKE wildcard scopes.
fn source(query: &CacheQuery, policy: &Policy) -> Result<(String, Vec<Value>)> {
    source_page(query, policy, None, None)
}
fn source_page(
    query: &CacheQuery,
    policy: &Policy,
    after: Option<&str>,
    limit: Option<u32>,
) -> Result<(String, Vec<Value>)> {
    anyhow::ensure!(
        matches!(
            query.kind,
            None | Some(CleanupCategoryKind::UserCaches) | Some(CleanupCategoryKind::XcodeCaches)
        ),
        "This category has no cache selection"
    );
    anyhow::ensure!(
        policy.home.is_absolute() && policy.exclusions.len() <= 200,
        "Cache review requires an absolute home and at most 200 exclusions"
    );
    if let Some(group) = &query.group {
        anyhow::ensure!(
            !group.contains('/') && group != "." && group != "..",
            "Choose an indexed cache namespace"
        );
    }
    let base = policy.home.join("Library/Caches");
    let base_text = base.to_string_lossy().into_owned();
    let mut values = vec![
        Value::Text(query.session_id.clone()),
        Value::Text(base_text.clone()),
    ];
    let mut branches = vec![];
    let mut roots = vec![base.clone()];
    let data = PathBuf::from(format!("/System/Volumes/Data{base_text}"));
    if crate::volumes::friendly(&data) == base {
        roots.push(data);
    }
    for root in roots {
        let root_text = root.to_string_lossy().into_owned();
        values.push(Value::Text(root_text.clone()));
        let r = values.len();
        let mut condition =
            format!("session_id=?1 AND directory=0 AND path>=?{r}||'/' AND path<?{r}||'0'");
        if let Some(group) = &query.group {
            if group.is_empty() {
                condition.push_str(&format!(" AND parent=?{r}"));
            } else {
                values.push(Value::Text(root.join(group).to_string_lossy().into_owned()));
                let g = values.len();
                condition.push_str(&format!(" AND path>=?{g}||'/' AND path<?{g}||'0'"));
            }
        }
        if let Some(kind) = &query.kind {
            values.push(Value::Text(
                root.join("com.apple.dt.Xcode")
                    .to_string_lossy()
                    .into_owned(),
            ));
            let x = values.len();
            condition.push_str(&format!(
                " AND {}(path>=?{x}||'/' AND path<?{x}||'0')",
                if *kind == CleanupCategoryKind::UserCaches {
                    "NOT "
                } else {
                    ""
                }
            ));
        }
        for exclusion in &policy.exclusions {
            let friendly = crate::volumes::friendly(exclusion);
            if base.starts_with(&friendly) {
                condition.push_str(" AND 0");
                continue;
            }
            if let Ok(tail) = friendly.strip_prefix(&base) {
                values.push(Value::Text(root.join(tail).to_string_lossy().into_owned()));
                let e = values.len();
                condition.push_str(&format!(
                    " AND NOT(path=?{e} OR (path>=?{e}||'/' AND path<?{e}||'0'))"
                ));
            }
        }
        if let Some(after) = after.filter(|v| !v.is_empty()) {
            anyhow::ensure!(
                Path::new(after).starts_with(&base),
                "Invalid cache page cursor"
            );
            let tail = Path::new(after).strip_prefix(&base)?;
            values.push(Value::Text(root.join(tail).to_string_lossy().into_owned()));
            let a = values.len();
            condition.push_str(&format!(" AND path>?{a}"));
        }
        let bounded = limit
            .map(|n| format!(" ORDER BY path LIMIT {n}"))
            .unwrap_or_default();
        branches.push(format!("SELECT ?2||substr(path,length(?{r})+1) AS path,logical,allocated,modified FROM (SELECT path,logical,allocated,modified FROM files INDEXED BY sqlite_autoindex_files_1 WHERE {condition}{bounded})"));
    }
    let sql=format!("WITH raw AS ({}), normalized AS (SELECT path,MAX(logical) AS logical,MAX(allocated) AS allocated,MAX(modified) AS modified FROM raw GROUP BY path), scoped AS (SELECT *,substr(path,length(?2)+2) AS relative FROM normalized), candidates AS (SELECT path,logical,allocated,modified,CASE WHEN instr(relative,'/')=0 THEN '' ELSE substr(relative,1,instr(relative,'/')-1) END AS namespace FROM scoped) ",branches.join(" UNION ALL "));
    Ok((sql, values))
}
fn scope(query: &CacheQuery, values: &mut Vec<Value>) -> String {
    let mut condition = "1=1".to_string();
    if let Some(kind) = &query.kind {
        condition.push_str(if *kind == CleanupCategoryKind::XcodeCaches {
            " AND namespace='com.apple.dt.Xcode'"
        } else {
            " AND namespace!='com.apple.dt.Xcode'"
        });
    }
    if let Some(group) = &query.group {
        values.push(Value::Text(group.clone()));
        condition.push_str(&format!(" AND namespace=?{}", values.len()));
    }
    condition
}
fn ensure_source(c: &Connection, session: &str) -> Result<()> {
    anyhow::ensure!(
        database::session(c, session)?.status != "running",
        "Wait for the storage scan to finish"
    );
    Ok(())
}
fn title(key: &str) -> String {
    if key.is_empty() {
        "Files directly in Caches".into()
    } else {
        key.into()
    }
}

pub fn groups(c: &Connection, query: CacheQuery, policy: &Policy) -> Result<CacheGroupPage> {
    ensure_source(c, &query.session_id)?;
    if let Some(id) = &query.batch_id {
        check_batch(c, id, &query.session_id, policy)?;
    }
    let (mut sql, mut values) = source(&query, policy)?;
    let condition = scope(&query, &mut values);
    values.push(Value::Text(query.batch_id.clone().unwrap_or_default()));
    let batch = values.len();
    values.push(Value::Text(query.cursor.clone().unwrap_or_default()));
    let cursor = values.len();
    sql.push_str(&format!("SELECT namespace,COUNT(*),SUM(allocated),(SELECT COUNT(*) FROM cache_batch_items b WHERE b.batch_id=?{batch} AND b.namespace=candidates.namespace),(SELECT COUNT(*) FROM cache_batch_items b WHERE b.batch_id=?{batch} AND b.namespace=candidates.namespace AND b.eligible=1),(SELECT COUNT(*) FROM cache_batch_items b WHERE b.batch_id=?{batch} AND b.namespace=candidates.namespace AND b.status='refused') FROM candidates WHERE {condition} {} GROUP BY namespace ORDER BY namespace LIMIT 101",if query.cursor.is_some(){format!("AND namespace>?{cursor}")}else{format!("AND ?{cursor}=?{cursor}")}));
    let mut rows = c
        .prepare(&sql)?
        .query_map(params_from_iter(values.iter()), |r| {
            let key: String = r.get(0)?;
            Ok(CacheGroup {
                title: title(&key),
                key,
                file_count: r.get(1)?,
                allocated_bytes: r.get(2)?,
                selected_count: r.get(3)?,
                eligible_count: r.get(4)?,
                refused_count: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let next_cursor = if rows.len() > 100 {
        rows.truncate(100);
        rows.last().map(|r| r.key.clone())
    } else {
        None
    };
    Ok(CacheGroupPage {
        groups: rows,
        next_cursor,
    })
}
pub fn entries(c: &Connection, query: CacheQuery, policy: &Policy) -> Result<CacheBulkPage> {
    ensure_source(c, &query.session_id)?;
    if let Some(id) = &query.batch_id {
        check_batch(c, id, &query.session_id, policy)?;
    }
    let (mut sql, mut values) = source_page(&query, policy, query.cursor.as_deref(), Some(101))?;
    let condition = scope(&query, &mut values);
    values.push(Value::Text(query.batch_id.unwrap_or_default()));
    let batch = values.len();
    values.push(Value::Text(query.cursor.unwrap_or_default()));
    let cursor = values.len();
    sql.push_str(&format!("SELECT path,namespace,allocated,logical,modified,EXISTS(SELECT 1 FROM cache_batch_items b WHERE b.batch_id=?{batch} AND b.path=candidates.path) FROM candidates WHERE {condition} AND path>?{cursor} ORDER BY path LIMIT 101"));
    let mut items = c
        .prepare(&sql)?
        .query_map(params_from_iter(values.iter()), |r| {
            Ok(CacheBulkItem {
                path: r.get(0)?,
                group: r.get(1)?,
                allocated_bytes: r.get(2)?,
                logical_bytes: r.get(3)?,
                modified_at: r.get(4)?,
                selected: r.get(5)?,
                eligible: false,
                status: "indexed".into(),
                detail: "Indexed candidate; fresh checks required".into(),
                trash_path: None,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let next_cursor = if items.len() > 100 {
        items.truncate(100);
        items.last().map(|r| r.path.clone())
    } else {
        None
    };
    Ok(CacheBulkPage { items, next_cursor })
}
pub fn summary(c: &Connection, id: &str) -> Result<CacheBulkSummary> {
    let mut result=c.query_row("SELECT id,session_id,revision,status,created_at,reviewed_at,expires_at,detail FROM cache_batches WHERE id=?1",[id],|r|Ok(CacheBulkSummary{id:r.get(0)?,session_id:r.get(1)?,revision:r.get(2)?,status:r.get(3)?,created_at:r.get(4)?,reviewed_at:r.get(5)?,expires_at:r.get(6)?,detail:r.get(7)?,selected_count:0,processed_count:0,eligible_count:0,refused_count:0,moved_count:0,skipped_count:0,failed_count:0,unverified_count:0,unattempted_count:0,allocated_bytes:0.,reviewed_bytes:0.}))?;
    c.query_row("SELECT COUNT(*),COALESCE(SUM(status!='selected'),0),COALESCE(SUM(eligible),0),COALESCE(SUM(status='refused'),0),COALESCE(SUM(status='movedToTrash'),0),COALESCE(SUM(status='skipped'),0),COALESCE(SUM(status='failed'),0),COALESCE(SUM(status='unverified'),0),COALESCE(SUM(status IN ('selected','eligible','unattempted')),0),COALESCE(SUM(indexed_allocated),0),COALESCE(SUM(CASE WHEN eligible=1 THEN allocated ELSE 0 END),0) FROM cache_batch_items WHERE batch_id=?1",[id],|r|{result.selected_count=r.get(0)?;result.processed_count=r.get(1)?;result.eligible_count=r.get(2)?;result.refused_count=r.get(3)?;result.moved_count=r.get(4)?;result.skipped_count=r.get(5)?;result.failed_count=r.get(6)?;result.unverified_count=r.get(7)?;result.unattempted_count=r.get(8)?;result.allocated_bytes=r.get(9)?;result.reviewed_bytes=r.get(10)?;Ok(())})?;
    Ok(result)
}
fn state(c: &Connection, id: &str, status: &str, detail: &str) -> Result<()> {
    c.execute(
        "UPDATE cache_batches SET status=?2,detail=?3 WHERE id=?1",
        params![id, status, detail],
    )?;
    Ok(())
}
fn cancelled(cancel: &AtomicBool, connected: bool) -> bool {
    cancel.load(Ordering::SeqCst) || !connected
}

pub fn select(
    c: &Connection,
    request: CacheSelectionRequest,
    policy: &Policy,
    cancel: &AtomicBool,
    mut send: impl FnMut(CacheBulkSummary) -> bool,
) -> Result<CacheBulkSummary> {
    let query = &request.query;
    ensure_source(c, &query.session_id)?;
    if let Some(paths) = &request.paths {
        anyhow::ensure!(
            !paths.is_empty()
                && paths.len() <= 100
                && paths.iter().collect::<std::collections::HashSet<_>>().len() == paths.len(),
            "Choose 1–100 unique indexed paths"
        );
        let (base, mut values) = source(query, policy)?;
        let condition = scope(query, &mut values);
        let slots = paths
            .iter()
            .map(|p| {
                values.push(Value::Text(p.clone()));
                format!("?{}", values.len())
            })
            .collect::<Vec<_>>()
            .join(",");
        let count: u64 = c.query_row(
            &format!(
                "{base} SELECT COUNT(*) FROM candidates WHERE {condition} AND path IN ({slots})"
            ),
            params_from_iter(values.iter()),
            |r| r.get(0),
        )?;
        anyhow::ensure!(
            count == paths.len() as u64,
            "A selected path is outside the chosen indexed cache group"
        );
    }

    let id = if let Some(id) = &query.batch_id {
        anyhow::ensure!(
            check_batch(c, id, &query.session_id, policy)? == "draft",
            "Create a new selection after review"
        );
        anyhow::ensure!(
            Some(summary(c, id)?.revision) == request.revision,
            "Selection changed; refresh before selecting again"
        );
        id.clone()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        c.execute("INSERT INTO cache_batches(id,session_id,policy,revision,status,created_at,detail) VALUES(?1,?2,?3,0,'draft',?4,'Nothing approved')",params![id,query.session_id,policy_key(policy)?,now()])?;
        id
    };
    c.execute(
        "UPDATE cache_batches SET revision=revision+1,detail='Nothing approved' WHERE id=?1",
        [&id],
    )?;
    if request.clear {
        c.execute("DELETE FROM cache_batch_items WHERE batch_id=?1", [&id])?;
        return summary(c, &id);
    }
    if let Some(paths) = &request.paths {
        anyhow::ensure!(
            !paths.is_empty() && paths.len() <= 100,
            "Choose at most one displayed page per file-selection request"
        );
    }
    let mut cursor = String::new();
    let mut progress = summary(c, &id)?;
    let mut connected = send(progress.clone());
    let mut last = Instant::now() - Duration::from_secs(1);
    loop {
        if cancelled(cancel, connected) {
            state(c,&id,"draft","Selection stopped; the displayed count includes only materialized files. Nothing approved.")?;
            break;
        }
        let (base, mut values) = source_page(
            query,
            policy,
            Some(&cursor),
            if request.paths.is_some() {
                None
            } else {
                Some(1000)
            },
        )?;
        let condition = scope(query, &mut values);
        values.push(Value::Text(cursor.clone()));
        let n = values.len();
        let mut condition = format!("{condition} AND path>?{n}");
        if let Some(paths) = &request.paths {
            let slots = paths
                .iter()
                .map(|p| {
                    values.push(Value::Text(p.clone()));
                    format!("?{}", values.len())
                })
                .collect::<Vec<_>>()
                .join(",");
            condition.push_str(&format!(" AND path IN ({slots})"));
        }
        let sql=format!("{base} SELECT path,namespace,logical,allocated,modified FROM candidates WHERE {condition} ORDER BY path LIMIT 1000");
        let rows = c
            .prepare(&sql)?
            .query_map(params_from_iter(values.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                    r.get::<_, f64>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if rows.is_empty() {
            break;
        }
        let tx = c.unchecked_transaction()?;
        {
            let mut insert=tx.prepare_cached("INSERT OR IGNORE INTO cache_batch_items(batch_id,path,namespace,logical,allocated,indexed_allocated,modified,status,detail) VALUES(?1,?2,?3,?4,?5,?5,?6,'selected','Fresh checks required')")?;
            let mut remove =
                tx.prepare_cached("DELETE FROM cache_batch_items WHERE batch_id=?1 AND path=?2")?;
            for (path, namespace, logical, allocated, modified) in &rows {
                let changed = if request.selected {
                    insert.execute(params![id, path, namespace, logical, allocated, modified])?
                } else {
                    remove.execute(params![id, path])?
                };
                if changed > 0 {
                    if request.selected {
                        progress.selected_count += 1;
                        progress.allocated_bytes += allocated;
                        progress.unattempted_count += 1;
                    } else {
                        progress.selected_count -= 1;
                        progress.allocated_bytes -= allocated;
                        progress.unattempted_count -= 1;
                    }
                }
            }
        }
        tx.commit()?;
        cursor = rows.last().unwrap().0.clone();
        if last.elapsed() >= Duration::from_millis(100) {
            connected = send(progress.clone());
            last = Instant::now();
        }
    }
    let result = summary(c, &id)?;
    let _ = send(result.clone());
    Ok(result)
}

pub fn prepare(
    c: &Connection,
    id: &str,
    revision: u32,
    policy: &Policy,
    cancel: &AtomicBool,
    mut send: impl FnMut(CacheBulkSummary) -> bool,
) -> Result<CacheBulkSummary> {
    let initial = summary(c, id)?;
    anyhow::ensure!(
        check_batch(c, id, &initial.session_id, policy)? == "draft"
            && initial.revision == revision
            && initial.selected_count > 0,
        "Select cache files before preparing this review"
    );
    ensure_source(c, &initial.session_id)?;
    state(
        c,
        id,
        "preparing",
        "Checking selected cache files; nothing approved",
    )?;
    let result = (|| -> Result<()> {
        let mut cursor = String::new();
        let mut progress = summary(c, id)?;
        let mut connected = send(progress.clone());
        let mut last = Instant::now();
        loop {
            if cancelled(cancel, connected) {
                state(
                    c,
                    id,
                    "cancelled",
                    "Fresh checking cancelled. Nothing was approved or moved.",
                )?;
                return Ok(());
            }
            let paths=c.prepare("SELECT path FROM cache_batch_items WHERE batch_id=?1 AND path>?2 ORDER BY path LIMIT 1000")?.query_map(params![id,cursor],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            if paths.is_empty() {
                break;
            }
            for path in &paths {
                if cancelled(cancel, connected) {
                    state(
                        c,
                        id,
                        "cancelled",
                        "Fresh checking cancelled. Nothing was approved or moved.",
                    )?;
                    return Ok(());
                }
                let checked = (|| -> Result<_> {
                    let friendly = PathBuf::from(path);
                    let indexed:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1 AND directory=0 AND (path=?2 OR path=?3))",params![initial.session_id,path,format!("/System/Volumes/Data{path}")],|r|r.get(0))?;
                    anyhow::ensure!(indexed, "No longer in the selected scan index");
                    cleanup::inspect(&friendly, policy)
                })();
                match checked {
                    Ok((file, seal)) => {
                        progress.eligible_count += 1;
                        progress.reviewed_bytes += file.allocated_bytes;
                        c.execute("UPDATE cache_batch_items SET eligible=1,status='eligible',logical=?3,allocated=?4,modified=?5,identity=?6,detail='Freshly checked; awaiting approval' WHERE batch_id=?1 AND path=?2",params![id,path,file.logical_bytes,file.allocated_bytes,file.modified_at,serde_json::to_string(&seal)?])?;
                    }
                    Err(e) => {
                        progress.refused_count += 1;
                        c.execute("UPDATE cache_batch_items SET status='refused',detail=?3 WHERE batch_id=?1 AND path=?2",params![id,path,format!("Not eligible: {e:#}")])?;
                    }
                }
                progress.processed_count += 1;
                if last.elapsed() >= Duration::from_millis(100) {
                    connected = send(progress.clone());
                    last = Instant::now();
                }
            }
            cursor = paths.last().unwrap().clone();
        }
        if cancelled(cancel, connected) {
            state(
                c,
                id,
                "cancelled",
                "Fresh checking cancelled. Nothing approved.",
            )?;
            return Ok(());
        }
        let ready = now();
        let expires = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();
        let eligible = summary(c, id)?.eligible_count;
        c.execute(
            "UPDATE cache_batches SET status=?2,reviewed_at=?3,expires_at=?4,detail=?5 WHERE id=?1",
            params![
                id,
                if eligible > 0 { "review" } else { "empty" },
                ready,
                expires,
                "Frozen exact-file review. Refused files are excluded. Nothing moved."
            ],
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        state(
            c,
            id,
            "failed",
            "Fresh checking failed. Nothing approved or moved; create a new selection.",
        )?;
        return Err(e);
    }
    let result = summary(c, id)?;
    if !send(result.clone()) && result.status == "review" {
        c.execute("UPDATE cache_batches SET status='cancelled',reviewed_at=NULL,expires_at=NULL,detail='Review delivery disconnected. Nothing approved or moved.' WHERE id=?1",[id])?;
        return summary(c, id);
    }
    Ok(result)
}
pub fn page(c: &Connection, query: CacheBulkPageQuery) -> Result<CacheBulkPage> {
    summary(c, &query.id)?;
    let filter = match query.status.as_deref() {
        None => "1=1",
        Some("eligible") => "eligible=1",
        Some("refused") => "status='refused'",
        Some("unattempted") => "status IN ('selected','eligible','unattempted')",
        Some("movedToTrash") => "status='movedToTrash'",
        Some("skipped") => "status='skipped'",
        Some("failed") => "status='failed'",
        Some("unverified") => "status='unverified'",
        _ => anyhow::bail!("Unknown result filter"),
    };
    let mut items=c.prepare(&format!("SELECT path,namespace,allocated,logical,modified,eligible,status,detail,trash_path FROM cache_batch_items WHERE batch_id=?1 AND path>?2 AND {filter} ORDER BY path LIMIT 101"))?.query_map(params![query.id,query.cursor.unwrap_or_default()],|r|Ok(CacheBulkItem{path:r.get(0)?,group:r.get(1)?,allocated_bytes:r.get(2)?,logical_bytes:r.get(3)?,modified_at:r.get(4)?,selected:true,eligible:r.get(5)?,status:r.get(6)?,detail:r.get(7)?,trash_path:r.get(8)?}))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let next_cursor = if items.len() > 100 {
        items.truncate(100);
        items.last().map(|r| r.path.clone())
    } else {
        None
    };
    Ok(CacheBulkPage { items, next_cursor })
}
pub fn reviewed_groups(c: &Connection, id: &str, cursor: Option<String>) -> Result<CacheGroupPage> {
    summary(c, id)?;
    let after = cursor.clone().unwrap_or_default();
    let mut groups=c.prepare(&format!("SELECT namespace,COUNT(*),SUM(CASE WHEN eligible=1 THEN allocated ELSE 0 END),SUM(eligible),SUM(status='refused') FROM cache_batch_items WHERE batch_id=?1 {} GROUP BY namespace ORDER BY namespace LIMIT 101",if cursor.is_some(){"AND namespace>?2"}else{"AND ?2=?2"}))?.query_map(params![id,after],|r|{let key:String=r.get(0)?;Ok(CacheGroup{title:title(&key),key,file_count:r.get(1)?,allocated_bytes:r.get(2)?,selected_count:r.get(1)?,eligible_count:r.get(3)?,refused_count:r.get(4)?})})?.collect::<rusqlite::Result<Vec<_>>>()?;
    let next_cursor = if groups.len() > 100 {
        groups.truncate(100);
        groups.last().map(|r| r.key.clone())
    } else {
        None
    };
    Ok(CacheGroupPage {
        groups,
        next_cursor,
    })
}
pub fn history(c: &Connection, offset: u32) -> Result<Vec<CacheBulkSummary>> {
    let ids=c.prepare("SELECT id FROM cache_batches WHERE status!='draft' ORDER BY created_at DESC,id LIMIT 50 OFFSET ?1")?.query_map([offset],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    ids.iter().map(|id| summary(c, id)).collect()
}
pub fn decline(c: &Connection, id: &str) -> Result<CacheBulkSummary> {
    let s = summary(c, id)?;
    anyhow::ensure!(
        matches!(s.status.as_str(), "draft" | "review" | "empty"),
        "This review is already terminal"
    );
    state(
        c,
        id,
        "declined",
        "Kept all files. Create a fresh selection for another attempt.",
    )?;
    summary(c, id)
}
pub fn execute(
    c: &Connection,
    approval: CacheBulkApproval,
    policy: &Policy,
    cancel: &AtomicBool,
    mut trash: impl FnMut(&Path) -> Result<PathBuf>,
    mut send: impl FnMut(CacheBulkSummary) -> bool,
) -> Result<CacheBulkSummary> {
    let id = &approval.id;
    let initial = summary(c, id)?;
    anyhow::ensure!(
        approval.applications_closed && approval.consequences_accepted,
        "Confirm related applications are closed and accept the cache consequences"
    );
    anyhow::ensure!(
        check_batch(c, id, &initial.session_id, policy)? == "review"
            && initial.revision == approval.revision
            && initial.reviewed_at.as_deref() == Some(&approval.reviewed_at),
        "Approval does not match this unused review"
    );
    anyhow::ensure!(
        initial.expires_at.as_ref().is_some_and(
            |e| chrono::DateTime::parse_from_rfc3339(e).is_ok_and(|e| e > chrono::Utc::now())
        ),
        "Review expired. Create a fresh selection."
    );
    state(
        c,
        id,
        "running",
        "Approval recorded; moving only frozen eligible files",
    )?;
    c.execute(
        "UPDATE cache_batches SET approved_at=?2 WHERE id=?1",
        params![id, now()],
    )?;
    let result = (|| -> Result<()> {
        let mut cursor = String::new();
        let mut progress = summary(c, id)?;
        let mut connected = send(progress.clone());
        let mut last = Instant::now();
        loop {
            if cancelled(cancel, connected) {
                break;
            }
            let rows=c.prepare("SELECT path,identity FROM cache_batch_items WHERE batch_id=?1 AND eligible=1 AND status='eligible' AND path>?2 ORDER BY path LIMIT 1000")?.query_map(params![id,cursor],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
            if rows.is_empty() {
                break;
            }
            for (path, json) in &rows {
                if cancelled(cancel, connected) {
                    break;
                }
                let identity: cleanup::Identity = serde_json::from_str(json)?;
                let outcome = cleanup::move_cache_file(
                    Path::new(path),
                    &identity,
                    policy,
                    cancel,
                    &mut trash,
                    || {
                        c.execute("UPDATE cache_batch_items SET status='moving',detail='Intent recorded before native Trash' WHERE batch_id=?1 AND path=?2",params![id,path])?;
                        Ok(())
                    },
                );
                c.execute("UPDATE cache_batch_items SET status=?3,detail=?4,trash_path=?5 WHERE batch_id=?1 AND path=?2",params![id,path,outcome.status,outcome.detail,outcome.trash_path])?;
                progress.unattempted_count = progress.unattempted_count.saturating_sub(1);
                match outcome.status.as_str() {
                    "movedToTrash" => progress.moved_count += 1,
                    "skipped" => progress.skipped_count += 1,
                    "failed" => progress.failed_count += 1,
                    "unverified" => progress.unverified_count += 1,
                    _ => progress.unattempted_count += 1,
                }
                if last.elapsed() >= Duration::from_millis(100) {
                    connected = send(progress.clone());
                    last = Instant::now();
                }
            }
            cursor = rows.last().unwrap().0.clone();
        }
        if cancelled(cancel, connected) {
            c.execute("UPDATE cache_batch_items SET status='unattempted',detail='Not attempted after cancellation or disconnection' WHERE batch_id=?1 AND status='eligible'",[id])?;
            state(c,id,"cancelled","Stopped remaining moves. Inspect per-file outcomes; nothing resumes automatically.")?;
        } else {
            let s = summary(c, id)?;
            state(c,id,if s.moved_count==s.selected_count{"complete"}else{"partial"},"Per-file verification only. Moving to Trash does not free space until it is emptied. Coqui never empties Trash. Recovery depends on the files remaining there.")?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        interrupt(c, id)?;
        return Err(e);
    }
    c.execute(
        "UPDATE cache_batches SET finished_at=?2 WHERE id=?1",
        params![id, now()],
    )?;
    let result = summary(c, id)?;
    let _ = send(result.clone());
    Ok(result)
}
fn interrupt(c: &Connection, id: &str) -> Result<()> {
    c.execute("UPDATE cache_batch_items SET status=CASE WHEN status='moving' THEN 'unverified' ELSE 'unattempted' END,detail=CASE WHEN status='moving' THEN 'Interrupted during move; inspect source and Trash manually. Do not retry automatically.' ELSE 'Not attempted before interruption' END WHERE batch_id=?1 AND status IN ('selected','eligible','moving')",[id])?;
    state(
        c,
        id,
        "interrupted",
        "Interrupted. No automatic retry; inspect uncertain moves manually.",
    )?;
    Ok(())
}
pub fn recover(c: &Connection) -> Result<()> {
    let ids = c
        .prepare(
            "SELECT id FROM cache_batches WHERE status IN ('preparing','running','draft','review')",
        )?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        interrupt(c, &id)?;
    }
    Ok(())
}
