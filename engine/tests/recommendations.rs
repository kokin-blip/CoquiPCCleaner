use coqui_engine::{database, model::*, recommendations, records};
use serde_json::json;

fn fixture() -> (tempfile::TempDir, rusqlite::Connection) {
    let temp = tempfile::tempdir().unwrap();
    let c = database::open(&temp.path().join("audit.db")).unwrap();
    let mut settings = database::settings(&c).unwrap();
    settings.workload.workloads = vec!["Software development".into(), "Gaming".into()];
    database::save_settings(&c, &settings).unwrap();
    let s: DiagnosticSession = serde_json::from_value(json!({
        "id":"scan", "startedAt":now(), "finishedAt":now(), "status":"partial", "stage":"Partial",
        "scope":"/fixture", "fileCount":12, "logicalBytes":100, "allocatedBytes":4096,
        "skippedCount":2, "snapshot":null, "findings":[], "permissions":[], "categories":[], "candidates":[], "errors":[], "coverage":[], "volumePlan":null
    })).unwrap();
    database::save_session(&c, &s).unwrap();
    (temp, c)
}
fn baseline(c: &rusqlite::Connection, workload: &str, cpus: &[f64], swaps: &[f64]) {
    let samples = cpus.iter().enumerate().map(|(i,cpu)| json!({
        "observedAt":now(),"cpuPercent":cpu,"usedMemoryBytes":1000.,"totalMemoryBytes":1000.,
        "swapBytes":swaps[i],"diskReadBytes":0.,"diskWrittenBytes":0.,"processes":[],"qualification":"Fixture; no pressure collector"
    })).collect::<Vec<_>>();
    let b: Baseline = serde_json::from_value(json!({
        "id":"baseline", "createdAt":now(), "finishedAt":now(), "status":"complete",
        "request":{"workload":workload,"target":"Build in under two minutes","context":"Same inputs","durationSeconds":6},
        "samples":samples, "power":[], "summary":[], "qualification":"Disposable measurement fixture"
    })).unwrap();
    records::save(
        c,
        "baseline",
        &b.id,
        &b.created_at,
        &b.status,
        "Fixture",
        &b,
    )
    .unwrap();
}

#[test]
fn read_plan_preserves_missing_measurements_and_writes_nothing() {
    let (_temp, c) = fixture();
    let before = c.total_changes();
    let p = recommendations::build(&c, "scan", None).unwrap();
    assert_eq!(before, c.total_changes());
    assert_eq!(p.actions[0].id, "coverage");
    assert!(p
        .workloads
        .iter()
        .all(|w| w.status == "Needs representative measurement"));
    assert!(p.bottlenecks.is_empty());
    assert!(p.qualification.contains("not confirmed causes"));
    assert!(records::list(&c, Some("recommendationPlan"), 0)
        .unwrap()
        .is_empty());
    for a in p.actions {
        assert!(
            !a.risk.is_empty()
                && !a.reversibility.is_empty()
                && !a.permission.is_empty()
                && !a.restart.is_empty()
                && !a.verification.is_empty()
                && !a.expected_benefit.is_empty()
        );
    }
}

#[test]
fn explicit_task_result_is_workload_specific_and_old_plan_stays_nullable() {
    use coqui_engine::advice;
    let (_temp, c) = fixture();
    let mut trial = WorkloadTrial {
        id: "trial".into(),
        created_at: now(),
        finished_at: Some(now()),
        status: "complete".into(),
        request: WorkloadTrialRequest {
            workload: "Software development".into(),
            task: "Build fixture".into(),
            inputs: "fixture revision".into(),
            application: "compiler 1".into(),
            conditions: "test only".into(),
            target_seconds: 10.,
        },
        elapsed_seconds: Some(8.),
        completion_reported: true,
        samples: vec![],
        power: vec![],
        qualification: "Synthetic test only".into(),
    };
    records::save(
        &c,
        "workloadTrial",
        &trial.id,
        &trial.created_at,
        &trial.status,
        "Fixture",
        &trial,
    )
    .unwrap();
    let plan = recommendations::build_with_trial(&c, "scan", None, Some("trial")).unwrap();
    assert_eq!(plan.trial_id.as_deref(), Some("trial"));
    assert_eq!(plan.workloads[0].status, "Task met recorded target");
    assert_eq!(plan.workloads[1].status, "Needs representative measurement");
    assert!(!plan.actions.iter().any(|a| a.id == "baseline"));
    let mut request = AdviceRequest {
        session_id: "scan".into(),
        baseline_id: None,
        trial_id: Some("trial".into()),
        workload: "Software development".into(),
        expectation: "A later changed target".into(),
        annual_growth_percent: 10.,
        current_demand_percent: 60.,
        quote_low: None,
        quote_high: None,
        quote_source: "".into(),
    };
    let advice = advice::assess(&c, request.clone()).unwrap();
    assert_eq!(advice.findings[0].kind, "taskOutcome");
    assert!(advice.findings[0].explanation.contains("10.000 seconds"));
    request.workload = "Gaming".into();
    assert!(advice::assess(&c, request).is_err());
    trial.status = "cancelled".into();
    records::save(
        &c,
        "workloadTrial",
        &trial.id,
        &trial.created_at,
        &trial.status,
        "Fixture",
        &trial,
    )
    .unwrap();
    assert!(recommendations::build_with_trial(&c, "scan", None, Some("trial")).is_err());
    let mut old = serde_json::to_value(plan).unwrap();
    old.as_object_mut().unwrap().remove("trialId");
    assert!(serde_json::from_value::<RecommendationPlan>(old)
        .unwrap()
        .trial_id
        .is_none());
}

#[test]
fn high_cpu_and_swap_growth_are_workload_leads_not_confirmed_causes() {
    let (_temp, c) = fixture();
    baseline(
        &c,
        "Software development",
        &[95., 92., 98., 40.],
        &[0., 10., 20., 30.],
    );
    let p = recommendations::build(&c, "scan", Some("baseline")).unwrap();
    assert!(p.actions.iter().any(|a| a.id == "cpu"));
    assert!(p.actions.iter().any(|a| a.id == "memory"));
    assert_eq!(p.workloads[0].status, "Representative interval recorded");
    assert_eq!(p.workloads[1].status, "Needs representative measurement");
    assert!(p
        .bottlenecks
        .iter()
        .all(|b| !b.next_test.is_empty() && b.confidence.contains("un")));
    assert!(p
        .bottlenecks
        .iter()
        .find(|b| b.subsystem == "Memory and swap")
        .unwrap()
        .confidence
        .contains("pressure unmeasured"));
    assert!(p
        .actions
        .iter()
        .find(|a| a.id == "cpu")
        .unwrap()
        .evidence
        .iter()
        .all(|e| e.observed_at != p.created_at));
}

#[test]
fn low_cpu_and_high_used_memory_do_not_prove_suitability_or_pressure() {
    let (_temp, c) = fixture();
    baseline(&c, "An unrelated task", &[10., 15., 12.], &[40., 40., 40.]);
    let p = recommendations::build(&c, "scan", Some("baseline")).unwrap();
    assert!(!p.actions.iter().any(|a| a.id == "cpu" || a.id == "memory"));
    assert!(p
        .workloads
        .iter()
        .all(|w| w.status == "Needs representative measurement"));
    assert!(p.bottlenecks[0].possible_impact.contains("single-thread"));
}

#[test]
fn preference_changes_review_order_not_measurement_or_authority() {
    let (_temp, c) = fixture();
    let balanced = recommendations::build(&c, "scan", None).unwrap();
    let mut settings = database::settings(&c).unwrap();
    settings.workload.preference = "Efficiency".into();
    database::save_settings(&c, &settings).unwrap();
    let efficiency = recommendations::build(&c, "scan", None).unwrap();
    assert_eq!(balanced.actions[1].id, "baseline");
    assert_eq!(efficiency.actions[1].id, "power");
    assert_eq!(efficiency.actions[1].classification, "Advanced / manual");
    assert!(efficiency.actions[1]
        .permission
        .contains("Coqui changes no power setting"));
}

#[test]
fn explicit_saved_plan_survives_reopen_and_rejects_incomplete_baseline() {
    let (temp, c) = fixture();
    baseline(&c, "Gaming", &[10., 15.], &[0., 0.]);
    assert!(recommendations::build(&c, "scan", Some("baseline")).is_err());
    let p = recommendations::save(&c, "scan", None).unwrap();
    drop(c);
    let c = database::open(&temp.path().join("audit.db")).unwrap();
    let saved: RecommendationPlan = records::get(&c, "recommendationPlan", &p.id).unwrap();
    assert_eq!(saved.session_id, "scan");
    assert_eq!(
        records::list(&c, Some("recommendationPlan"), 0).unwrap()[0].status,
        "review"
    );
}

#[test]
fn requested_investigation_tasks_are_streamed_and_cancel_without_hidden_collection() {
    use coqui_engine::assistant;
    use std::sync::atomic::{AtomicBool, Ordering};
    let (_temp, c) = fixture();
    let i = assistant::ask(&c, None, Some("scan"), "Review storage").unwrap();
    let cancel = AtomicBool::new(false);
    let result = assistant::collect(
        &c,
        &i.id,
        InvestigationCollection::Baseline(BaselineRequest {
            workload: "Coding".into(),
            target: "Compile in two minutes".into(),
            context: "Same inputs".into(),
            duration_seconds: 6,
        }),
        &[],
        &cancel,
        |stage| {
            assert_eq!(stage.id, i.id);
            false
        },
    )
    .unwrap();
    assert!(cancel.load(Ordering::SeqCst));
    assert_eq!(result.status, "cancelled");
    assert!(result.baseline_id.is_none());
    assert!(records::list(&c, Some("baseline"), 0).unwrap().is_empty());
    let reopened: Investigation = records::get(&c, "investigation", &i.id).unwrap();
    assert_eq!(reopened.tasks.last().unwrap().status, "cancelled");
    assert!(serde_json::from_value::<InvestigationCollection>(
        json!({"kind":"shell","request":"rm"})
    )
    .is_err());
}

#[test]
fn task_errors_are_retained_and_unrelated_baselines_are_not_silently_adopted() {
    use coqui_engine::assistant;
    use std::sync::atomic::AtomicBool;
    let (_temp, c) = fixture();
    baseline(&c, "Unrelated workload", &[95., 95., 95.], &[0., 0., 0.]);
    let i = assistant::ask(&c, None, Some("scan"), "My coding work is slow").unwrap();
    assert!(i.tasks.iter().all(|t| t.evidence.is_empty()));
    let result = assistant::collect(
        &c,
        &i.id,
        InvestigationCollection::Baseline(BaselineRequest {
            workload: "Coding".into(),
            target: "Compile".into(),
            context: "Same inputs".into(),
            duration_seconds: 1,
        }),
        &[],
        &AtomicBool::new(false),
        |_| true,
    )
    .unwrap();
    assert_eq!(result.status, "failed");
    assert!(result.tasks.last().unwrap().detail.contains("6–600"));
    let mut legacy = serde_json::to_value(&i).unwrap();
    legacy.as_object_mut().unwrap().remove("baselineId");
    assert!(serde_json::from_value::<Investigation>(legacy)
        .unwrap()
        .baseline_id
        .is_none());
}

#[test]
fn explicit_investigation_capture_links_its_own_real_baseline() {
    use coqui_engine::{assistant, baseline};
    use std::sync::atomic::AtomicBool;
    let (_temp, c) = fixture();
    let i = assistant::ask(&c, None, Some("scan"), "My coding task is slow").unwrap();
    let mut progress = vec![];
    let result = assistant::collect(
        &c,
        &i.id,
        InvestigationCollection::Baseline(BaselineRequest {
            workload: "Coding".into(),
            target: "Record this interval".into(),
            context: "Disposable test process".into(),
            duration_seconds: 6,
        }),
        &[],
        &AtomicBool::new(false),
        |stage| {
            progress.push(stage);
            true
        },
    )
    .unwrap();
    assert_eq!(result.status, "evidenceReady");
    assert!(result
        .tasks
        .iter()
        .all(|task| task.name != "ReviewWorkloadBaseline"));
    let captured = baseline::get(&c, result.baseline_id.as_ref().unwrap()).unwrap();
    assert_eq!(captured.status, "complete");
    assert!(captured.samples.len() >= 3);
    assert!(progress.iter().any(|stage| stage.status == "running"
        && stage
            .tasks
            .last()
            .unwrap()
            .detail
            .contains("actual samples")));
    let answer = assistant::ask(&c, Some(&i.id), None, "This coding task is slow").unwrap();
    assert!(answer.tasks.iter().any(|task| !task.evidence.is_empty()));
    assert_eq!(answer.tasks.last().unwrap().status, "complete");
    let mut cancelled = captured;
    cancelled.status = "cancelled".into();
    records::save(
        &c,
        "baseline",
        &cancelled.id,
        &cancelled.created_at,
        &cancelled.status,
        "Cancelled test interval",
        &cancelled,
    )
    .unwrap();
    let answer = assistant::ask(&c, Some(&i.id), None, "This coding task is slow").unwrap();
    assert_eq!(answer.tasks.last().unwrap().status, "needsMeasurement");
    assert!(answer.tasks.last().unwrap().detail.contains("cancelled"));
}
