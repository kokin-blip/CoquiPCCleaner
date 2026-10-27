//! Explain retained measurements and review destinations, never authorize changes.
use crate::{baseline, database, model::*, records};
use anyhow::Result;
use rusqlite::Connection;

fn action(
    id: &str,
    title: &str,
    explanation: &str,
    destination: &str,
    priority: u32,
    workloads: &[String],
    evidence: Vec<DiagnosticEvidence>,
) -> RecommendationAction {
    RecommendationAction {
        id: id.into(),
        title: title.into(),
        explanation: explanation.into(),
        priority,
        classification: "Informational".into(),
        affected_workloads: workloads.to_vec(),
        expected_benefit: "Clarify the cause before deciding whether a change is warranted.".into(),
        confidence: "Further measurement needed".into(),
        risk: "Read-only review".into(),
        reversibility: "Opening this review changes no inspected files or system settings.".into(),
        permission: "Existing readable scope; denied locations remain unavailable.".into(),
        restart: "Not required for this review.".into(),
        verification: "Repeat the same task under comparable conditions; record its actual result."
            .into(),
        destination: destination.into(),
        evidence,
    }
}
fn measured(
    label: &str,
    value: String,
    unit: &str,
    at: &str,
    qualification: &str,
) -> DiagnosticEvidence {
    let mut e = DiagnosticEvidence::observed(label, value, unit, "Retained workload baseline");
    e.observed_at = at.into();
    e.qualification = qualification.into();
    e
}

/// Reading a plan performs no collection, record write, housekeeping or filesystem inspection.
pub fn build(
    c: &Connection,
    session_id: &str,
    baseline_id: Option<&str>,
) -> Result<RecommendationPlan> {
    build_with_trial(c, session_id, baseline_id, None)
}
pub fn build_with_trial(
    c: &Connection,
    session_id: &str,
    baseline_id: Option<&str>,
    trial_id: Option<&str>,
) -> Result<RecommendationPlan> {
    let s = database::session(c, session_id)?;
    anyhow::ensure!(
        s.status != "running",
        "Wait for analysis to finish before reviewing a plan"
    );
    let settings = database::settings(c)?;
    let workloads = &settings.workload.workloads;
    let preference = &settings.workload.preference;
    let b = baseline_id.map(|id| baseline::get(c, id)).transpose()?;
    let trial = trial_id
        .map(|id| crate::trials::completed(c, id))
        .transpose()?;
    if let Some(b) = &b {
        anyhow::ensure!(
            b.status == "complete" && b.samples.len() >= 3,
            "Choose a completed baseline with at least three samples"
        );
        anyhow::ensure!(
            b.samples.iter().all(|m| m.cpu_percent.is_finite()
                && m.used_memory_bytes.is_finite()
                && m.swap_bytes.is_finite()),
            "Baseline has invalid readings"
        );
    }
    let mut p = RecommendationPlan {
        id: uuid::Uuid::new_v4().to_string(), created_at: now(), session_id: s.id.clone(),
        baseline_id: b.as_ref().map(|b| b.id.clone()), trial_id: trial.as_ref().map(|t| t.id.clone()), preference: preference.clone(),
        actions: vec![], workloads: vec![], bottlenecks: vec![],
        qualification: format!("Review of saved scan {} ({}) and {}. These are observations and investigation leads, not confirmed causes, current readings, executable approvals or performance promises. CPU ≥90% in at least half the selected baseline's samples is a review threshold, not a benchmark. Missing GPU, thermal, pressure and physical-throughput measurements remain unavailable.",
            s.started_at, s.status, b.as_ref().map(|b| format!("explicitly selected baseline {} ({})", b.request.workload, b.created_at)).unwrap_or_else(|| "no selected workload baseline".into())),
    };
    if s.status != "complete" || s.skipped_count > 0 || !s.errors.is_empty() {
        let mut e = DiagnosticEvidence::observed(
            "Scan coverage",
            format!("{}; {} omitted entries", s.status, s.skipped_count),
            "",
            "Retained metadata scan",
        );
        e.observed_at = s.started_at.clone();
        e.qualification = "Complete traversal and comprehensive access are different. Counts do not include unreadable or deliberately skipped contents.".into();
        let mut a = action(
            "coverage",
            "Review incomplete coverage",
            "Inspect omitted locations before drawing whole-machine conclusions.",
            "System Analysis",
            1,
            workloads,
            vec![e],
        );
        a.confidence = "Observed coverage limitation".into();
        p.actions.push(a);
    }
    if let Some(m) = &s.snapshot {
        if let Some(d) = m
            .disks
            .iter()
            .filter(|d| d.mount == "/" || d.mount == "/System/Volumes/Data")
            .find(|d| {
                d.total_bytes.is_finite()
                    && d.available_bytes.is_finite()
                    && d.total_bytes > 0.
                    && d.available_bytes >= 0.
                    && d.available_bytes / d.total_bytes < 0.1
            })
        {
            let mut e = DiagnosticEvidence::observed(
                "Filesystem capacity available",
                format!("{:.1}", 100. * d.available_bytes / d.total_bytes),
                "%",
                "Saved macOS filesystem statistics",
            );
            e.observed_at = m.observed_at.clone();
            e.qualification = format!(
                "{}; APFS shared capacity is not additive or guaranteed reclaimable space.",
                d.mount
            );
            let mut a = action("capacity", "Review storage capacity", "Less than 10% was available on the recorded internal filesystem. Inspect contributing folders and separately review any removal.", "Storage", 2, workloads, vec![e.clone()]);
            a.expected_benefit = "Identify the largest relevant footprints; no recovered-byte or speed improvement is promised.".into();
            a.confidence = "Observed capacity; performance impact unconfirmed".into();
            a.classification = "Requires careful review".into();
            a.risk = "Inspection is read-only. Selected files may contain unique work; removal needs its own approval.".into();
            a.verification = "After an independently approved change, rescan and compare current filesystem capacity; Trash may retain allocation.".into();
            p.actions.push(a);
            p.bottlenecks.push(BottleneckAssessment { subsystem: "Storage capacity".into(), workload: workloads.join(", "), observation: "Low recorded free capacity".into(), confidence: "Observed capacity; not a confirmed performance bottleneck".into(), possible_impact: "Tasks needing temporary workspace may be constrained; actual task behavior is unmeasured.".into(), next_test: "Reproduce the affected task and record its output, available capacity and comparable baseline.".into(), evidence: vec![e] });
        }
        if !m.startup_items.is_empty() {
            let mut e = DiagnosticEvidence::observed(
                "Readable startup configurations",
                m.startup_items.len().to_string(),
                "entries",
                "Saved launch configuration inventory",
            );
            e.observed_at = m.observed_at.clone();
            e.qualification = "Configuration is not runtime enabled state, complete login-item coverage or measured startup delay.".into();
            let mut a = action("startup", "Review startup and background applications", "Inspect applications you recognize in Login Items and record a manual plan if a change is appropriate.", "Performance", if preference == "Efficiency" { 3 } else { 6 }, workloads, vec![e]);
            a.classification = "Advanced / manual".into();
            a.permission = "macOS controls the manual setting and any authorization.".into();
            a.reversibility = "Use your recorded previous setting in System Settings; Coqui has no automatic startup rollback.".into();
            a.verification = "Measure the same login stage separately; a plist change does not prove an improvement.".into();
            p.actions.push(a);
        }
    }
    let cpu = b.as_ref().map(|b| {
        let high = b.samples.iter().filter(|s| s.cpu_percent >= 90.).count();
        let e = measured("CPU samples at or above 90%", format!("{high}/{}", b.samples.len()), "samples", &b.created_at, "System-wide CPU activity during this interval; task completion time and causality are unmeasured.");
        (high * 2 >= b.samples.len(), e)
    });
    if let (Some(b), Some((high, e))) = (&b, &cpu) {
        p.bottlenecks.push(BottleneckAssessment { subsystem: "CPU activity".into(), workload: b.request.workload.clone(), observation: if *high { "Sustained high CPU activity" } else { "No sustained high CPU activity at the review threshold" }.into(), confidence: "Observed interval; limiting component unconfirmed".into(), possible_impact: "High use may be productive work. Low system-average use does not exclude a single-thread, GPU, I/O or application limit.".into(), next_test: format!("Repeat '{}' with the same inputs and measure its completion time; compare a controlled change.", b.request.target), evidence: vec![e.clone()] });
        if *high {
            let mut a = action("cpu", "Investigate the measured workload", "Identify the app and task behind sustained CPU activity before choosing a setting or hardware change.", "Performance", if preference == "Performance" { 2 } else { 4 }, &[b.request.workload.clone()], vec![e.clone()]);
            a.confidence = "Observed CPU activity; cause and improvement unverified".into();
            p.actions.push(a);
        }
        let first = b.samples.first().unwrap();
        let last = b.samples.last().unwrap();
        let delta = last.swap_bytes - first.swap_bytes;
        let e = measured("Swap change during baseline", format!("{delta:.0}"), "bytes", &b.created_at, "Swap growth is an investigation lead, not an operating-system pressure reading or proof that memory caused a delay.");
        p.bottlenecks.push(BottleneckAssessment { subsystem: "Memory and swap".into(), workload: b.request.workload.clone(), observation: if delta > 0. { "Swap use increased during the interval" } else { "No net increase in recorded swap use" }.into(), confidence: "Observed counters; memory pressure unmeasured".into(), possible_impact: "Used memory alone does not establish a constraint. Compression, pressure, application demand and task timing need context.".into(), next_test: "Reproduce the task while checking Activity Monitor's Memory Pressure, then repeat with fewer optional applications.".into(), evidence: vec![e.clone()] });
        if delta > 0. {
            let mut a = action("memory", "Check memory pressure during the task", "Swap increased in the selected baseline. Compare the same task with Activity Monitor and a controlled application workload.", "Performance", 4, &[b.request.workload.clone()], vec![e]);
            a.confidence = "Observed swap growth; pressure and cause unconfirmed".into();
            p.actions.push(a);
        }
    } else if trial.is_none() {
        p.actions.push(action("baseline", "Measure a representative workload", "Capture the task, expected result and conditions before diagnosing a bottleneck or choosing an upgrade.", "Performance", if preference == "Performance" { 2 } else { 5 }, workloads, vec![]));
    }
    if let Some(trial) = &trial {
        let elapsed = trial.elapsed_seconds.unwrap();
        let met = elapsed <= trial.request.target_seconds;
        let mut a = action("task", if met { "Repeat the measured task" } else { "Investigate the measured task delay" },
            &format!("'{}' took {:.3} seconds against a {:.3}-second target. Completion was user-reported in this single interval; repeat before judging a configuration or upgrade.", trial.request.task, elapsed, trial.request.target_seconds),
            "Performance", 3, &[trial.request.workload.clone()], crate::trials::evidence(trial));
        a.confidence = "Measured interval; user-reported task completion; one trial".into();
        a.expected_benefit =
            "Establish whether repeated runs meet your task target under comparable conditions."
                .into();
        p.actions.push(a);
        p.qualification.push_str(&format!(" Explicitly selected task trial {} ({}) uses user-controlled timing and declared conditions; task output and controls were not verified.", trial.id, trial.created_at));
    }
    for workload in workloads {
        if let Some(trial) = trial.as_ref().filter(|t| {
            t.request
                .workload
                .trim()
                .eq_ignore_ascii_case(workload.trim())
        }) {
            let met = trial.elapsed_seconds.unwrap() <= trial.request.target_seconds;
            p.workloads.push(WorkloadAssessment { workload: workload.clone(), status: if met { "Task met recorded target" } else { "Task exceeded recorded target" }.into(),
                explanation: format!("One user-reported completion of '{}': {:.3}s, declared target {:.3}s. This interval does not establish all-workload suitability, limiting hardware or upgrade benefit.", trial.request.task, trial.elapsed_seconds.unwrap(), trial.request.target_seconds),
                evidence: crate::trials::evidence(trial), next_test: "Repeat the same task and inputs at least three times per comparison group; inspect versions, power and background conditions.".into() });
            continue;
        }
        let matching = b.as_ref().filter(|b| {
            b.request
                .workload
                .trim()
                .eq_ignore_ascii_case(workload.trim())
        });
        p.workloads.push(WorkloadAssessment {
            workload: workload.clone(), status: if matching.is_some() { "Representative interval recorded" } else { "Needs representative measurement" }.into(),
            explanation: matching.map(|b| format!("Recorded task: {}. {}. Whether it meets your expectation still requires the actual task result; hardware age or low average CPU is not a suitability verdict.", b.request.target, b.request.context)).unwrap_or_else(|| "No explicitly selected completed baseline matches this workload. Installed specifications alone do not establish performance suitability.".into()),
            evidence: matching.map(|_| cpu.as_ref().map(|(_,e)| vec![e.clone()]).unwrap_or_default()).unwrap_or_default(),
            next_test: "Capture a named representative task, measure its result and compare it with your own target.".into(),
        });
    }
    if !s.candidates.is_empty() {
        let mut a = action("cache", "Inspect eligible cache candidates", "Review each origin and consequence; folder recognition, size and age do not establish disposability.", "Optimization", 7, workloads, vec![]);
        a.classification = "Requires careful review".into();
        a.risk = "Caches may be needed offline or during work. Exact-path review and fresh execution checks remain mandatory.".into();
        a.expected_benefit = "Find separately reviewable cache files; allocated size is not guaranteed recoverable capacity.".into();
        a.reversibility = "Eligible moves use Trash; recovery depends on the retained item and supported recovery procedure.".into();
        p.actions.push(a);
    }
    let mut power = action("power", "Compare native power preferences", &format!("Your {preference} preference determines which tradeoffs to review; only modes offered by this exact Mac are relevant."), "Performance", if preference == "Efficiency" { 2 } else { 8 }, workloads, b.as_ref().map(|b| b.power.clone()).unwrap_or_default());
    power.classification = "Advanced / manual".into();
    power.expected_benefit = "Evaluate the responsiveness, sustained-work and energy tradeoff under the same task; no benefit is guaranteed.".into();
    power.reversibility = "Record the previous native setting before changing it manually; no automatic rollback is offered.".into();
    power.permission =
        "System Settings and model-specific mode availability; Coqui changes no power setting."
            .into();
    p.actions.push(power);
    p.actions.sort_by_key(|a| (a.priority, a.id.clone()));
    Ok(p)
}

pub fn save(
    c: &Connection,
    session_id: &str,
    baseline_id: Option<&str>,
) -> Result<RecommendationPlan> {
    let p = build(c, session_id, baseline_id)?;
    records::save(
        c,
        "recommendationPlan",
        &p.id,
        &p.created_at,
        "review",
        "Workload review plan",
        &p,
    )?;
    Ok(p)
}
pub fn save_with_trial(
    c: &Connection,
    session_id: &str,
    baseline_id: Option<&str>,
    trial_id: Option<&str>,
) -> Result<RecommendationPlan> {
    let p = build_with_trial(c, session_id, baseline_id, trial_id)?;
    records::save(
        c,
        "recommendationPlan",
        &p.id,
        &p.created_at,
        "review",
        "Workload review plan",
        &p,
    )?;
    Ok(p)
}
