//! Qualified application/developer storage leads. Never infers disposability.
use crate::{database, model::*, records};
use anyhow::Result;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
pub fn analyze(
    c: &Connection,
    id: &str,
    exclusions: &[std::path::PathBuf],
    cancel: &AtomicBool,
) -> Result<ResourceReview> {
    let session = database::session(c, id)?;
    anyhow::ensure!(session.status != "running", "Wait for the scan");
    let mut evidence = vec![];
    let mut suggestions = vec![];
    let mut q=c.prepare("SELECT path,allocated FROM files WHERE session_id=?1 AND directory=1 AND (name IN ('node_modules','.venv','venv','DerivedData','target','.dart_tool','__pycache__','CoreSimulator','Docker','Android','Archives','Logs') OR parent LIKE '%/Library/Caches') ORDER BY allocated DESC LIMIT 100")?;
    let rows = q
        .query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let apps = session
        .snapshot
        .as_ref()
        .map(|m| {
            m.applications
                .iter()
                .take(500)
                .take_while(|_| !cancel.load(Ordering::SeqCst))
                .filter(|app| {
                    !exclusions
                        .iter()
                        .any(|p| Path::new(&app.path).starts_with(p) || p.starts_with(&app.path))
                })
                .filter_map(|app| crate::platform::bundle_identifier(Path::new(&app.path)))
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    for (path, size) in rows {
        let mut e = DiagnosticEvidence::observed(
            &path,
            size.to_string(),
            "bytes",
            "Saved scoped metadata index",
        );
        e.observed_at = session.started_at.clone();
        e.qualification="Indexed allocation only. Folder names do not confirm ownership, enabled state, rebuildability or recoverable space.".into();
        if path.contains("/Library/Caches/") {
            let folder = Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if folder.split('.').count() >= 3 {
                let mut association = e.clone();
                association.label = format!("Application association: {path}");
                association.value = if apps.contains(folder) {
                    "Matching readable bundle identifier"
                } else {
                    "No matching identifier in readable application inventory; unconfirmed leftover"
                }
                .into();
                association.unit = String::new();
                association.state = if apps.contains(folder) {
                    CapabilityState::Supported
                } else {
                    CapabilityState::Unknown
                };
                association.source =
                    "Saved application paths + bounded public Info.plist identifiers".into();
                association.qualification="Apps can live elsewhere and share caches. Missing matches do not establish uninstallation, ownership or disposability. Inspect in Finder; only separately eligible cache files can enter cleanup review.".into();
                evidence.push(association);
            }
        }
        evidence.push(e);
    }
    if let Some(m) = &session.snapshot {
        suggestions.push(DiagnosticFinding{id:uuid::Uuid::new_v4().to_string(),title:"Application association needs confirmation".into(),explanation:format!("{} application bundles were observed in readable Applications locations. Missing matches do not prove an orphan: apps can live elsewhere, share containers or retain valuable data. Inspect cache/application support ownership in Finder and the app's own storage controls before acting. Application Support and preference removal is unavailable.",m.applications.len()),severity:"info".into(),confidence:"Incomplete coverage".into(),kind:"organization".into(),evidence:vec![]});
    }
    suggestions.push(DiagnosticFinding{id:uuid::Uuid::new_v4().to_string(),title:"Review intentional archives and Downloads".into(),explanation:"Use Storage's category filters and folder drill-down to inspect documents, archives and large files. Organize only files you recognize; Coqui does not move documents automatically or infer junk from age/size.".into(),severity:"info".into(),confidence:"Manual review".into(),kind:"organization".into(),evidence:vec![]});
    if let Some(report) = crate::developer::get(c, id)? {
        let mut seen = HashSet::new();
        let mut related = BTreeMap::<String, Vec<(String, String)>>::new();
        for artifact in report.artifacts.iter().take(50) {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            let Some(root) = artifact.git.root.as_ref() else {
                continue;
            };
            if !seen.insert(root.clone()) || seen.len() > 20 {
                continue;
            }
            let common = artifact.git.common_directory.clone().unwrap_or_default();
            if [
                &artifact.git.root,
                &artifact.git.git_directory,
                &artifact.git.common_directory,
            ]
            .into_iter()
            .flatten()
            .any(|path| {
                exclusions
                    .iter()
                    .any(|e| Path::new(path).starts_with(e) || e.starts_with(path))
            }) {
                continue;
            }
            evidence.push(DiagnosticEvidence{label:format!("Registered repository: {root}"),value:format!("{} worktrees; common directory {common}",artifact.git.worktrees.len()),unit:String::new(),source:"Saved Git/worktree validation".into(),observed_at:report.checked_at.clone(),state:CapabilityState::Supported,qualification:"Shared common directory confirms registered worktree relationships, not duplicate or disposable repositories. Related checkout dirtiness is not implied.".into()});
            if crate::developer::available(Path::new(root), exclusions).is_ok() {
                if let Ok(bytes) = crate::developer::git(
                    Path::new(root),
                    &["config", "--local", "--get-regexp", r"^remote\..*\.url$"],
                    cancel,
                ) {
                    for line in bytes.split(|b| *b == b'\n') {
                        if let Some(at) = line.iter().position(|b| b.is_ascii_whitespace()) {
                            let url = &line[at + 1..];
                            if !url.is_empty() {
                                let hash = format!("{:x}", Sha256::digest(url));
                                related
                                    .entry(hash)
                                    .or_default()
                                    .push((root.clone(), common.clone()));
                            }
                        }
                    }
                }
            }
        }
        for (hash, mut repositories) in related {
            repositories.sort();
            repositories.dedup();
            let commons = repositories.iter().map(|(_, c)| c).collect::<HashSet<_>>();
            if repositories.len() > 1 && commons.len() > 1 {
                let mut e = DiagnosticEvidence::observed(
                    "Possible related clones",
                    repositories
                        .iter()
                        .map(|(r, _)| r.as_str())
                        .collect::<Vec<_>>()
                        .join("; "),
                    "",
                    "Fresh read-only Git local remote configuration",
                );
                e.qualification=format!("Equal configured-remote fingerprint {} with distinct common directories. This suggests a relationship, not equivalent contents, backup status, complete clone discovery or cleanup authority. URLs/credentials are not retained.",&hash[..12]);
                e.state = CapabilityState::Unknown;
                evidence.push(e);
            }
        }
    }
    let capabilities=[("Project caches / build outputs",CapabilityState::Supported,"Optimization → Developer artifacts runs fresh Git/worktree checks; only explicitly supported outputs can enter approval."),("Node dependencies / Python environments",CapabilityState::ManualOnly,"Detected by project markers. Dependency installs, editable packages and environment-local state can be unique; automatic cleanup is unavailable. Inspect exact project and recreate through its package manager only after your own review."),("Xcode / simulators / Android emulators",CapabilityState::ManualOnly,"Indexed folders are leads. Xcode Settings → Locations and Platforms, Organizer Archives, and Android Studio Device Manager provide application-native review. Archives, signing material and simulator device data may be valuable; no generic removal."),("Docker resources",CapabilityState::ManualOnly,"Folder allocation is not Docker reclaimable space. Inspect the selected Docker context and Docker Desktop storage/resources. Images, containers and especially volumes can contain unique data. No daemon connection, remote-context access or generic prune is performed."),("Repository / clone relationships",CapabilityState::Supported,"Git reports identify registered worktrees and their shared common directory. Similar names/remotes are not proof of duplicate clones; repository removal is unavailable."),("Application leftovers",CapabilityState::Unknown,"Installed-app coverage is incomplete. Missing associations do not establish orphaned state; inspect exact paths manually.")].into_iter().map(|(name,state,explanation)|SystemCapability{name:name.into(),state,explanation:explanation.into()}).collect();
    let mut r=ResourceReview{id:uuid::Uuid::new_v4().to_string(),created_at:now(),session_id:id.into(),capabilities,evidence,suggestions,qualification:"Scoped metadata review, at most 100 matching folders. Run Developer artifacts for repository-aware validation. Native-manager-only resources and unconfirmed leftovers never acquire cleanup authority.".into()};
    if cancel.load(Ordering::SeqCst) {
        r.qualification
            .push_str(" Cancelled: relationship inspection is incomplete.");
    }
    records::save(
        c,
        "resources",
        &r.id,
        &r.created_at,
        if cancel.load(Ordering::SeqCst) {
            "cancelled"
        } else {
            "complete"
        },
        "Developer resources and organization",
        &r,
    )?;
    Ok(r)
}
