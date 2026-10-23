//! Explicit native validation of installer-created disposable objects only.
//! This CLI has no caller-supplied paths or general RPC interface.
use crate::{
    helper,
    model::*,
    privileged::{Job, Operation, Preferences},
    removal::{RemovalApproval, RemovalPlan, RemovalRequest},
};
use anyhow::{Context, Result};
use std::{
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    id: String,
    purpose: String,
}
fn marker() -> Result<Marker> {
    let path = Path::new("/Library/Application Support/CoquiPCCleaner/Helper/validation.json");
    let f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let m = f.metadata()?;
    anyhow::ensure!(
        m.is_file() && m.uid() == 0 && m.mode() & 0o022 == 0 && m.len() < 4096 && m.nlink() == 1,
        "Disposable validation marker invalid"
    );
    let mut bytes = vec![];
    f.take(4097).read_to_end(&mut bytes)?;
    let marker: Marker = serde_json::from_slice(&bytes)?;
    uuid::Uuid::parse_str(&marker.id)?;
    anyhow::ensure!(
        marker.purpose == "coqui-disposable-administrator-validation",
        "This is not a disposable validation installation"
    );
    Ok(marker)
}
fn preferences() -> Result<Preferences> {
    let home = crate::privileged::account(unsafe { libc::getuid() })?.0;
    let path = home.join("Library/Application Support/CoquiPCCleaner/coqui.sqlite3");
    if path.exists() {
        let c = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let s = crate::database::settings(&c)?;
        return Ok(Preferences {
            excluded_paths: s.excluded_paths,
            protected_paths: crate::applications::preferences(&c)?.protected_paths,
            retention_days: s.retention_days,
        });
    }
    Ok(Preferences {
        excluded_paths: vec![],
        protected_paths: vec![],
        retention_days: 30,
    })
}
fn wait(mut job: Job, cancel_after_stage: bool) -> Result<Job> {
    if job.status == "authorized" {
        job = serde_json::from_value(helper::production(
            Operation::Start { id: job.id.clone() },
            false,
        )?)?;
    }
    let start = std::time::Instant::now();
    let mut cancelled = false;
    while job.status == "running" {
        if cancel_after_stage && job.session.is_some() && !cancelled {
            helper::production(Operation::Cancel { id: job.id.clone() }, false)?;
            cancelled = true;
        }
        anyhow::ensure!(
            start.elapsed() < std::time::Duration::from_secs(150),
            "Native fixture job timed out; inspect retained records"
        );
        std::thread::sleep(std::time::Duration::from_millis(150));
        job = serde_json::from_value(helper::production(
            Operation::Status { id: job.id.clone() },
            false,
        )?)?;
    }
    Ok(job)
}
fn run(op: Operation) -> Result<Job> {
    wait(
        serde_json::from_value(helper::production(op, true)?)?,
        false,
    )
}
fn successful(job: Job) -> Result<Job> {
    anyhow::ensure!(
        job.error.is_none(),
        "{}",
        job.error.as_deref().unwrap_or("Helper job failed")
    );
    Ok(job)
}
fn execute(plan: &RemovalPlan, p: &Preferences) -> Result<Job> {
    run(Operation::Execute {
        approval: RemovalApproval {
            id: plan.id.clone(),
            paths: plan.items.iter().map(|i| i.path.clone()).collect(),
            acknowledged: true,
        },
        preferences: p.clone(),
    })
}
pub fn validate() -> Result<()> {
    let marker = marker()?;
    let p = preferences()?;
    let app = PathBuf::from(format!(
        "/Applications/Coqui Administrator Fixture {}.app",
        marker.id
    ));
    let cache = PathBuf::from(format!(
        "/Library/Caches/org.coqui.validation.{}",
        marker.id
    ));
    let health = helper::production(Operation::Health, false)?;
    anyhow::ensure!(
        health["compatible"] == true,
        "Matching production helper required"
    );
    helper::production(
        Operation::SetPreferences {
            preferences: p.clone(),
        },
        false,
    )?;
    // Verify signed copies and other builds cannot impersonate the installed peer.
    let temporary =
        std::env::temp_dir().join(format!("coqui-helper-peer-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&temporary)?;
    let result = (|| -> Result<()> {
        for (name, other) in [("copy", false), ("impostor", true)] {
            let copy = temporary.join(name);
            std::fs::copy(std::env::current_exe()?, &copy)?;
            if other {
                anyhow::ensure!(
                    std::process::Command::new("/usr/bin/codesign")
                        .args([
                            "--force",
                            "--sign",
                            "-",
                            "--identifier",
                            "com.coqui.DisposableImpostor"
                        ])
                        .arg(&copy)
                        .status()?
                        .success(),
                    "Fixture signing failed"
                );
            }
            let output = std::process::Command::new(copy).arg("--health").output()?;
            anyhow::ensure!(!output.status.success(), "Uninstalled peer was accepted");
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&temporary);
    result?;
    anyhow::ensure!(
        helper::request(
            &serde_json::json!({"protocol":2,"version":"mismatch","request":{"operation":"health"}})
        )
        .is_err(),
        "Mismatched protocol build accepted"
    );
    let unauthorized = helper::request(
        &serde_json::json!({"protocol":2,"version":env!("CARGO_PKG_VERSION"),"request":Operation::PreviewRemoval{request:RemovalRequest{paths:vec![cache.to_string_lossy().into_owned()],application:Some(app.to_string_lossy().into_owned())},preferences:p.clone()}}),
    );
    anyhow::ensure!(
        unauthorized.is_err(),
        "Missing operation authorization accepted"
    );
    let inspection = successful(run(Operation::InspectApplication {
        path: app.to_string_lossy().into_owned(),
        preferences: p.clone(),
    })?)?
    .application
    .context("No fixture inventory")?;
    anyhow::ensure!(
        inspection.protection.is_empty(),
        "Disposable fixture protected: {}",
        inspection.protection.join("; ")
    );
    let preview = successful(run(Operation::PreviewRemoval {
        request: RemovalRequest {
            paths: vec![
                cache.to_string_lossy().into_owned(),
                app.to_string_lossy().into_owned(),
            ],
            application: Some(app.to_string_lossy().into_owned()),
        },
        preferences: p.clone(),
    })?)?
    .plan
    .context("No fixture removal preview")?;
    let moved = successful(execute(&preview, &p)?)?
        .plan
        .context("No move result")?;
    eprintln!(
        "Disposable fixture move result: {}",
        serde_json::to_string(&moved)?
    );
    let recover = (|| -> Result<RemovalPlan> {
        anyhow::ensure!(
            moved.items.iter().all(|i| i.status == "movedToTrash"),
            "Fixture moves incomplete; inspect retained helper history"
        );
        anyhow::ensure!(
            !app.exists() && !cache.exists(),
            "Original fixture still exists after verified move"
        );
        let replay = execute(&preview, &p)?;
        anyhow::ensure!(replay.error.is_some(), "Consumed fixture approval replayed");
        let restore = successful(run(Operation::PreviewRestore {
            id: moved.id.clone(),
            paths: moved.items.iter().map(|i| i.path.clone()).collect(),
            preferences: p.clone(),
        })?)?
        .plan
        .context("No restore review")?;
        let restored = successful(execute(&restore, &p)?)?
            .plan
            .context("No restore result")?;
        anyhow::ensure!(
            restored.items.iter().all(|i| i.status == "restored"),
            "Fixture restore incomplete; inspect retained helper history"
        );
        Ok(restored)
    })();
    if recover.is_err() {
        // Only verified fixture moves receive a new, separate recovery review. Never replay an approval or retry an unverified move.
        let retained: Vec<RemovalPlan> =
            serde_json::from_value(helper::production(Operation::History { offset: 0 }, false)?)?;
        if let Some(source) = retained.iter().find(|r| r.id == moved.id) {
            let paths = source
                .items
                .iter()
                .filter(|i| i.status == "movedToTrash")
                .map(|i| i.path.clone())
                .collect::<Vec<_>>();
            if !paths.is_empty() {
                if let Ok(j) = run(Operation::PreviewRestore {
                    id: source.id.clone(),
                    paths,
                    preferences: p.clone(),
                }) {
                    if let Some(review) = j.plan {
                        let _ = execute(&review, &p);
                    }
                }
            }
        }
    }
    let restored = recover?;
    anyhow::ensure!(
        std::fs::symlink_metadata(&app)?.uid() == 0
            && std::fs::symlink_metadata(&cache)?.uid() == 0,
        "Fixture ownership changed"
    );
    let root = std::env::temp_dir().join(format!(
        "coqui-native-scope-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let c = crate::database::open(&root)?;
    let plan = crate::volumes::plan(&c)?;
    let volume = plan
        .volumes
        .iter()
        .find(|v| v.internal && v.mount == "/")
        .or_else(|| plan.volumes.iter().find(|v| v.internal))
        .context("No mounted internal fixture scan scope")?;
    let request = ScanRequest {
        mode: "deep".into(),
        plan_id: Some(plan.id.clone()),
        roots: vec![volume.mount.clone()],
        excluded_paths: p.excluded_paths.clone(),
    };
    let scan: Job = serde_json::from_value(helper::production(
        Operation::Scan {
            request,
            plan,
            preferences: p,
        },
        true,
    )?)?;
    let scan = successful(wait(scan, true)?)?;
    let session = scan.session.context("No native metadata scan result")?;
    let history: Vec<RemovalPlan> =
        serde_json::from_value(helper::production(Operation::History { offset: 0 }, false)?)?;
    anyhow::ensure!(
        history
            .iter()
            .find(|r| r.id == moved.id)
            .is_some_and(|r| r.items.iter().all(|i| i.status == "restoredFromTrash")),
        "Restored originals retain stale recovery offers"
    );
    std::thread::sleep(std::time::Duration::from_millis(200));
    helper::production(
        Operation::Lifecycle {
            action: crate::privileged::Lifecycle::Stop,
        },
        true,
    )?;
    std::thread::sleep(std::time::Duration::from_millis(1200));
    anyhow::ensure!(
        helper::production(Operation::Health, false)?["compatible"] == true,
        "On-demand helper restart failed"
    );
    let reopened: Vec<RemovalPlan> =
        serde_json::from_value(helper::production(Operation::History { offset: 0 }, false)?)?;
    anyhow::ensure!(
        reopened.iter().any(|r| r.id == restored.id),
        "Helper records did not survive restart"
    );
    unsafe extern "C" {
        fn coqui_authorization_interactive_requests() -> u32;
    }
    let authorization_requests = unsafe { coqui_authorization_interactive_requests() };
    anyhow::ensure!(authorization_requests == 1, "Expected one interactive authorization request across the fixture workflow, got {authorization_requests}");
    helper::clear_authorization();
    anyhow::ensure!(
        helper::authorization_remaining() == 0,
        "Administrator access did not lock"
    );
    drop(c);
    let _ = std::fs::remove_file(&root);
    let _ = std::fs::remove_file(root.with_extension("sqlite3-wal"));
    let _ = std::fs::remove_file(root.with_extension("sqlite3-shm"));
    println!(
        "{}",
        serde_json::json!({"interactiveAuthorizationRequests":authorization_requests,"authorizationLockedAtEnd":true,"fixtureId":marker.id,"authenticatedInstalledPeer":true,"uninstalledAndImpostorPeers":"rejected","missingAuthorization":"rejected","versionMismatch":"rejected","approvalReplay":"rejected","rootOwnedTrashAndRestore":"verified","restoredRecord":restored.id,"scan":session,"helperRestart":"retained records reopened","qualification":"Disposable administrator fixture validation only; no existing app/user files removed. Full hardware, TCC and Gatekeeper acceptance remain separately qualified."})
    );
    Ok(())
}

/// Read-only diagnosis of this installation's marked fixture records only.
pub fn report() -> Result<()> {
    let marker = marker()?;
    let mut records = vec![];
    for offset in (0..1000).step_by(50) {
        let page: Vec<RemovalPlan> =
            serde_json::from_value(helper::production(Operation::History { offset }, false)?)?;
        let count = page.len();
        records.extend(
            page.into_iter()
                .filter(|p| p.items.iter().any(|i| i.path.contains(&marker.id))),
        );
        if count < 50 {
            break;
        }
    }
    println!("{}", serde_json::to_string_pretty(&records)?);
    Ok(())
}
