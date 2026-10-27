//! Pure summaries of retained observations. No collection or cleanup authority.
use crate::{database, model::*};
use anyhow::Result;
use rusqlite::{params, Connection};

/// Coqui's large-file review threshold uses indexed allocated bytes, not age.
pub const LARGE_FILE_THRESHOLD_BYTES: f64 = 1_073_741_824.;

fn scan_time(session: &DiagnosticSession) -> Option<String> {
    session
        .finished_at
        .as_ref()
        .filter(|value| !value.is_empty())
        .cloned()
}

fn category(
    session: &DiagnosticSession,
    name: &str,
    kind: ScanReviewKind,
    destination: ScanReviewDestination,
) -> ScanReviewGroup {
    // These persisted categories aggregate file rows, never cumulative folder
    // rows or candidate-directory sizes. Empty unfinished aggregates are unknown.
    let measured =
        !session.categories.is_empty() || (session.status == "complete" && session.file_count == 0);
    let found = session.categories.iter().find(|item| item.name == name);
    ScanReviewGroup {
        kind,
        state: if !measured {
            ScanReviewState::NotChecked
        } else if session.status == "complete" {
            ScanReviewState::Ready
        } else {
            ScanReviewState::Partial
        },
        source: ScanReviewSource::RetainedScanCategories,
        observed_at: measured.then(|| scan_time(session)).flatten(),
        count: measured.then(|| found.map_or(0, |item| item.file_count)),
        logical_bytes: measured.then(|| found.map_or(0., |item| item.logical_bytes)),
        allocated_bytes: measured.then(|| found.map_or(0., |item| item.allocated_bytes)),
        threshold_bytes: None,
        destination,
        qualification: format!(
            "{name} is a retained file-only scan category, not an eligible-removal or reclaimable-space total. {} {}",
            if name == "Development" {
                "Development includes multiple tools, not only Xcode."
            } else {
                "Cache and temporary paths can contain useful data."
            },
            if measured {
                "Counts and bytes describe the indexed scope at collection; coverage gaps and live-filesystem changes still apply. No fresh eligibility check was run."
            } else {
                "Category aggregation was not recorded; missing measurements are not zero."
            }
        ),
    }
}

fn snapshot_group(session: &DiagnosticSession, startup: bool) -> ScanReviewGroup {
    let snapshot = session.snapshot.as_ref();
    ScanReviewGroup {
        kind: if startup {
            ScanReviewKind::Startup
        } else {
            ScanReviewKind::Applications
        },
        state: if snapshot.is_some() {
            ScanReviewState::Partial
        } else {
            ScanReviewState::NotChecked
        },
        source: ScanReviewSource::RetainedSnapshot,
        observed_at: snapshot
            .map(|value| &value.observed_at)
            .filter(|value| !value.is_empty())
            .cloned(),
        count: snapshot.map(|value| {
            if startup {
                value.startup_items.len() as u32
            } else {
                value.applications.len() as u32
            }
        }),
        logical_bytes: None,
        allocated_bytes: None,
        threshold_bytes: None,
        destination: if startup {
            ScanReviewDestination::Startup
        } else {
            ScanReviewDestination::Applications
        },
        qualification: if snapshot.is_none() {
            "No retained foreground/system snapshot was recorded for this scan. This view does not collect missing observations or interpret them as zero.".into()
        } else if startup {
            "Retained readable on-disk launch configurations only. This is not complete modern login/background-item coverage, enabled state, measured startup delay or a recommendation to disable entries. No fresh collection was run.".into()
        } else {
            "Retained scan-snapshot application count, not a refreshed application inventory. The snapshot covers readable /Applications and user Applications folders up to three levels. Installed versions and update availability were not checked here.".into()
        },
    }
}

fn applications(c: &Connection, session: &DiagnosticSession) -> Result<ScanReviewGroup> {
    let mut query = c.prepare(
        "SELECT json_extract(json,'$.observedAt') FROM application_observations WHERE missing_at IS NULL",
    )?;
    let mut rows = query.query([])?;
    let mut count = 0u32;
    let mut all_dated = true;
    let mut latest = None;
    while let Some(row) = rows.next()? {
        count = count.checked_add(1).ok_or_else(|| {
            anyhow::anyhow!("Application observation count exceeds the supported range")
        })?;
        let observed: Option<String> = row.get(0)?;
        match observed.and_then(|value| chrono::DateTime::parse_from_rfc3339(&value).ok()) {
            Some(date) => {
                if latest.is_none_or(|previous| date > previous) {
                    latest = Some(date);
                }
            }
            None => all_dated = false,
        }
    }
    // An empty table is not evidence of a completed empty inventory. Legacy
    // undated observations cannot borrow the scan's observation time.
    if count == 0 || !all_dated || latest.is_none() {
        return Ok(snapshot_group(session, false));
    }
    Ok(ScanReviewGroup {
        kind: ScanReviewKind::Applications,
        state: ScanReviewState::Partial,
        source: ScanReviewSource::RetainedApplicationInventory,
        observed_at: latest.map(|date| date.to_rfc3339()),
        count: Some(count),
        logical_bytes: None,
        allocated_bytes: None,
        threshold_bytes: None,
        destination: ScanReviewDestination::Applications,
        qualification: "Retained application observations with no recorded missing path, independent of this scan. The date is the latest individual observation, not a complete-refresh timestamp. Older entries may remain; unreadable metadata and apps outside the three-level Applications scope can be absent. No paths, running state, versions or update availability were refreshed.".into(),
    })
}

fn foreground_group(
    c: &Connection,
    session: &DiagnosticSession,
    startup: bool,
) -> Result<ScanReviewGroup> {
    let Some(report) = crate::scan_observations::read(c, &session.id)? else {
        return if startup {
            Ok(snapshot_group(session, true))
        } else {
            applications(c, session)
        };
    };
    let module = if startup { "startup" } else { "applications" };
    let check = report.checks.iter().find(|check| check.module == module);
    let measured =
        check.is_some_and(|check| ["complete", "partial"].contains(&check.status.as_str()));
    Ok(ScanReviewGroup {
        kind: if startup { ScanReviewKind::Startup } else { ScanReviewKind::Applications },
        state: match check.map(|check| check.status.as_str()) {
            Some("complete" | "partial") => ScanReviewState::Partial,
            Some("notSelected" | "pending" | "running") => ScanReviewState::NotChecked,
            _ => ScanReviewState::Unavailable,
        },
        source: ScanReviewSource::RetainedScanObservations,
        observed_at: check.filter(|_| measured).and_then(|check| check.finished_at.clone()),
        count: check.filter(|_| measured).and_then(|check| check.count),
        logical_bytes: None, allocated_bytes: None, threshold_bytes: None,
        destination: if startup { ScanReviewDestination::Startup } else { ScanReviewDestination::Applications },
        qualification: check.map(|check| format!("This scan's {} check: {}. {} {} omissions. This view reads saved observations only.", module, check.status, check.qualification, check.omissions)).unwrap_or_else(|| "This scan has no recorded module result; missing measurements remain unavailable.".into()),
    })
}

fn large_files(c: &Connection, session: &DiagnosticSession) -> Result<ScanReviewGroup> {
    let indexed: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM files WHERE session_id=?1)",
        [&session.id],
        |row| row.get(0),
    )?;
    let measured = indexed || (session.status == "complete" && session.file_count == 0);
    let totals: Option<(u32, f64, f64)> = if measured {
        Some(c.query_row(
            "SELECT count(*),COALESCE(sum(logical),0),COALESCE(sum(allocated),0) FROM files INDEXED BY file_size WHERE session_id=?1 AND allocated>=?2 AND directory=0",
            params![session.id, LARGE_FILE_THRESHOLD_BYTES],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?)
    } else {
        None
    };
    Ok(ScanReviewGroup {
        kind: ScanReviewKind::LargeFiles,
        state: if !measured {
            if session.status == "running" {
                ScanReviewState::NotChecked
            } else {
                ScanReviewState::Unavailable
            }
        } else if session.status == "complete" {
            ScanReviewState::Ready
        } else {
            ScanReviewState::Partial
        },
        source: ScanReviewSource::RetainedFileIndex,
        observed_at: measured.then(|| scan_time(session)).flatten(),
        count: totals.map(|value| value.0),
        logical_bytes: totals.map(|value| value.1),
        allocated_bytes: totals.map(|value| value.2),
        threshold_bytes: Some(LARGE_FILE_THRESHOLD_BYTES),
        destination: ScanReviewDestination::LargeFiles,
        qualification: if measured {
            "Exact retained file-row totals at or above 1 GiB allocated (1,073,741,824 bytes). Logical size can differ. Folder rows are excluded; partial traversal/import can omit files. Size is not disposal authority, unique APFS usage or guaranteed recoverable capacity. No filesystem check was run.".into()
        } else {
            "The scan has no available retained file index for this measurement. Missing large-file counts and bytes are not zero; no collection was started.".into()
        },
    })
}

pub fn build(c: &Connection, session_id: &str) -> Result<ScanReview> {
    let session = database::session(c, session_id)?;
    Ok(ScanReview {
        session_id: session.id.clone(),
        scan_status: session.status.clone(),
        observed_at: scan_time(&session),
        groups: vec![
            category(
                &session,
                "Caches",
                ScanReviewKind::Caches,
                ScanReviewDestination::CacheFiles,
            ),
            category(
                &session,
                "Development",
                ScanReviewKind::Development,
                ScanReviewDestination::DeveloperArtifacts,
            ),
            foreground_group(c, &session, true)?,
            foreground_group(c, &session, false)?,
            large_files(c, &session)?,
        ],
    })
}
