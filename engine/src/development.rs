//! Explanatory evidence only; never used by a removal policy.
use crate::model::{DevelopmentIndicator, GitValidation};
use chrono::DateTime;
use std::path::Path;

fn indicator(label: &str, attention: bool, detail: impl Into<String>) -> DevelopmentIndicator {
    DevelopmentIndicator {
        label: label.into(),
        attention,
        detail: detail.into(),
    }
}

pub fn path_indicators(path: &str, kind: &str) -> Vec<DevelopmentIndicator> {
    let mut out = vec![];
    let parts = Path::new(path)
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>();
    if parts
        .windows(2)
        .any(|p| matches!(p[0], ".codex" | ".claude" | ".cursor") && p[1] == "worktrees")
    {
        out.push(indicator("Agent workspace", true, "Path is in a known agent worktree namespace. Creator and current use are unverified. It may contain unique edits, commits and ignored files; inspect it with the owning tool before removing a workspace."));
    }
    if parts.contains(&".git") {
        out.push(indicator("Git history · keep", true, "Repository metadata can contain local branches, stashes, reflogs and worktree registrations. A remote copy of one branch does not back up all of this state. Do not delete Git internals."));
    }
    let generated = matches!(
        kind,
        "rustBuild" | "webBuild" | "pythonCache" | "xcodeBuild" | "gradleBuild" | "unityBuild"
    );
    if generated {
        let debug = (kind == "rustBuild" && parts.windows(2).any(|p| p == ["target", "debug"]))
            || (kind == "xcodeBuild"
                && parts
                    .iter()
                    .any(|p| *p == "Debug" || p.starts_with("Debug-")));
        out.push(indicator(if debug { "Debug build" } else { "Build / tool output" }, false,
            "Recognized output path. Tools commonly regenerate it, but unique additions, required inputs and current use have not been verified. Rebuilding may take time."));
    } else if matches!(kind, "dependencies" | "environment" | "packageCache") {
        out.push(indicator("Dependencies · review", true, "Restoring may require exact versions, network access or offline packages. Local changes and environment configuration may not be in Git."));
    }
    if !out.is_empty() {
        out.push(indicator("Review before deleting", true, "Review contents and recovery before deleting. A folder name, old timestamp or clean Git status does not establish safe removal. Supported outputs have a separate fresh cleanup review."));
    }
    out
}

pub fn git_indicators(repo: &GitValidation, checked_at: &str) -> Vec<DevelopmentIndicator> {
    let mut out = vec![];
    if repo.status == "dirty" {
        out.push(indicator("Local changes · keep", true, "The checked worktree has staged, modified, untracked or conflicting entries. These are not established as uploaded or backed up. Preserve and review them."));
    } else if repo.status == "clean" {
        out.push(indicator("Clean worktree", false, "No changes found by the bounded Git status check. Ignored contents and submodules are outside that check; this is not a deletion approval."));
    } else {
        out.push(indicator(
            "Git unverified",
            true,
            repo.qualification.clone(),
        ));
    }
    if let Some(r) = &repo.recovery {
        let label = match r.status.as_str() {
            "matchesCachedRemote" => "Matches cached remote",
            "ahead" | "diverged" => "Commits absent from cached remote",
            "behind" => "Behind cached remote",
            "noRemoteUpstream" => "Remote backup unverified",
            _ => "Remote state unavailable",
        };
        out.push(indicator(
            label,
            r.status != "matchesCachedRemote",
            format!(
                "{} Checked {}.{}",
                r.qualification,
                checked_at,
                r.ahead
                    .zip(r.behind)
                    .map(|(a, b)| format!(" {a} ahead / {b} behind cached upstream."))
                    .unwrap_or_default()
            ),
        ));
        if let (Some(commit), Ok(checked)) =
            (&r.last_commit_at, DateTime::parse_from_rfc3339(checked_at))
        {
            if let Ok(committed) = DateTime::parse_from_rfc3339(commit) {
                let days = checked.signed_duration_since(committed).num_days();
                if days >= 90 {
                    out.push(indicator("Older commit · 90+ days", false, format!("HEAD committer date: {commit} ({days} days before this check). This measures commit age, not last use, project abandonment or safe deletion.")));
                }
            }
        }
    } else {
        out.push(indicator("Remote backup unverified", true, format!("Retained Git check {checked_at}. This review has no remote-history observation. Recheck artifacts. A configured remote alone does not verify an upload or a backup.")));
    }
    out
}
