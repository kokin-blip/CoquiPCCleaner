//! Saved-evidence comparisons and local notices. This module launches no utilities/network requests.
use crate::{model::*, records};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};

fn time(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}
fn local_evidence(r: &FirmwareReport, label: &str, value: &str) -> DiagnosticEvidence {
    let mut e = DiagnosticEvidence::observed(label, value.into(), "", "sw_vers / saved audit");
    e.observed_at = r.created_at.clone();
    if value.is_empty() || value == "Unavailable" {
        e.state = CapabilityState::Unknown;
    }
    e.qualification =
        "Saved observation at the audit time; no current-state or installation verdict.".into();
    e
}
fn row(
    property: &str,
    before: Option<DiagnosticEvidence>,
    after: Option<DiagnosticEvidence>,
    platform: bool,
) -> FirmwareDelta {
    let (state,explanation)=match (&before,&after) {
        _ if !platform=>("notComparable","Reported model/chip/architecture eligibility differs or is unconfirmed. No configuration change is inferred."),
        (Some(b),Some(a)) if b.state==CapabilityState::Supported && a.state==CapabilityState::Supported && b.unit==a.unit=>
            if b.value==a.value {("unchanged","The recorded values match. This does not establish integrity, successful installation or current state.")} else {("changed","The recorded values differ. Coqui does not infer which version is newer or that a firmware operation succeeded.")},
        (Some(_),Some(_))=>("notComparable","One or both observations are unavailable, restricted, manual-only or custom. Availability changes are not confirmed configuration changes."),
        _=>("notCollected","This property was not collected in both selected scopes. Missing evidence is not a healthy reading."),
    };
    FirmwareDelta {
        property: property.into(),
        before,
        after,
        state: state.into(),
        explanation: explanation.into(),
    }
}
fn update_evidence(r: &FirmwareReport) -> Option<DiagnosticEvidence> {
    if !r.request.updates {
        return None;
    }
    let stamp = r.update_checked_at.as_ref().unwrap_or(&r.created_at);
    let titles = r
        .updates
        .iter()
        .take(5)
        .map(|u| format!("{} {}", u.title, u.version))
        .collect::<Vec<_>>()
        .join("; ");
    let mut e = DiagnosticEvidence::observed(
        "Offered software",
        format!(
            "{} offering(s). {}{}",
            r.updates.len(),
            titles,
            if r.updates.len() > 5 {
                "; more in source audit"
            } else {
                ""
            }
        ),
        "",
        "softwareupdate --list / saved response",
    );
    e.observed_at = stamp.clone();
    if !["available", "noneOffered"].contains(&r.update_state.as_str())
        || r.update_checked_at.is_none()
    {
        e.state = CapabilityState::Unknown;
        e.value = format!(
            "{}: no complete macOS listing established. {} retained offering(s).",
            r.update_state,
            r.updates.len()
        );
    }
    e.qualification="An offering disappearing does not prove it was installed. Catalog, eligibility, deferral and connectivity can change listings.".into();
    Some(e)
}
pub fn compare(c: &Connection, before: &str, after: &str) -> Result<FirmwareComparison> {
    anyhow::ensure!(before != after, "Choose two different audits.");
    let b: FirmwareReport = records::get(c, "firmware", before)?;
    let a: FirmwareReport = records::get(c, "firmware", after)?;
    anyhow::ensure!(["complete","partial"].contains(&b.status.as_str()) && ["complete","partial"].contains(&a.status.as_str()),"Compare finished complete or partial audits; interrupted/cancelled runs remain inspectable separately.");
    anyhow::ensure!(
        time(&b.created_at)? < time(&a.created_at)?,
        "Choose the earlier audit as Before and a later audit as After."
    );
    let compatible = b.platform.eligible
        && a.platform.eligible
        && b.platform.manufacturer == a.platform.manufacturer
        && b.platform.model == a.platform.model
        && b.platform.chip == a.platform.chip
        && b.platform.architecture == a.platform.architecture;
    let mut rows = vec![];
    for (name, bv, av) in [
        (
            "macOS version",
            &b.platform.os_version,
            &a.platform.os_version,
        ),
        ("macOS build", &b.platform.os_build, &a.platform.os_build),
    ] {
        rows.push(row(
            name,
            Some(local_evidence(&b, name, bv)),
            Some(local_evidence(&a, name, av)),
            compatible,
        ));
    }
    for name in [
        "System firmware",
        "OS loader",
        "System Integrity Protection",
        "Authenticated root",
        "FileVault",
        "Startup security policy",
    ] {
        rows.push(row(
            name,
            b.evidence.iter().find(|e| e.label == name).cloned(),
            a.evidence.iter().find(|e| e.label == name).cloned(),
            compatible,
        ));
    }
    let mut offered = row(
        "Offered software",
        update_evidence(&b),
        update_evidence(&a),
        compatible,
    );
    if offered.state == "changed" || offered.state == "unchanged" {
        // All offerings determine equality, including those omitted from the compact display.
        let mut bk = b
            .updates
            .iter()
            .map(|u| format!("{}\0{}\0{}", u.label, u.title, u.version))
            .collect::<Vec<_>>();
        let mut ak = a
            .updates
            .iter()
            .map(|u| format!("{}\0{}\0{}", u.label, u.title, u.version))
            .collect::<Vec<_>>();
        bk.sort();
        ak.sort();
        offered.state = if bk == ak { "unchanged" } else { "changed" }.into();
        offered.explanation="Compare recorded macOS offerings only. A missing offering is not proof of installation, firmware inclusion or current eligibility.".into();
    }
    rows.push(offered);
    let report=FirmwareComparison{id:uuid::Uuid::new_v4().to_string(),created_at:now(),status:if compatible{"complete"}else{"partial"}.into(),before_report_id:b.id,after_report_id:a.id,before_observed_at:b.created_at,after_observed_at:a.created_at,before_platform:b.platform,after_platform:a.platform,comparable_platform:compatible,rows,qualification:"Comparison of retained observations only. Matching model/chip is not unique physical-device attestation. Different versions do not establish ordering, firmware integrity, successful installation/reboot/recovery or the cause of a change. Scope and permission gaps stay not comparable. Saved comparisons retain their own copy of these selected readings until retention/purge.".into()};
    records::save(
        c,
        "firmwareComparison",
        &report.id,
        &report.created_at,
        &report.status,
        "Firmware audit comparison",
        &report,
    )?;
    Ok(report)
}
pub fn preferences(c: &Connection) -> Result<FirmwarePreferences> {
    let json: Option<String> = c
        .query_row(
            "SELECT json FROM firmware_preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    json.map(|j| Ok(serde_json::from_str(&j)?))
        .unwrap_or_else(|| Ok(FirmwarePreferences::default()))
}
fn save(c: &Connection, p: &FirmwarePreferences) -> Result<()> {
    c.execute("INSERT INTO firmware_preferences VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(p)?])?;
    Ok(())
}
pub fn save_reminder(c: &Connection, enabled: bool, days: u32) -> Result<FirmwareCenter> {
    anyhow::ensure!(
        (1..=365).contains(&days),
        "Choose a review interval from 1 to 365 days."
    );
    let mut p = preferences(c)?;
    if !enabled {
        p.next_due_at = None;
    } else if !p.reminder_enabled || p.interval_days != days || p.next_due_at.is_none() {
        p.next_due_at = Some((Utc::now() + chrono::Duration::days(days.into())).to_rfc3339());
    }
    p.reminder_enabled = enabled;
    p.interval_days = days;
    save(c, &p)?;
    center(c)
}
fn latest(c: &Connection, usable: bool) -> Result<Option<FirmwareReport>> {
    let json:Option<String>=c.query_row("SELECT json FROM records WHERE kind='firmware' AND status IN ('complete','partial','cancelled','interrupted','failed') AND json_extract(json,'$.request.updates')=1 AND (?1=0 OR (json_extract(json,'$.updateState') IN ('available','partial','noneOffered') AND json_extract(json,'$.updateCheckedAt') IS NOT NULL AND status IN ('complete','partial'))) ORDER BY julianday(created_at) DESC,rowid DESC LIMIT 1",[usable as u32],|r|r.get(0)).optional()?;
    json.map(|j| Ok(serde_json::from_str(&j)?)).transpose()
}
pub fn center(c: &Connection) -> Result<FirmwareCenter> {
    center_at(c, Utc::now())
}
fn center_at(c: &Connection, at: DateTime<Utc>) -> Result<FirmwareCenter> {
    let p = preferences(c)?;
    let attempt = latest(c, false)?;
    let usable = latest(c, true)?;
    let mut notices = vec![];
    if let Some(r) =
        usable.filter(|r| !r.updates.is_empty() && p.dismissed_report_id.as_deref() != Some(&r.id))
    {
        let stamp = r.update_checked_at.as_deref().unwrap_or(&r.created_at);
        let age = time(stamp).map(|t| at - t).ok();
        let later_uncertain = attempt.as_ref().is_some_and(|x| x.id != r.id);
        let stale = age
            .is_none_or(|a| a < chrono::Duration::zero() || a >= chrono::Duration::hours(24))
            || later_uncertain
            || r.update_state == "partial";
        notices.push(FirmwareNotice{id:r.id.clone(),kind:"offeredUpdates".into(),title:"macOS software offerings to review".into(),message:format!("{} offering(s) in this saved macOS response. {} A listing can become stale immediately; it does not establish standalone firmware availability. Open the source audit or request a new consented check.",r.updates.len(),if later_uncertain{"A later attempt did not establish a replacement usable listing."}else if stale{"This listing is incomplete, over 24 hours old or has uncertain time."}else{"No current-state or latest-firmware verdict is claimed."}),report_id:Some(r.id),observed_at:stamp.into(),stale});
    }
    if p.reminder_enabled {
        if let Some(due) = &p.next_due_at {
            if time(due)? <= at {
                notices.push(FirmwareNotice{id:format!("firmware-review:{due}"),kind:"reviewReminder".into(),title:"Review firmware and update evidence".into(),message:"Your optional review reminder is due. Open Firmware & Security to select readings. This reminder never launches a utility or contacts Apple. Dismiss to schedule the next interval; missed time coalesces into this one reminder.".into(),report_id:None,observed_at:due.clone(),stale:false});
            }
        }
    }
    Ok(FirmwareCenter{preferences:p,notices,latest_attempt_at:attempt.as_ref().map(|r|r.created_at.clone()),latest_attempt_state:attempt.map(|r|if r.status=="cancelled" || r.status=="interrupted" || r.status=="failed" {r.status}else{r.update_state}).unwrap_or_else(||"notRequested".into()),qualification:"Local in-app notices from retained evidence and an optional review schedule, refreshed only while Coqui is open. No macOS notification permission, automatic network check, OS background agent or firmware operation. Dismissing a notice changes only Coqui's local record. Diagnostic purge removes evidence notices; the reminder preference/schedule is preserved and can be disabled here.".into()})
}
pub fn dismiss(c: &Connection, id: &str) -> Result<FirmwareCenter> {
    let current = center(c)?;
    let n = current.notices.iter().find(|n| n.id == id).ok_or_else(|| {
        anyhow::anyhow!("This notice changed or is no longer retained. Refresh the notices.")
    })?;
    let mut p = current.preferences;
    if n.kind == "reviewReminder" {
        p.next_due_at =
            Some((Utc::now() + chrono::Duration::days(p.interval_days.into())).to_rfc3339());
    } else {
        p.dismissed_report_id = n.report_id.clone();
    }
    save(c, &p)?;
    center(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    fn db() -> Connection {
        database::open(std::path::Path::new(":memory:")).unwrap()
    }
    fn audit(id: &str, stamp: &str) -> FirmwareReport {
        serde_json::from_value(serde_json::json!({"id":id,"createdAt":stamp,"finishedAt":stamp,"status":"complete","stage":"Finished","request":{"inventory":true,"security":true,"updates":true,"networkAcknowledged":true},"platform":{"manufacturer":"Apple","model":"Mac16,8","chip":"Apple M4 Pro","architecture":"aarch64","osVersion":"26.6.1","osBuild":"fixture","eligible":true,"qualification":"Fixture"},"evidence":[],"capabilities":[],"findings":[],"updates":[],"updateState":"noneOffered","updateCheckedAt":stamp,"errors":[],"qualification":"Fixture"})).unwrap()
    }
    fn put(c: &Connection, r: &FirmwareReport) {
        records::save(c, "firmware", &r.id, &r.created_at, &r.status, "Fixture", r).unwrap();
    }
    fn offer(r: &mut FirmwareReport, count: usize) {
        r.update_state = "available".into();
        r.updates = (0..count)
            .map(|i| FirmwareUpdate {
                label: format!("fixture-{i}"),
                title: format!("Fixture {i}"),
                version: "1".into(),
                restart: false,
                qualification: "Fixture only".into(),
            })
            .collect();
    }
    fn pair(c: &Connection) -> (FirmwareReport, FirmwareReport) {
        let b = audit("before", "2026-10-21T10:00:00Z");
        let a = audit("after", "2026-10-22T10:00:00Z");
        put(c, &b);
        put(c, &a);
        (b, a)
    }
    #[test]
    fn compares_values_without_certifying_manual_or_missing_properties() {
        let c = db();
        let (mut b, mut a) = pair(&c);
        let mut e = DiagnosticEvidence::observed("System firmware", "1".into(), "", "fixture");
        b.evidence.push(e.clone());
        e.value = "2".into();
        a.evidence.push(e);
        let mut manual = DiagnosticEvidence::observed(
            "Startup security policy",
            "Unavailable".into(),
            "",
            "fixture",
        );
        manual.state = CapabilityState::ManualOnly;
        b.evidence.push(manual.clone());
        a.evidence.push(manual);
        put(&c, &b);
        put(&c, &a);
        let d = compare(&c, &b.id, &a.id).unwrap();
        assert_eq!(d.rows[2].state, "changed");
        assert!(d.rows[2].explanation.contains("does not infer"));
        assert_eq!(d.rows[3].state, "notCollected");
        assert_eq!(d.rows[7].state, "notComparable");
        let restored: FirmwareComparison = records::get(&c, "firmwareComparison", &d.id).unwrap();
        assert_eq!(restored.before_report_id, b.id);
    }
    #[test]
    fn rejects_same_reversed_and_unfinished_audits() {
        let c = db();
        let (b, mut a) = pair(&c);
        assert!(compare(&c, &b.id, &b.id).is_err());
        assert!(compare(&c, &a.id, &b.id).is_err());
        a.status = "cancelled".into();
        put(&c, &a);
        assert!(compare(&c, &b.id, &a.id).is_err());
    }
    #[test]
    fn different_or_unconfirmed_platforms_remain_not_comparable() {
        let c = db();
        let (b, mut a) = pair(&c);
        a.platform.model = "Mac99,9".into();
        put(&c, &a);
        let d = compare(&c, &b.id, &a.id).unwrap();
        assert_eq!(d.status, "partial");
        assert!(d.rows.iter().all(|r| r.state == "notComparable"));
    }
    #[test]
    fn missing_permission_is_not_a_configuration_change() {
        let c = db();
        let (mut b, mut a) = pair(&c);
        let e = DiagnosticEvidence::observed("FileVault", "On".into(), "", "fixture");
        b.evidence.push(e.clone());
        let mut unavailable = e;
        unavailable.state = CapabilityState::RequiresPermission;
        unavailable.value = "Unavailable".into();
        a.evidence.push(unavailable);
        put(&c, &b);
        put(&c, &a);
        assert_eq!(
            compare(&c, &b.id, &a.id).unwrap().rows[6].state,
            "notComparable"
        );
    }
    #[test]
    fn full_update_set_is_compared_including_beyond_display_limit() {
        let c = db();
        let (mut b, mut a) = pair(&c);
        offer(&mut b, 6);
        a.updates = b.updates.clone();
        a.update_state = "available".into();
        a.updates[5].version = "2".into();
        put(&c, &b);
        put(&c, &a);
        let d = compare(&c, &b.id, &a.id).unwrap();
        assert_eq!(d.rows[8].state, "changed");
        assert!(d.rows[8].explanation.contains("not proof of installation"));
        a.update_state = "partial".into();
        put(&c, &a);
        assert_eq!(
            compare(&c, &b.id, &a.id).unwrap().rows[8].state,
            "notComparable"
        );
    }
    #[test]
    fn no_default_reminder_or_listing_is_inferred() {
        let c = db();
        let center = center(&c).unwrap();
        assert!(!center.preferences.reminder_enabled);
        assert!(center.notices.is_empty());
        assert_eq!(center.latest_attempt_state, "notRequested");
        assert!(save_reminder(&c, true, 0).is_err());
        assert!(save_reminder(&c, true, 366).is_err());
    }
    #[test]
    fn later_failed_check_preserves_qualified_source_then_complete_empty_replaces_it() {
        let c = db();
        let (mut b, mut a) = pair(&c);
        offer(&mut b, 2);
        put(&c, &b);
        a.update_state = "temporarilyUnavailable".into();
        put(&c, &a);
        let at = time("2026-10-22T10:01:00Z").unwrap();
        let d = center_at(&c, at).unwrap();
        assert_eq!(d.notices[0].report_id.as_deref(), Some("before"));
        assert!(d.notices[0].stale);
        assert!(d.notices[0].message.contains("later attempt"));
        a.update_state = "noneOffered".into();
        put(&c, &a);
        assert!(center_at(&c, at).unwrap().notices.is_empty());
    }
    #[test]
    fn stale_partial_and_future_timestamps_are_qualified() {
        let c = db();
        let mut r = audit("listing", "2026-10-22T10:00:00Z");
        offer(&mut r, 1);
        put(&c, &r);
        for at in ["2026-10-23T10:00:00Z", "2026-10-21T10:00:00Z"] {
            assert!(center_at(&c, time(at).unwrap()).unwrap().notices[0].stale);
        }
        r.update_state = "partial".into();
        r.status = "partial".into();
        put(&c, &r);
        assert!(
            center_at(&c, time("2026-10-22T10:01:00Z").unwrap())
                .unwrap()
                .notices[0]
                .stale
        );
    }
    #[test]
    fn dismissal_persists_but_new_explicit_listing_gets_a_notice() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        let c = database::open(&path).unwrap();
        let mut r = audit("listing", "2026-10-22T10:00:00Z");
        offer(&mut r, 1);
        put(&c, &r);
        assert!(dismiss(&c, &r.id).unwrap().notices.is_empty());
        drop(c);
        let c = database::open(&path).unwrap();
        assert!(center(&c).unwrap().notices.is_empty());
        r.id = "new-listing".into();
        r.created_at = "2026-10-22T11:00:00Z".into();
        put(&c, &r);
        assert_eq!(center(&c).unwrap().notices[0].id, r.id);
    }
    #[test]
    fn missed_reminders_coalesce_and_dismissal_schedules_one_future_interval() {
        let c = db();
        save_reminder(&c, true, 3).unwrap();
        let mut p = preferences(&c).unwrap();
        p.next_due_at = Some("2020-01-01T00:00:00Z".into());
        save(&c, &p).unwrap();
        assert_eq!(save_reminder(&c, true, 3).unwrap().notices.len(), 1);
        let d = center(&c).unwrap();
        assert_eq!(d.notices[0].kind, "reviewReminder");
        let next = dismiss(&c, &d.notices[0].id).unwrap();
        assert!(next.notices.is_empty());
        let delta = time(next.preferences.next_due_at.as_ref().unwrap()).unwrap() - Utc::now();
        assert!(delta.num_hours() >= 71 && delta.num_hours() <= 72);
        assert!(save_reminder(&c, false, 3)
            .unwrap()
            .preferences
            .next_due_at
            .is_none());
    }
    #[test]
    fn migration_and_purge_preserve_schedule_but_remove_evidence_and_dismissal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v4.db");
        let c = database::open(&path).unwrap();
        c.execute_batch("DROP TABLE firmware_preferences; PRAGMA user_version=4;")
            .unwrap();
        drop(c);
        let c = database::open(&path).unwrap();
        assert_eq!(
            c.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            8
        );
        let (b, a) = pair(&c);
        compare(&c, &b.id, &a.id).unwrap();
        save_reminder(&c, true, 8).unwrap();
        let mut p = preferences(&c).unwrap();
        p.dismissed_report_id = Some(b.id);
        save(&c, &p).unwrap();
        database::purge(&c).unwrap();
        let after = preferences(&c).unwrap();
        assert_eq!(after.next_due_at, p.next_due_at);
        assert!(after.reminder_enabled);
        assert!(after.dismissed_report_id.is_none());
        assert!(records::list(&c, None, 0).unwrap().is_empty());
    }
    #[test]
    fn retained_comparison_survives_source_expiry_and_old_json_is_readable() {
        let c = db();
        let (b, a) = pair(&c);
        assert_eq!(b.platform.model_name, "");
        let d = compare(&c, &b.id, &a.id).unwrap();
        c.execute("DELETE FROM records WHERE kind='firmware'", [])
            .unwrap();
        let restored: FirmwareComparison = records::get(&c, "firmwareComparison", &d.id).unwrap();
        assert_eq!(restored.rows.len(), 9);
        assert!(center(&c).unwrap().notices.is_empty());
    }
}
