use crate::{model::*, platform, records};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use sysinfo::System;
pub fn power() -> Vec<DiagnosticEvidence> {
    match platform::command("/usr/bin/pmset", &["-g", "custom"], Duration::from_secs(3)) {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            let mut source = "Power source";
            let mut out = vec![];
            for line in text.lines() {
                let t = line.trim();
                if t.ends_with(':') {
                    source = if t.contains("Battery") {
                        "Battery"
                    } else if t.contains("AC") {
                        "AC"
                    } else {
                        "Power profile"
                    };
                    continue;
                }
                let fields = t.split_whitespace().collect::<Vec<_>>();
                if fields.len() == 2
                    && [
                        "lowpowermode",
                        "highpowermode",
                        "displaysleep",
                        "sleep",
                        "powernap",
                    ]
                    .contains(&fields[0])
                    && fields[1].parse::<u32>().is_ok()
                {
                    let mut e = DiagnosticEvidence::observed(
                        &format!("{source} {}", fields[0]),
                        fields[1].into(),
                        "raw setting",
                        "pmset -g custom (read-only)",
                    );
                    e.qualification="Observed public utility setting. Model-specific Energy Mode UI mapping is not assumed.".into();
                    out.push(e)
                }
            }
            if out.is_empty() {
                vec![DiagnosticEvidence::unavailable(
                    "Power profile",
                    "No recognized readable settings",
                )]
            } else {
                out
            }
        }
        Err(_) => vec![DiagnosticEvidence::unavailable(
            "Power profile",
            "Public read-only power query unavailable; inspect System Settings manually",
        )],
    }
}
fn save(c: &Connection, b: &Baseline) -> Result<()> {
    records::save(
        c,
        "baseline",
        &b.id,
        &b.created_at,
        &b.status,
        &b.request.workload,
        b,
    )
}
pub fn get(c: &Connection, id: &str) -> Result<Baseline> {
    records::get(c, "baseline", id)
}
pub fn capture(
    c: &Connection,
    request: BaselineRequest,
    cancel: &AtomicBool,
    mut emit: impl FnMut(Baseline) -> bool,
) -> Result<Baseline> {
    anyhow::ensure!(
        (6..=600).contains(&request.duration_seconds),
        "Choose a 6–600 second measurement interval"
    );
    for text in [&request.workload, &request.target, &request.context] {
        anyhow::ensure!(
            !text.trim().is_empty() && text.len() <= 2000,
            "Describe workload, target and comparison conditions (up to 2,000 characters each)"
        );
    }
    let mut b=Baseline{id:uuid::Uuid::new_v4().to_string(),created_at:now(),finished_at:None,status:"running".into(),request,samples:vec![],power:power(),summary:vec![],qualification:"Two-second system samples. Run the same representative task, input, app versions, power source and conditions for comparisons. CPU use describes activity, memory use alone does not establish pressure. Readable process I/O is not physical disk throughput. Differences do not prove causality; repeated trials are preferable.".into()};
    save(c, &b)?;
    let mut system = System::new_all();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(b.request.duration_seconds.into()) {
        for _ in 0..20 {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        b.samples.push(platform::metrics(&mut system));
        save(c, &b)?;
        if !emit(b.clone()) {
            cancel.store(true, Ordering::SeqCst);
        }
    }
    b.status = if cancel.load(Ordering::SeqCst) {
        "cancelled"
    } else {
        "complete"
    }
    .into();
    b.finished_at = Some(now());
    if !b.samples.is_empty() {
        let mean =
            b.samples.iter().map(|s| s.cpu_percent as f64).sum::<f64>() / b.samples.len() as f64;
        let peak = b
            .samples
            .iter()
            .map(|s| s.cpu_percent as f64)
            .fold(0., f64::max);
        b.summary.push(DiagnosticEvidence::observed(
            "Mean system CPU",
            format!("{mean:.1}"),
            "%",
            "Recorded two-second baseline samples",
        ));
        b.summary.push(DiagnosticEvidence::observed(
            "Peak system CPU",
            format!("{peak:.1}"),
            "%",
            "Recorded two-second baseline samples",
        ));
        if let Some(s) = b.samples.last() {
            b.summary.push(DiagnosticEvidence::observed(
                "Ending swap allocation",
                s.swap_bytes.to_string(),
                "bytes",
                "sysinfo baseline sample",
            ));
        }
        b.summary.push(DiagnosticEvidence::observed(
            "Measurement interval",
            format!("{:.1}", start.elapsed().as_secs_f64()),
            "seconds",
            "Monotonic elapsed time",
        ));
    }
    save(c, &b)?;
    emit(b.clone());
    Ok(b)
}
pub fn compare(c: &Connection, before: &str, after: &str) -> Result<BaselineComparison> {
    anyhow::ensure!(before != after, "Choose two different baselines");
    let a = get(c, before)?;
    let b = get(c, after)?;
    anyhow::ensure!(
        a.status == "complete"
            && b.status == "complete"
            && !a.samples.is_empty()
            && !b.samples.is_empty(),
        "Both baselines must have completed measurements"
    );
    let mut observations = vec![];
    if a.request.workload != b.request.workload
        || a.request.target != b.request.target
        || a.request.context != b.request.context
        || a.request.duration_seconds != b.request.duration_seconds
    {
        observations.push("Conditions/protocol differ. This is a descriptive comparison; repeat with matching conditions before drawing conclusions.".into());
    }
    let mean = |s: &Baseline| {
        s.samples.iter().map(|m| m.cpu_percent as f64).sum::<f64>() / s.samples.len() as f64
    };
    observations.push(format!("Mean CPU: {:.1}% → {:.1}% ({:+.1} percentage points). Lower CPU is not necessarily faster task completion.",mean(&a),mean(&b),mean(&b)-mean(&a)));
    observations.push(format!(
        "Ending used memory: {:.0} → {:.0} bytes. This alone does not establish memory pressure.",
        a.samples.last().unwrap().used_memory_bytes,
        b.samples.last().unwrap().used_memory_bytes
    ));
    Ok(BaselineComparison{before:a,after:b,observations,qualification:"No benchmark score or task completion time was inferred. User-provided conditions are declarations, not verified controls; background activity, thermal state and app changes can confound results.".into()})
}
pub fn plan(c: &Connection, kind: &str, requested: &str) -> Result<ManualPlan> {
    anyhow::ensure!(
        ["power", "login"].contains(&kind),
        "Unsupported settings plan"
    );
    anyhow::ensure!(
        !requested.trim().is_empty() && requested.len() <= 1000,
        "Describe the exact requested change"
    );
    let previous = if kind == "power" {
        power()
    } else {
        platform::snapshot()?
            .startup_items
            .into_iter()
            .take(100)
            .map(|i| {
                let mut e = DiagnosticEvidence::observed(
                    &i.name,
                    i.state,
                    "",
                    "Readable startup configuration",
                );
                e.qualification = i.explanation;
                e
            })
            .collect()
    };
    let p=ManualPlan{id:uuid::Uuid::new_v4().to_string(),created_at:now(),kind:kind.into(),requested:requested.into(),previous,steps:if kind=="power"{vec!["Record the current Battery/Energy settings and power source.".into(),"Choose a power mode offered by this Mac. Balanced maps to the normal/Automatic choice only where available; Efficiency to Low Power; Performance to High Power only on supported models.".into(),"Change it yourself in System Settings, then return to record observations and capture a comparable baseline.".into()]}else{vec!["Open General → Login Items & Extensions and identify the exact app.".into(),"Review whether this app is needed at login. Make the intended change yourself.".into(),"Return to record observations. Readable plist configuration cannot confirm every modern login-item toggle or its effect.".into()]},status:"awaitingManualAction".into(),observed:vec![],verification:"Not applied. Coqui has not changed any system setting.".into(),recovery:"Manual only. Refer to recorded previous observations and your own System Settings notes. No automatic rollback or guaranteed restoration is available.".into()};
    records::save(
        c,
        "manualPlan",
        &p.id,
        &p.created_at,
        &p.status,
        &p.requested,
        &p,
    )?;
    Ok(p)
}
pub fn verify_plan(c: &Connection, id: &str, performed: bool) -> Result<ManualPlan> {
    let mut p: ManualPlan = records::get(c, "manualPlan", id)?;
    anyhow::ensure!(
        p.status == "awaitingManualAction",
        "This plan is already recorded"
    );
    p.status = if performed {
        "manuallyReported"
    } else {
        "declined"
    }
    .into();
    if performed {
        p.observed = if p.kind == "power" {
            power()
        } else {
            platform::snapshot()?
                .startup_items
                .into_iter()
                .take(100)
                .map(|i| {
                    DiagnosticEvidence::observed(
                        &i.name,
                        i.state,
                        "",
                        "Readable startup configuration",
                    )
                })
                .collect()
        };
        p.verification="User reported completing the manual step. These are newly observed readings; Coqui cannot confirm the requested UI setting, attribution, or performance benefit. Compare a fresh baseline.".into();
    } else {
        p.verification = "Declined. No system change was performed by Coqui.".into()
    };
    records::save(
        c,
        "manualPlan",
        &p.id,
        &p.created_at,
        &p.status,
        &p.requested,
        &p,
    )?;
    Ok(p)
}
