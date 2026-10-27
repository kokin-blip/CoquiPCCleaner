use coqui_engine::{database, model::*, records, trials};
use rusqlite::Connection;

fn trial(c: &Connection, id: &str, seconds: f64) -> WorkloadTrial {
    let t = WorkloadTrial {
        id: id.into(),
        created_at: now(),
        finished_at: Some(now()),
        status: "complete".into(),
        request: WorkloadTrialRequest {
            workload: "Software development".into(),
            task: "Fixture build".into(),
            inputs: "revision 1, clean".into(),
            application: "compiler 1".into(),
            conditions: "fixture conditions".into(),
            target_seconds: 10.,
        },
        elapsed_seconds: Some(seconds),
        completion_reported: true,
        samples: vec![],
        power: vec![],
        qualification: "Synthetic test data, not a device measurement".into(),
    };
    save(c, &t);
    t
}
fn save(c: &Connection, t: &WorkloadTrial) {
    records::save(
        c,
        "workloadTrial",
        &t.id,
        &t.created_at,
        &t.status,
        &t.request.task,
        t,
    )
    .unwrap();
}
fn request(before: &[&str], after: &[&str]) -> TrialComparisonRequest {
    TrialComparisonRequest {
        before_ids: before.iter().map(|s| s.to_string()).collect(),
        after_ids: after.iter().map(|s| s.to_string()).collect(),
        change: "Synthetic comparison".into(),
    }
}

#[test]
fn read_only_comparison_computes_even_and_odd_medians_and_saves_only_explicitly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trials.db");
    let c = database::open(&path).unwrap();
    for (id, time) in [("a", 9.), ("b", 15.), ("c", 7.), ("d", 12.), ("e", 8.)] {
        trial(&c, id, time);
    }
    let changes = c.total_changes();
    let r = request(&["a", "b"], &["c", "d", "e"]);
    let compared = trials::compare(&c, r.clone()).unwrap();
    assert_eq!(c.total_changes(), changes);
    assert_eq!(compared.before.median_seconds, 12.);
    assert_eq!(compared.after.median_seconds, 8.);
    assert_eq!(compared.before.minimum_seconds, 9.);
    assert_eq!(compared.after.maximum_seconds, 12.);
    assert_eq!(compared.before.target_met_count, 1);
    assert_eq!(compared.after.target_met_count, 2);
    assert!((compared.median_change_percent + 100. / 3.).abs() < 0.0001);
    assert!(compared.matching_conditions);
    assert_eq!(compared.limitations.len(), 1);
    assert!(records::list(&c, Some("trialComparison"), 0)
        .unwrap()
        .is_empty());
    let saved = trials::save_comparison(&c, r).unwrap();
    drop(c);
    let c = database::open(&path).unwrap();
    let reopened: TrialComparison = records::get(&c, "trialComparison", &saved.id).unwrap();
    assert_eq!(reopened.after.median_seconds, 8.);
    assert_eq!(reopened.request.after_ids.len(), 3);
}

#[test]
fn different_protocols_reject_and_different_conditions_and_targets_are_qualified() {
    let dir = tempfile::tempdir().unwrap();
    let c = database::open(&dir.path().join("trials.db")).unwrap();
    trial(&c, "a", 10.);
    let mut b = trial(&c, "b", 8.);
    b.request.application = "compiler 2".into();
    b.request.conditions = "on battery".into();
    b.request.target_seconds = 5.;
    save(&c, &b);
    let r = request(&["a"], &["b"]);
    let comparison = trials::compare(&c, r.clone()).unwrap();
    assert!(!comparison.matching_conditions);
    assert_eq!(comparison.limitations.len(), 3);
    assert_eq!(comparison.after.target_met_count, 0);
    b.request.inputs = "revision 2".into();
    save(&c, &b);
    assert!(trials::compare(&c, r.clone()).is_err());
    b.request.inputs = "revision 1, clean".into();
    b.request.task = "Other task".into();
    save(&c, &b);
    assert!(trials::compare(&c, r).is_err());
    for invalid in [
        request(&[], &["a"]),
        request(&["a"], &["a"]),
        request(&["a", "a"], &["b"]),
        request(&["missing"], &["a"]),
        request(&["a"; 21], &["b"]),
    ] {
        assert!(trials::compare(&c, invalid).is_err());
    }
}

#[test]
fn incomplete_and_missing_intervals_never_establish_completion_or_zero_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let c = database::open(&dir.path().join("trials.db")).unwrap();
    let original = trial(&c, "a", 5.);
    for status in ["running", "cancelled", "timedOut", "interrupted", "failed"] {
        let mut t = original.clone();
        t.status = status.into();
        save(&c, &t);
        assert!(trials::completed(&c, "a").is_err());
    }
    for seconds in [None, Some(0.), Some(-1.), Some(600.)] {
        let mut t = original.clone();
        t.elapsed_seconds = seconds;
        save(&c, &t);
        assert!(trials::completed(&c, "a").is_err());
    }
    let mut t = original.clone();
    t.status = "running".into();
    t.completion_reported = false;
    t.finished_at = None;
    t.elapsed_seconds = Some(3.);
    save(&c, &t);
    database::recover_interrupted(&c).unwrap();
    let recovered: WorkloadTrial = records::get(&c, "workloadTrial", "a").unwrap();
    assert_eq!(recovered.status, "interrupted");
    assert_eq!(recovered.elapsed_seconds, Some(3.));
    assert!(!recovered.completion_reported);
    assert!(recovered.finished_at.is_none());
    t.elapsed_seconds = None;
    assert_eq!(trials::evidence(&t)[0].state, CapabilityState::Unknown);
    assert_eq!(trials::evidence(&t)[0].value, "Unavailable");
}

#[test]
fn validation_bounds_the_task_protocol_and_target() {
    let dir = tempfile::tempdir().unwrap();
    let c = database::open(&dir.path().join("trials.db")).unwrap();
    let mut r = trial(&c, "a", 5.).request;
    for target in [0., -1., 86401., f64::NAN, f64::INFINITY] {
        r.target_seconds = target;
        assert!(trials::validate(&r).is_err());
    }
    r.target_seconds = 2.;
    r.inputs.clear();
    assert!(trials::validate(&r).is_err());
    r.inputs = "a".repeat(2001);
    assert!(trials::validate(&r).is_err());
}
