//! User-operated task timing. No task command, document, application or setting is executed.
use crate::{baseline, model::*, platform, records};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use sysinfo::System;

pub struct TrialControl {
    pub id: String,
    finished: Mutex<Option<Instant>>,
}
impl Default for TrialControl {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            finished: Mutex::new(None),
        }
    }
}
impl TrialControl {
    pub fn finish(&self) -> Result<()> {
        let mut at = self
            .finished
            .lock()
            .map_err(|_| anyhow::anyhow!("Task timer lock failed"))?;
        anyhow::ensure!(at.is_none(), "Task completion was already reported");
        *at = Some(Instant::now());
        Ok(())
    }
    fn finished_at(&self) -> Result<Option<Instant>> {
        Ok(*self
            .finished
            .lock()
            .map_err(|_| anyhow::anyhow!("Task timer lock failed"))?)
    }
}
pub fn validate(request: &WorkloadTrialRequest) -> Result<()> {
    for text in [
        &request.workload,
        &request.task,
        &request.inputs,
        &request.application,
        &request.conditions,
    ] {
        anyhow::ensure!(!text.trim().is_empty() && text.len() <= 2000, "Describe the workload, task, inputs, application/version and conditions (up to 2,000 characters each)");
    }
    anyhow::ensure!(
        request.target_seconds.is_finite()
            && request.target_seconds > 0.
            && request.target_seconds <= 86400.,
        "Choose a positive target of at most 86,400 seconds"
    );
    Ok(())
}
fn save(c: &Connection, trial: &WorkloadTrial) -> Result<()> {
    records::save(
        c,
        "workloadTrial",
        &trial.id,
        &trial.created_at,
        &trial.status,
        &trial.request.task,
        trial,
    )
}
pub fn capture(
    c: &Connection,
    request: WorkloadTrialRequest,
    control: &TrialControl,
    cancel: &AtomicBool,
    emit: impl FnMut(WorkloadTrial) -> bool,
) -> Result<WorkloadTrial> {
    validate(&request)?;
    // Preparation precedes the Running event. The user starts their task after that event.
    anyhow::ensure!(
        !cancel.load(Ordering::SeqCst),
        "Task timing cancelled before preparation"
    );
    let power = baseline::power();
    let mut system = System::new_all();
    capture_for(
        c,
        request,
        control,
        cancel,
        emit,
        Duration::from_secs(600),
        power,
        &mut system,
    )
}
fn capture_for(
    c: &Connection,
    request: WorkloadTrialRequest,
    control: &TrialControl,
    cancel: &AtomicBool,
    mut emit: impl FnMut(WorkloadTrial) -> bool,
    limit: Duration,
    power: Vec<DiagnosticEvidence>,
    system: &mut System,
) -> Result<WorkloadTrial> {
    let mut trial = WorkloadTrial {
        id: control.id.clone(), created_at: now(), finished_at: None, status: "running".into(),
        request, elapsed_seconds: Some(0.), completion_reported: false, samples: vec![], power,
        qualification: "User-operated task timing, capped at ten minutes while Coqui is open. Coqui measures the interval between Running and receiving your completion report; human start/finish latency is included. Coqui does not run or verify the task or its output. Two-second CPU/memory samples describe activity, not causality, memory pressure, hardware wear or a benchmark score. Cancelled, timed-out and interrupted trials do not establish completion.".into(),
    };
    let start = Instant::now();
    let result = (|| -> Result<()> {
        save(c, &trial)?;
        if !emit(trial.clone()) {
            cancel.store(true, Ordering::SeqCst);
        }
        let mut next_sample = Duration::from_secs(2);
        let mut next_event = Duration::from_secs(1);
        loop {
            let elapsed = start.elapsed();
            if cancel.load(Ordering::SeqCst) || elapsed >= limit || control.finished_at()?.is_some()
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
            if cancel.load(Ordering::SeqCst) || control.finished_at()?.is_some() {
                break;
            }
            let elapsed = start.elapsed();
            if elapsed >= limit {
                break;
            }
            if elapsed >= next_sample {
                trial.samples.push(platform::metrics(system));
                next_sample = start.elapsed() + Duration::from_secs(2);
            }
            if elapsed >= next_event {
                trial.elapsed_seconds = Some(start.elapsed().as_secs_f64());
                save(c, &trial)?;
                if !emit(trial.clone()) {
                    cancel.store(true, Ordering::SeqCst);
                }
                next_event = start.elapsed() + Duration::from_secs(1);
            }
        }
        let completed_at = control.finished_at()?;
        let completed_elapsed = completed_at.and_then(|at| at.checked_duration_since(start));
        trial.status = if cancel.load(Ordering::SeqCst) {
            "cancelled"
        } else if completed_elapsed.is_some_and(|elapsed| elapsed < limit) {
            "complete"
        } else {
            "timedOut"
        }
        .into();
        trial.completion_reported = trial.status == "complete";
        trial.elapsed_seconds = Some(
            if trial.completion_reported {
                completed_elapsed.unwrap()
            } else {
                start.elapsed()
            }
            .as_secs_f64(),
        );
        trial.finished_at = Some(now());
        save(c, &trial)?;
        Ok(())
    })();
    if let Err(error) = result {
        trial.status = "failed".into();
        trial.completion_reported = false;
        trial.finished_at = Some(now());
        trial.elapsed_seconds = Some(start.elapsed().as_secs_f64());
        trial.qualification = format!(
            "Capture failed: {error}. Last observations do not establish task completion. {}",
            trial.qualification
        );
        let _ = save(c, &trial);
        emit(trial);
        return Err(error);
    }
    emit(trial.clone());
    Ok(trial)
}
pub fn completed(c: &Connection, id: &str) -> Result<WorkloadTrial> {
    let trial: WorkloadTrial = records::get(c, "workloadTrial", id)?;
    validate(&trial.request)?;
    anyhow::ensure!(
        trial.status == "complete"
            && trial.completion_reported
            && trial.finished_at.is_some()
            && trial
                .elapsed_seconds
                .is_some_and(|s| s.is_finite() && s > 0. && s < 600.),
        "Choose a completed task trial with an actual measured interval"
    );
    Ok(trial)
}
pub fn evidence(trial: &WorkloadTrial) -> Vec<DiagnosticEvidence> {
    let mut elapsed = match trial.elapsed_seconds.filter(|s| s.is_finite() && *s > 0.) {
        Some(seconds) => DiagnosticEvidence::observed(
            "Measured task interval",
            format!("{seconds:.3}"),
            "seconds",
            "Monotonic timer; user-controlled task boundaries",
        ),
        None => DiagnosticEvidence::unavailable(
            "Measured task interval",
            "No valid interval was recorded",
        ),
    };
    elapsed.observed_at = trial
        .finished_at
        .clone()
        .unwrap_or_else(|| trial.created_at.clone());
    elapsed.qualification = "Task completion is user-reported; human boundary latency is included. This is not an automated application benchmark.".into();
    let mut target = DiagnosticEvidence::observed(
        "Declared task target",
        trial.request.target_seconds.to_string(),
        "seconds or less",
        "User-specified task protocol",
    );
    target.observed_at = trial.created_at.clone();
    target.qualification = format!(
        "{}; inputs: {}; application/version: {}; conditions: {}",
        trial.request.task,
        trial.request.inputs,
        trial.request.application,
        trial.request.conditions
    );
    vec![elapsed, target]
}
pub fn compare(c: &Connection, request: TrialComparisonRequest) -> Result<TrialComparison> {
    anyhow::ensure!(
        !request.change.trim().is_empty() && request.change.len() <= 2000,
        "Describe the comparison or intended change (up to 2,000 characters)"
    );
    anyhow::ensure!(
        (1..=20).contains(&request.before_ids.len()) && (1..=20).contains(&request.after_ids.len()),
        "Select one to twenty trials in each group"
    );
    let ids = request
        .before_ids
        .iter()
        .chain(&request.after_ids)
        .collect::<HashSet<_>>();
    anyhow::ensure!(
        ids.len() == request.before_ids.len() + request.after_ids.len(),
        "Each trial may appear only once across the two groups"
    );
    let first = completed(c, &request.before_ids[0])?;
    let protocol = first.request;
    let mut matching_conditions = true;
    let mut target_matches = true;
    let mut group = |ids: &[String]| -> Result<TrialGroup> {
        let mut trials = vec![];
        for id in ids {
            let trial = completed(c, id)?;
            anyhow::ensure!(trial.request.workload == protocol.workload && trial.request.task == protocol.task && trial.request.inputs == protocol.inputs, "Compare the same workload, task and inputs; different tasks cannot share a timing comparison");
            matching_conditions &= trial.request.application == protocol.application
                && trial.request.conditions == protocol.conditions;
            target_matches &= trial.request.target_seconds == protocol.target_seconds;
            trials.push(TrialObservation {
                id: trial.id,
                created_at: trial.created_at,
                elapsed_seconds: trial.elapsed_seconds.unwrap(),
                target_met: trial.elapsed_seconds.unwrap() <= trial.request.target_seconds,
            });
        }
        let mut times = trials.iter().map(|t| t.elapsed_seconds).collect::<Vec<_>>();
        times.sort_by(f64::total_cmp);
        let mid = times.len() / 2;
        let median = if times.len() % 2 == 0 {
            (times[mid - 1] + times[mid]) / 2.
        } else {
            times[mid]
        };
        Ok(TrialGroup {
            target_met_count: trials.iter().filter(|t| t.target_met).count() as u32,
            trials,
            median_seconds: median,
            minimum_seconds: times[0],
            maximum_seconds: *times.last().unwrap(),
        })
    };
    let before = group(&request.before_ids)?;
    let after = group(&request.after_ids)?;
    let mut limitations = vec![];
    if !matching_conditions {
        limitations.push("Application/version or comparison conditions differ. These groups are descriptive; repeat with matching conditions before assessing a change.".into());
    }
    if !target_matches {
        limitations.push("Declared targets differ. Target-met counts use each trial's own target and are not a common success criterion.".into());
    }
    if before.trials.len() < 3 || after.trials.len() < 3 {
        limitations.push("Fewer than three trials in a group. Repeat the same task; a single interval does not establish a reliable change.".into());
    }
    let delta = 100. * (after.median_seconds - before.median_seconds) / before.median_seconds;
    Ok(TrialComparison { id: uuid::Uuid::new_v4().to_string(), created_at: now(), request, protocol, before, after, median_change_percent: delta, matching_conditions: matching_conditions && target_matches, limitations,
        qualification: "Measured elapsed intervals and user-reported completion only. Negative median change means a shorter recorded interval; it does not prove that the declared change caused it. Protocol/conditions are declarations, not verified controls. Thermal state, background work, caching and human timing can confound results. No workload-wide suitability, upgrade benefit, hardware lifespan or recovery claim is established.".into() })
}
pub fn save_comparison(c: &Connection, request: TrialComparisonRequest) -> Result<TrialComparison> {
    let comparison = compare(c, request)?;
    records::save(
        c,
        "trialComparison",
        &comparison.id,
        &comparison.created_at,
        "review",
        &comparison.protocol.task,
        &comparison,
    )?;
    Ok(comparison)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> WorkloadTrialRequest {
        WorkloadTrialRequest {
            workload: "Development".into(),
            task: "Disposable task".into(),
            inputs: "fixture".into(),
            application: "test 1".into(),
            conditions: "test process".into(),
            target_seconds: 5.,
        }
    }
    #[test]
    fn completion_is_timed_at_request_and_cancellation_wins() {
        let temp = tempfile::tempdir().unwrap();
        let c = crate::database::open(&temp.path().join("trial.db")).unwrap();
        let control = TrialControl::default();
        let cancel = AtomicBool::new(false);
        let trial = capture_for(
            &c,
            request(),
            &control,
            &cancel,
            |stage| {
                if stage.status == "running" {
                    control.finish().unwrap();
                }
                true
            },
            Duration::from_secs(1),
            vec![],
            &mut System::new(),
        )
        .unwrap();
        assert_eq!(trial.status, "complete");
        assert!(trial.elapsed_seconds.unwrap() > 0.);
        assert!(trial.samples.is_empty());
        assert!(control.finish().is_err());
        let control = TrialControl::default();
        let mut first = true;
        let trial = capture_for(
            &c,
            request(),
            &control,
            &cancel,
            |_| {
                if first {
                    first = false;
                    control.finish().unwrap();
                    cancel.store(true, Ordering::SeqCst);
                }
                true
            },
            Duration::from_secs(1),
            vec![],
            &mut System::new(),
        )
        .unwrap();
        assert_eq!(trial.status, "cancelled");
        assert!(!trial.completion_reported);
    }
    #[test]
    fn deadline_and_channel_loss_do_not_claim_task_completion() {
        let temp = tempfile::tempdir().unwrap();
        let c = crate::database::open(&temp.path().join("trial.db")).unwrap();
        let trial = capture_for(
            &c,
            request(),
            &TrialControl::default(),
            &AtomicBool::new(false),
            |_| true,
            Duration::from_millis(1),
            vec![],
            &mut System::new(),
        )
        .unwrap();
        assert_eq!(trial.status, "timedOut");
        assert!(!trial.completion_reported);
        let cancel = AtomicBool::new(false);
        let trial = capture_for(
            &c,
            request(),
            &TrialControl::default(),
            &cancel,
            |_| false,
            Duration::from_secs(1),
            vec![],
            &mut System::new(),
        )
        .unwrap();
        assert_eq!(trial.status, "cancelled");
        assert!(cancel.load(Ordering::SeqCst));
    }
}
