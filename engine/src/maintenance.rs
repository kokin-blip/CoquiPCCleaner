//! Schedules are opt-in and evaluated only while Coqui is open. No launch agent or deletion.
use crate::{model::*, records};
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
pub fn get(c: &Connection) -> Result<Option<MaintenanceRule>> {
    let text: Option<String> = c
        .query_row("SELECT json FROM maintenance WHERE id=1", [], |r| r.get(0))
        .optional()?;
    text.map(|s| Ok(serde_json::from_str(&s)?)).transpose()
}
pub fn save(c: &Connection, rule: &MaintenanceRule) -> Result<()> {
    anyhow::ensure!(
        ["reminder", "scan"].contains(&rule.mode.as_str())
            && (1..=365).contains(&rule.interval_days),
        "Choose reminder/scan and an interval of 1–365 days"
    );
    anyhow::ensure!(["standard","selected"].contains(&rule.scope.mode.as_str()),"Scheduled scopes support standard or explicitly selected folders; deep scans require manual scope confirmation");
    anyhow::ensure!(
        rule.capacity_threshold_percent >= 50 && rule.capacity_threshold_percent <= 99,
        "Capacity threshold must be 50–99%"
    );
    anyhow::ensure!(
        chrono::DateTime::parse_from_rfc3339(&rule.next_due_at).is_ok(),
        "Invalid next-run timestamp"
    );
    if rule.scope.mode == "selected" {
        anyhow::ensure!(
            !rule.scope.roots.is_empty()
                && rule.scope.roots.len() <= 32
                && rule
                    .scope
                    .roots
                    .iter()
                    .all(|p| std::path::Path::new(p).is_absolute()),
            "Select 1–32 absolute folder paths"
        );
    }
    c.execute(
        "INSERT INTO maintenance VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
        [serde_json::to_string(rule)?],
    )?;
    Ok(())
}
pub fn event(
    c: &Connection,
    status: &str,
    message: String,
    scan_id: Option<String>,
) -> Result<MaintenanceEvent> {
    let e = MaintenanceEvent {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: now(),
        status: status.into(),
        message,
        scan_id,
    };
    records::save(
        c,
        "maintenance",
        &e.id,
        &e.created_at,
        &e.status,
        "Maintenance",
        &e,
    )?;
    Ok(e)
}
/// Reserve one due occurrence before any work; missed occurrences coalesce on reopen.
pub fn due(c: &mut Connection) -> Result<Option<(MaintenanceRule, MaintenanceEvent)>> {
    let tx = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let Some(mut r) = get(&tx)? else {
        return Ok(None);
    };
    if !r.enabled
        || chrono::DateTime::parse_from_rfc3339(&r.next_due_at)?.with_timezone(&chrono::Utc)
            > chrono::Utc::now()
    {
        return Ok(None);
    }
    r.next_due_at =
        (chrono::Utc::now() + chrono::Duration::days(r.interval_days.into())).to_rfc3339();
    save(&tx, &r)?;
    let e=event(&tx,if r.mode=="scan"{"running"}else{"reminder"},if r.mode=="scan"{"Scheduled read-only analysis started. Cleanup always needs manual review."}else{"Review storage and developer artifacts. Open Optimization after a fresh scan; no cleanup has run."}.into(),None)?;
    tx.commit()?;
    Ok(Some((r, e)))
}
pub fn finish(
    c: &Connection,
    mut e: MaintenanceEvent,
    status: &str,
    scan_id: Option<String>,
    message: String,
) -> Result<MaintenanceEvent> {
    e.status = status.into();
    e.scan_id = scan_id;
    e.message = message;
    records::save(
        c,
        "maintenance",
        &e.id,
        &e.created_at,
        &e.status,
        "Maintenance",
        &e,
    )?;
    Ok(e)
}
