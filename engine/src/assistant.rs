//! Transparent local guided investigator. No remote model or arbitrary commands.
use crate::{database, model::*, records};
use anyhow::Result;
use rusqlite::Connection;
pub fn ask(
    c: &Connection,
    id: Option<&str>,
    session_id: Option<&str>,
    text: &str,
) -> Result<Investigation> {
    anyhow::ensure!(
        !text.trim().is_empty() && text.len() <= 4000,
        "Enter a question up to 4,000 characters"
    );
    let mut i = if let Some(id) = id {
        records::get::<Investigation>(c, "investigation", id)?
    } else {
        Investigation{id:uuid::Uuid::new_v4().to_string(),created_at:now(),title:text.chars().take(80).collect(),session_id:session_id.map(str::to_string),messages:vec![],tasks:vec![],findings:vec![],actions:vec![],status:"clarification".into(),qualification:"Local guided investigation, not a generative model. Answers use only retained scan/baseline observations and explicit rules. No data leaves this Mac. A saved observation is not a current measurement or confirmed cause.".into()}
    };
    anyhow::ensure!(
        i.messages.len() < 100,
        "Start a new investigation after 50 exchanges"
    );
    let scan = i
        .session_id
        .as_deref()
        .map(|id| database::session(c, id))
        .transpose()?;
    i.messages.push(InvestigationMessage {
        role: "user".into(),
        text: text.into(),
        created_at: now(),
    });
    i.status = "clarification".into();
    i.tasks.clear();
    i.findings.clear();
    i.actions.clear();
    let t = text.to_lowercase();
    let prior = i
        .messages
        .iter()
        .rev()
        .skip(1)
        .find(|m| m.role == "user")
        .map(|m| m.text.to_lowercase())
        .unwrap_or_default();
    let startup = t.contains("startup")
        || t.contains("login")
        || t.contains("boot")
        || prior.contains("startup")
        || prior.contains("boot");
    let response = if scan.is_none() {
        i.actions.push("System Analysis".into());
        "Run or select a system analysis scan first so I can cite its observations. Then describe what happens, which app is affected, and whether the issue is reproducible.".into()
    } else if startup
        && !t.contains("after login")
        && !t.contains("at login")
        && !t.contains("power-on")
        && !t.contains("power on")
    {
        "When is the delay: power-on before login, at login, or after login? Startup configuration cannot establish boot duration or a cause.".into()
    } else if startup {
        let s = scan.as_ref().unwrap();
        let mut evidence = vec![DiagnosticEvidence::observed(
            "Saved scan",
            s.started_at.clone(),
            "",
            "Retained analysis",
        )];
        if let Some(m) = &s.snapshot {
            evidence.extend(m.startup_items.iter().take(30).map(|item| {
                let mut e =
                    DiagnosticEvidence::observed(&item.name, item.state.clone(), "", &item.path);
                e.observed_at = m.observed_at.clone();
                e.qualification = item.explanation.clone();
                e
            }));
        }
        i.tasks.push(DiagnosticTask{name:"ReviewStartupConfiguration".into(),status:"complete".into(),evidence,detail:"Inspected saved readable startup entries. Modern login-item/runtime coverage is incomplete.".into()});
        i.actions = vec!["Performance".into(), "System Analysis".into()];
        i.status = "evidenceReady".into();
        "I reviewed the saved startup configuration. These entries are possible investigation leads, not confirmed causes of the delay. Inspect Login Items, change only an app you recognize through a recorded manual plan, then capture a comparable workload baseline. Power-on delays may require Apple diagnostics or support; I have no boot trace.".into()
    } else if [
        "storage",
        "space",
        "duplicate",
        "cache",
        "clean",
        "docker",
        "project",
    ]
    .iter()
    .any(|w| t.contains(w))
    {
        let s = scan.as_ref().unwrap();
        let mut evidence = vec![
            DiagnosticEvidence::observed(
                "Indexed files",
                s.file_count.to_string(),
                "files",
                "Saved metadata scan",
            ),
            DiagnosticEvidence::observed(
                "Indexed allocation",
                s.allocated_bytes.to_string(),
                "bytes",
                "Saved metadata scan",
            ),
        ];
        for e in &mut evidence {
            e.observed_at = s.started_at.clone();
            e.qualification="Saved scope only; APFS sharing and permission gaps prevent exact reclaimable-space inference.".into();
        }
        i.tasks.push(DiagnosticTask {
            name: "ReviewStorageEvidence".into(),
            status: "complete".into(),
            evidence,
            detail: format!(
                "Scope: {}. {} skipped observations.",
                s.scope, s.skipped_count
            ),
        });
        i.findings = s
            .findings
            .iter()
            .filter(|f| f.kind.contains("storage") || f.kind.contains("cache"))
            .cloned()
            .collect();
        i.actions = vec![
            "Storage".into(),
            "Optimization".into(),
            "Maintenance".into(),
        ];
        i.status = "evidenceReady".into();
        "The saved index provides scoped allocation evidence. Open Storage for exact paths, or Optimization for duplicate/developer analysis and supported cleanup. No target is disposable merely because it is large or old. Review the fresh exact-path plan and consequences before approving; results and real Trash recovery availability are retained.".into()
    } else if [
        "slow",
        "cpu",
        "memory",
        "performance",
        "upgrade",
        "battery",
        "power",
    ]
    .iter()
    .any(|w| t.contains(w))
    {
        let baseline = records::latest::<Baseline>(c, "baseline")?;
        let evidence = baseline
            .as_ref()
            .map(|b| b.summary.clone())
            .unwrap_or_default();
        i.tasks.push(DiagnosticTask {
            name: "ReviewWorkloadBaseline".into(),
            status: if evidence.is_empty() {
                "needsMeasurement"
            } else {
                "complete"
            }
            .into(),
            evidence,
            detail: baseline
                .map(|b| {
                    format!(
                        "{}; {} samples; {}",
                        b.request.workload,
                        b.samples.len(),
                        b.status
                    )
                })
                .unwrap_or("No retained baseline; collect a representative interval.".into()),
        });
        i.actions = vec!["Performance".into(), "System Analysis".into()];
        i.status = "evidenceReady".into();
        "Which app and representative task feel slow, and what result do you expect? Capture a baseline while reproducing it. CPU activity describes that interval; used memory alone is not pressure. Upgrade guidance separates declared demand scenarios from observed activity, wear, OS support and hardware upgradeability. A no-change recommendation is valid.".into()
    } else {
        "I can guide storage/cache and duplicate review, startup delays, workload measurements, power plans, or upgrade scenarios. Which concern should we investigate, and what did you observe? I cannot diagnose unrelated questions or infer an executed action.".into()
    };
    i.messages.push(InvestigationMessage {
        role: "assistant".into(),
        text: response,
        created_at: now(),
    });
    records::save(
        c,
        "investigation",
        &i.id,
        &i.created_at,
        &i.status,
        &i.title,
        &i,
    )?;
    Ok(i)
}
