use crate::{baseline, database, model::*, records};
use anyhow::Result;
use rusqlite::Connection;

// Reference content reviewed during development, not fetched by an assessment.
// Retain these dates in each report; never substitute its creation time.
fn source_reviews() -> Vec<AdviceSourceReview> {
    [
        ("https://support.apple.com/en-us/101613", "Power modes on Mac", "Explains power modes and model-specific availability. Coqui does not verify that a mode is offered or enabled on this Mac."),
        ("https://support.apple.com/en-ie/102328", "Mac mini memory installation and upgrades", "Mac mini guidance only. It does not establish upgrade feasibility for another Mac family or a particular service option."),
        ("https://support.apple.com/macos/upgrade", "macOS Ventura upgrade guide", "Ventura-specific guidance. This link does not establish eligibility for a current or future macOS release."),
        ("https://www.apple.com/mac/compare/", "Compare Mac models", "Vendor model comparison. Coqui does not import current configurations, prices, availability or compatibility from this page."),
    ].into_iter().map(|(url, title, scope)| AdviceSourceReview {
        url: url.into(), title: title.into(), publisher: "Apple".into(),
        reviewed_on: Some("2026-10-23".into()), scope: scope.into(),
    }).collect()
}

fn verification_limits() -> Vec<String> {
    vec![
        "Software and peripheral requirements: exact application versions, plug-ins, drivers and peripherals are not checked against this Mac.".into(),
        "Upgrade feasibility: model-specific service options, component compatibility and an upgrade's performance benefit are not verified.".into(),
        "Prices and availability: no current vendor quote is fetched. Any supplied cost range remains an unverified user quote.".into(),
        "Future support and hardware life: no future macOS eligibility or remaining hardware lifespan is established.".into(),
        "Workload performance: activity samples and user-reported task intervals do not establish an automated benchmark, a limiting component or the cause of a delay.".into(),
    ]
}
pub fn assess(c: &Connection, r: AdviceRequest) -> Result<AdviceReport> {
    anyhow::ensure!(
        !r.workload.trim().is_empty()
            && !r.expectation.trim().is_empty()
            && r.workload.len() <= 2000
            && r.expectation.len() <= 2000,
        "Describe your workload and expected result"
    );
    anyhow::ensure!(
        r.current_demand_percent.is_finite()
            && (1.0..=100.0).contains(&r.current_demand_percent)
            && r.annual_growth_percent.is_finite()
            && (0.0..=100.0).contains(&r.annual_growth_percent),
        "Demand must be 1–100%; assumed annual growth 0–100%"
    );
    if r.quote_low.is_some() || r.quote_high.is_some() {
        let (l, h) = (r.quote_low.unwrap_or(-1.), r.quote_high.unwrap_or(-1.));
        anyhow::ensure!(
            l.is_finite()
                && h.is_finite()
                && l >= 0.
                && h >= l
                && !r.quote_source.trim().is_empty()
                && r.quote_source.len() <= 1000,
            "Provide a valid low/high quote and its source/currency"
        );
    }
    let session = database::session(c, &r.session_id)?;
    let b = r
        .baseline_id
        .as_deref()
        .map(|id| baseline::get(c, id))
        .transpose()?;
    let mut findings = vec![];
    let trial = r
        .trial_id
        .as_deref()
        .map(|id| crate::trials::completed(c, id))
        .transpose()?;
    if let Some(trial) = &trial {
        anyhow::ensure!(
            trial
                .request
                .workload
                .trim()
                .eq_ignore_ascii_case(r.workload.trim()),
            "Choose a task trial matching this workload"
        );
        findings.push(DiagnosticFinding { id: uuid::Uuid::new_v4().to_string(), title: if trial.elapsed_seconds.unwrap() <= trial.request.target_seconds { "Task met its declared target in this trial" } else { "Task exceeded its declared target in this trial" }.into(),
            explanation: format!("'{}' took {:.3} seconds, with a declared target of {:.3} seconds. Completion was user-reported; repeat before judging routine suitability or replacement. The recorded target belongs to this trial, not a later expectation.", trial.request.task, trial.elapsed_seconds.unwrap(), trial.request.target_seconds),
            severity: "info".into(), confidence: "Measured interval; user-reported completion; one trial".into(), kind: "taskOutcome".into(), evidence: crate::trials::evidence(trial) });
    }
    if let Some(b) = b
        .as_ref()
        .filter(|b| b.status == "complete" && b.samples.len() >= 3)
    {
        let high = b.samples.iter().filter(|s| s.cpu_percent >= 90.).count();
        findings.push(DiagnosticFinding{id:uuid::Uuid::new_v4().to_string(),title:if high*2>=b.samples.len(){"High CPU activity during this baseline"}else{"No sustained high CPU activity observed in this interval"}.into(),explanation:format!("{high}/{} samples at or above 90% system CPU. This does not identify the limiting component or establish task completion time. Expected outcome: {}",b.samples.len(),r.expectation),severity:"info".into(),confidence:"Observed interval only".into(),kind:"workload".into(),evidence:b.summary.clone()});
    }
    let mut scenarios = vec![];
    for (name, growth) in [
        ("Lower-growth scenario", r.annual_growth_percent / 2.),
        ("Declared-growth scenario", r.annual_growth_percent),
        (
            "Higher-growth scenario",
            (r.annual_growth_percent * 1.5).min(100.),
        ),
    ] {
        let years = if r.current_demand_percent >= 100. {
            "Already at the declared threshold".into()
        } else if growth == 0. {
            "No crossing under a zero-growth assumption".into()
        } else {
            format!(
                "{:.1} illustrative years to 100% assumed demand",
                ((100. / r.current_demand_percent).ln() / (1. + growth / 100.).ln()).min(100.)
            )
        };
        let mut e = DiagnosticEvidence::observed(
            name,
            years,
            "",
            "User-supplied demand model: demand × (1 + annual growth)^years",
        );
        e.qualification=format!("Starting demand {}%; assumed annual growth {growth:.1}%. This is not a hardware lifetime, OS support, benchmark, price or performance forecast.",r.current_demand_percent);
        scenarios.push(e);
    }
    scenarios.push(DiagnosticEvidence::unavailable(
        "Hardware wear / remaining life",
        "Available health readings cannot predict a remaining lifespan.",
    ));
    scenarios.push(DiagnosticEvidence::unavailable("Future macOS support horizon","No future support years are inferred. Verify the desired macOS release's official compatibility list."));
    let mut upgrade=DiagnosticEvidence::observed("Memory upgradeability","Apple Silicon unified memory is configured at purchase; verify exact model service options before buying.".into(),"","Apple model specifications / service documentation");
    upgrade.state = CapabilityState::ManualOnly;
    scenarios.push(upgrade);
    if let (Some(l), Some(h)) = (r.quote_low, r.quote_high) {
        let mut e = DiagnosticEvidence::observed(
            "Provided cost range",
            format!("{l:.2}–{h:.2}"),
            "user-provided currency/source",
            &r.quote_source,
        );
        e.qualification="User-supplied quote; not fetched, independently verified or a compatibility endorsement.".into();
        scenarios.push(e);
    } else {
        scenarios.push(DiagnosticEvidence::unavailable("Upgrade cost","No current sourced quote supplied. Obtain a model-specific vendor/service quote; no price was invented."));
    }
    let source_reviews = source_reviews();
    let report=AdviceReport{id:uuid::Uuid::new_v4().to_string(),created_at:now(),workload:r.workload,assessment:if findings.is_empty(){"More representative measurements needed before an upgrade recommendation."}else{"Keep the current Mac if it meets your actual task expectations. Investigate repeatable delays before comparing a replacement."}.into(),findings,scenarios,sources:source_reviews.iter().map(|s|s.url.clone()).collect(),source_reviews,verification_limits:verification_limits(),qualification:format!("Based on saved scan {} ({}) and explicit user assumptions. Compatibility requires checking exact software, peripherals and model; workload sufficiency, wear and OS eligibility are separate questions. No upgrade can be the correct outcome.",session.started_at,session.snapshot.as_ref().map(|s|s.model.as_str()).unwrap_or("model unavailable"))};
    records::save(
        c,
        "advice",
        &report.id,
        &report.created_at,
        "complete",
        &report.workload,
        &report,
    )?;
    Ok(report)
}
