use coqui_engine::{
    advice, assistant, baseline,
    cleanup::{self, Policy},
    database, duplicates, maintenance,
    model::*,
    records, resources, storage,
};
use std::{fs, os::unix::fs::symlink, path::PathBuf, sync::atomic::AtomicBool};
struct Fixture {
    dir: tempfile::TempDir,
    home: PathBuf,
    c: rusqlite::Connection,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().canonicalize().unwrap().join("home");
        fs::create_dir_all(home.join("Documents")).unwrap();
        let c = database::open(&dir.path().join("app.db")).unwrap();
        Self { dir, home, c }
    }
    fn file(&self, name: &str, bytes: &[u8]) -> String {
        let p = self.home.join("Documents").join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
        p.to_str().unwrap().into()
    }
    fn index(&mut self) {
        let mut s = DiagnosticSession {
            volume_plan: None,
            coverage: vec![],
            id: "scan".into(),
            started_at: now(),
            finished_at: Some(now()),
            status: "complete".into(),
            stage: "Complete".into(),
            scope: self.home.to_string_lossy().into(),
            file_count: 0,
            logical_bytes: 0.,
            allocated_bytes: 0.,
            skipped_count: 0,
            snapshot: None,
            findings: vec![],
            permissions: vec![],
            categories: vec![],
            candidates: vec![],
            errors: vec![],
        };
        database::save_session(&self.c, &s).unwrap();
        let mut rows = vec![];
        let w = storage::walk(
            &[self.home.clone()],
            &[],
            &AtomicBool::new(false),
            |r| {
                rows.push(r);
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
        database::insert_files(&mut self.c, "scan", &rows).unwrap();
        s.file_count = w.files;
        database::save_session(&self.c, &s).unwrap();
    }
    fn policy(&self) -> Policy {
        Policy {
            home: self.home.clone(),
            exclusions: vec![],
        }
    }
    fn report(&self) -> DuplicateReport {
        duplicates::analyze(
            &self.c,
            "scan",
            &self.policy(),
            &AtomicBool::new(false),
            |_| true,
        )
        .unwrap()
    }
    fn preview(&self, r: &DuplicateReport, keeper: &str, remove: &str) -> CleanupPlan {
        cleanup::preview_duplicates(
            &self.c,
            DuplicateCleanupRequest {
                report_id: r.id.clone(),
                group_id: r.groups[0].id.clone(),
                keep_path: keeper.into(),
                remove_paths: vec![remove.into()],
            },
            &self.policy(),
            &AtomicBool::new(false),
        )
        .unwrap()
    }
}
fn approve(p: &CleanupPlan) -> CleanupApproval {
    CleanupApproval {
        plan_id: p.id.clone(),
        reviewed_at: p.created_at.clone(),
        applications_closed: true,
        consequences_accepted: true,
    }
}
#[test]
fn exact_contents_require_bytes_and_keeper_and_verify_trash_without_deletion() {
    let mut f = Fixture::new();
    let keep = f.file("keep.txt", b"same bytes");
    let remove = f.file("copy.txt", b"same bytes");
    f.file("different.txt", b"other data");
    f.index();
    let r = f.report();
    assert_eq!(r.groups.len(), 1);
    assert_eq!(r.groups[0].files.len(), 2);
    let plan = f.preview(&r, &keep, &remove);
    assert!(plan.duplicate_cleanup && !plan.project_cleanup);
    let trash = f.dir.path().canonicalize().unwrap().join("trash-copy");
    let result = cleanup::execute(
        &f.c,
        approve(&plan),
        &f.policy(),
        &AtomicBool::new(false),
        |p| {
            fs::rename(p, &trash)?;
            Ok(trash.clone())
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(result.status, "complete", "{result:?}");
    assert_eq!(result.items[0].status, "movedToTrash");
    assert_eq!(fs::read(&keep).unwrap(), b"same bytes");
    assert_eq!(fs::read(&trash).unwrap(), b"same bytes");
    assert!(!PathBuf::from(remove).exists());
    assert!(cleanup::execute(
        &f.c,
        approve(&plan),
        &f.policy(),
        &AtomicBool::new(false),
        |_| panic!("Replay"),
        |_| true
    )
    .is_err());
}
#[test]
fn changed_keeper_new_exclusions_and_project_boundary_block_existing_duplicate_approval() {
    for condition in ["keeper", "exclusion", "project"] {
        let mut f = Fixture::new();
        let k = f.file("a", b"identical");
        let r = f.file("b", b"identical");
        f.index();
        let report = f.report();
        let plan = f.preview(&report, &k, &r);
        let mut policy = f.policy();
        if condition == "keeper" {
            fs::write(&k, b"changed!!").unwrap()
        }
        if condition == "exclusion" {
            policy.exclusions.push(k.clone().into())
        }
        if condition == "project" {
            fs::write(f.home.join("Documents/package.json"), "{}").unwrap()
        }
        let result = cleanup::execute(
            &f.c,
            approve(&plan),
            &policy,
            &AtomicBool::new(false),
            |_| panic!("Must revalidate keeper and scope"),
            |_| true,
        )
        .unwrap();
        assert_eq!(result.items[0].status, "skipped", "{condition}");
        assert!(PathBuf::from(r).exists());
    }
}
#[test]
fn keeper_changed_during_native_operation_is_unverified() {
    let mut f = Fixture::new();
    let k = f.file("a", b"identical");
    let r = f.file("b", b"identical");
    f.index();
    let report = f.report();
    let plan = f.preview(&report, &k, &r);
    let dest = f.dir.path().canonicalize().unwrap().join("trash");
    let result = cleanup::execute(
        &f.c,
        approve(&plan),
        &f.policy(),
        &AtomicBool::new(false),
        |p| {
            fs::rename(p, &dest)?;
            fs::write(&k, b"changed!!")?;
            Ok(dest.clone())
        },
        |_| true,
    )
    .unwrap();
    assert_eq!(result.status, "partial");
    assert_eq!(result.items[0].status, "unverified");
}
#[test]
fn duplicate_selection_cannot_remove_keeper_unconfirmed_or_project_files() {
    let mut f = Fixture::new();
    let k = f.file("a", b"identical");
    let r = f.file("b", b"identical");
    f.index();
    let report = f.report();
    let request = |keep: &str, paths: Vec<String>| DuplicateCleanupRequest {
        report_id: report.id.clone(),
        group_id: report.groups[0].id.clone(),
        keep_path: keep.into(),
        remove_paths: paths,
    };
    for req in [
        request(&k, vec![k.clone()]),
        request("/missing", vec![r.clone()]),
        request(&k, vec![r.clone(), r.clone()]),
        request(&k, vec!["/unconfirmed".into()]),
    ] {
        assert!(
            cleanup::preview_duplicates(&f.c, req, &f.policy(), &AtomicBool::new(false)).is_err()
        );
    }
    fs::write(f.home.join("Documents/Cargo.toml"), "[package]").unwrap();
    assert!(cleanup::preview_duplicates(
        &f.c,
        request(&k, vec![r]),
        &f.policy(),
        &AtomicBool::new(false)
    )
    .is_err());
}
#[test]
fn links_disappearing_files_exclusions_and_cancellation_are_qualified() {
    let mut f = Fixture::new();
    let k = f.file("a", b"identical");
    let gone = f.file("b", b"identical");
    let other = f.file("c", b"identical");
    symlink(&k, f.home.join("Documents/link")).unwrap();
    f.index();
    fs::remove_file(gone).unwrap();
    let mut policy = f.policy();
    policy.exclusions.push(other.into());
    let r = duplicates::analyze(&f.c, "scan", &policy, &AtomicBool::new(false), |_| true).unwrap();
    assert_eq!(r.status, "partial");
    assert!(r.groups.is_empty());
    assert!(!r.gaps.is_empty());
    let r = duplicates::analyze(&f.c, "scan", &policy, &AtomicBool::new(true), |_| true).unwrap();
    assert_eq!(r.status, "cancelled");
    assert_eq!(r.examined, 0);
}
#[test]
fn photo_similarity_never_acquires_exact_duplicate_authority() {
    let mut f = Fixture::new();
    let mut img = image::GrayImage::from_fn(64, 64, |x, y| image::Luma([((x + y) % 256) as u8]));
    let a = f.home.join("Documents/photo.png");
    img.save(&a).unwrap();
    img.put_pixel(0, 0, image::Luma([180]));
    img.save(f.home.join("Documents/modified.png")).unwrap();
    f.index();
    let r = f.report();
    assert!(r.groups.is_empty());
    assert_eq!(r.photo_matches.len(), 1);
    assert!(r.photo_matches[0].qualification.contains("No cleanup"));
}
fn rule() -> MaintenanceRule {
    MaintenanceRule {
        enabled: true,
        mode: "reminder".into(),
        interval_days: 7,
        next_due_at: "2020-01-01T00:00:00Z".into(),
        scope: ScanRequest {
            plan_id: None,
            mode: "standard".into(),
            roots: vec![],
            excluded_paths: vec!["/exclude".into()],
        },
        capacity_threshold_percent: 90,
    }
}
#[test]
fn maintenance_is_opt_in_bounded_and_coalesces_missed_runs() {
    let mut f = Fixture::new();
    assert!(maintenance::due(&mut f.c).unwrap().is_none());
    let mut r = rule();
    r.enabled = false;
    maintenance::save(&f.c, &r).unwrap();
    assert!(maintenance::due(&mut f.c).unwrap().is_none());
    r.enabled = true;
    maintenance::save(&f.c, &r).unwrap();
    let (saved, event) = maintenance::due(&mut f.c).unwrap().unwrap();
    assert_eq!(event.status, "reminder");
    assert!(saved.next_due_at > now());
    assert!(maintenance::due(&mut f.c).unwrap().is_none());
    assert_eq!(
        records::list(&f.c, Some("maintenance"), 0).unwrap().len(),
        1
    );
    r.scope.mode = "deep".into();
    assert!(maintenance::save(&f.c, &r).is_err());
    r.scope.mode = "standard".into();
    r.interval_days = 0;
    assert!(maintenance::save(&f.c, &r).is_err());
}
#[test]
fn local_assistant_clarifies_then_cites_retained_evidence_and_never_executes() {
    let mut f = Fixture::new();
    f.file("a", b"preserve");
    f.index();
    let missing = assistant::ask(&f.c, None, None, "slow startup").unwrap();
    assert!(missing.tasks.is_empty());
    assert!(missing.messages.last().unwrap().text.contains("scan"));
    let first = assistant::ask(&f.c, None, Some("scan"), "slow startup").unwrap();
    assert_eq!(first.status, "clarification");
    let next = assistant::ask(&f.c, Some(&first.id), None, "after login").unwrap();
    assert_eq!(next.tasks[0].name, "ReviewStartupConfiguration");
    assert_eq!(next.messages.len(), 4);
    let storage = assistant::ask(&f.c, None, Some("scan"), "review storage").unwrap();
    assert_eq!(storage.tasks[0].name, "ReviewStorageEvidence");
    assert!(storage.actions.contains(&"Optimization".into()));
    assert_eq!(
        records::get::<Investigation>(&f.c, "investigation", &next.id)
            .unwrap()
            .messages
            .len(),
        4
    );
    assert_eq!(fs::read(f.home.join("Documents/a")).unwrap(), b"preserve");
}

#[test]
fn requested_photo_previews_are_bounded_scoped_and_do_not_change_sources_or_records() {
    let mut f = Fixture::new();
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::new(800, 600));
    let mut encoded = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let bytes = encoded.into_inner();
    let a = f.file("preview-a.png", &bytes);
    let _b = f.file("preview-b.png", &bytes);
    f.index();
    let report = f.report();
    let before = f.c.total_changes();
    let preview =
        duplicates::preview_photo(&f.c, &report.id, &a, &f.policy(), &AtomicBool::new(false))
            .unwrap();
    assert_eq!(preview.width, 512);
    assert!(preview.height <= 512);
    assert!(!preview.png_bytes.is_empty());
    assert_eq!(f.c.total_changes(), before);
    assert_eq!(fs::read(&a).unwrap(), bytes);
    assert!(preview
        .qualification
        .contains("not a renewed equality check"));
    assert!(duplicates::preview_photo(
        &f.c,
        &report.id,
        "/unrelated.png",
        &f.policy(),
        &AtomicBool::new(false)
    )
    .is_err());
    let mut excluded = f.policy();
    excluded.exclusions.push(PathBuf::from(&a));
    assert!(
        duplicates::preview_photo(&f.c, &report.id, &a, &excluded, &AtomicBool::new(false))
            .is_err()
    );
    assert!(
        duplicates::preview_photo(&f.c, &report.id, &a, &f.policy(), &AtomicBool::new(true))
            .is_err()
    );
    fs::remove_file(&a).unwrap();
    symlink(&_b, &a).unwrap();
    assert!(
        duplicates::preview_photo(&f.c, &report.id, &a, &f.policy(), &AtomicBool::new(false))
            .is_err()
    );
}
fn advice_request() -> AdviceRequest {
    AdviceRequest {
        session_id: "scan".into(),
        baseline_id: None,
        trial_id: None,
        workload: "Build".into(),
        expectation: "under 2 minutes".into(),
        annual_growth_percent: 10.,
        current_demand_percent: 60.,
        quote_low: None,
        quote_high: None,
        quote_source: "".into(),
    }
}
#[test]
fn advice_separates_assumptions_absent_baselines_health_support_and_costs() {
    let mut f = Fixture::new();
    f.index();
    let a = advice::assess(&f.c, advice_request()).unwrap();
    assert!(a.assessment.contains("measurements"));
    assert!(a.findings.is_empty());
    assert_eq!(a.source_reviews.len(), a.sources.len());
    assert!(a.source_reviews.iter().all(|s| s.publisher == "Apple"
        && s.reviewed_on.as_deref() == Some("2026-10-23")
        && a.sources.contains(&s.url)));
    assert!(a
        .source_reviews
        .iter()
        .any(|s| s.title.contains("Ventura") && s.scope.contains("Ventura-specific")));
    assert!(a
        .verification_limits
        .iter()
        .any(|s| s.starts_with("Software and peripheral requirements:")));
    let saved: AdviceReport = records::get(&f.c, "advice", &a.id).unwrap();
    assert_eq!(
        saved.source_reviews[0].reviewed_on,
        a.source_reviews[0].reviewed_on
    );
    assert_eq!(saved.verification_limits, a.verification_limits);
    let mut legacy = serde_json::to_value(&a).unwrap();
    legacy.as_object_mut().unwrap().remove("sourceReviews");
    legacy.as_object_mut().unwrap().remove("verificationLimits");
    let legacy: AdviceReport = serde_json::from_value(legacy).unwrap();
    assert!(legacy.source_reviews.is_empty() && legacy.verification_limits.is_empty());
    assert_eq!(
        a.scenarios[0].source,
        "User-supplied demand model: demand × (1 + annual growth)^years"
    );
    assert!(a
        .scenarios
        .iter()
        .any(|e| e.label == "Upgrade cost" && e.state != CapabilityState::Supported));
    let mut r = advice_request();
    r.current_demand_percent = f64::NAN;
    assert!(advice::assess(&f.c, r).is_err());
    let mut r = advice_request();
    r.quote_low = Some(100.);
    r.quote_high = Some(50.);
    r.quote_source = "USD fixture".into();
    assert!(advice::assess(&f.c, r).is_err());
}
#[test]
fn retained_records_migrate_recover_paginate_expire_and_purge_without_losing_schedule() {
    let f = Fixture::new();
    let rule = rule();
    maintenance::save(&f.c, &rule).unwrap();
    for n in 0..55 {
        let value = serde_json::json!({"id":format!("{n}"),"status":"running"});
        records::save(
            &f.c,
            "test",
            &format!("{n}"),
            &now(),
            "running",
            "test",
            &value,
        )
        .unwrap();
    }
    assert_eq!(records::list(&f.c, None, 0).unwrap().len(), 50);
    assert_eq!(records::list(&f.c, None, 50).unwrap().len(), 5);
    database::recover_interrupted(&f.c).unwrap();
    let value: serde_json::Value = records::get(&f.c, "test", "0").unwrap();
    assert_eq!(value["status"], "interrupted");
    records::save(
        &f.c,
        "test",
        "old",
        "2000-01-01T00:00:00Z",
        "complete",
        "old",
        &value,
    )
    .unwrap();
    database::retain(&f.c, 30).unwrap();
    assert!(records::get::<serde_json::Value>(&f.c, "test", "old").is_err());
    database::purge(&f.c).unwrap();
    assert!(records::list(&f.c, None, 0).unwrap().is_empty());
    assert!(maintenance::get(&f.c).unwrap().unwrap().enabled);
}
#[test]
fn cancelled_baseline_is_retained_without_fabricated_summary() {
    let f = Fixture::new();
    let request = BaselineRequest {
        workload: "Test".into(),
        target: "Test target".into(),
        context: "Fixture".into(),
        duration_seconds: 6,
    };
    let b = baseline::capture(&f.c, request, &AtomicBool::new(true), |_| true).unwrap();
    assert_eq!(b.status, "cancelled");
    assert!(b.samples.is_empty() && b.summary.is_empty());
    assert_eq!(baseline::get(&f.c, &b.id).unwrap().status, "cancelled");
    assert!(baseline::compare(&f.c, &b.id, &b.id).is_err());
}
#[test]
fn real_baseline_interval_is_bounded_and_manual_decline_claims_no_change() {
    let f = Fixture::new();
    let request = BaselineRequest {
        workload: "Fixture idle".into(),
        target: "Observe".into(),
        context: "No controlled workload".into(),
        duration_seconds: 6,
    };
    let b = baseline::capture(&f.c, request, &AtomicBool::new(false), |_| true).unwrap();
    assert_eq!(b.status, "complete");
    assert!(b.samples.len() >= 3);
    assert!(b
        .samples
        .iter()
        .all(|s| (0.0..=100.0).contains(&s.cpu_percent)));
    assert!(!b.summary.is_empty());
    let p = baseline::plan(&f.c, "power", "Inspect Balanced mode").unwrap();
    assert_eq!(p.status, "awaitingManualAction");
    let p = baseline::verify_plan(&f.c, &p.id, false).unwrap();
    assert_eq!(p.status, "declined");
    assert!(p.verification.contains("No system change"));
    assert!(baseline::verify_plan(&f.c, &p.id, true).is_err());
}
#[test]
fn resource_review_does_not_authorize_unknown_leftovers_or_native_resources() {
    let mut f = Fixture::new();
    f.file("project/node_modules/cache", b"unique");
    f.index();
    let r = resources::analyze(&f.c, "scan", &[], &AtomicBool::new(false)).unwrap();
    assert!(r
        .capabilities
        .iter()
        .any(|c| c.name.contains("Docker") && c.state == CapabilityState::ManualOnly));
    assert!(r
        .capabilities
        .iter()
        .any(|c| c.name.contains("leftovers") && c.state == CapabilityState::Unknown));
    assert!(!r.evidence.is_empty());
    assert!(cleanup::history(&f.c, 0, 20).unwrap().is_empty());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "Explicit native adapter check: only a newly-created disposable duplicate fixture is moved to Trash and restored"]
fn native_duplicate_trash_verifies_keeper_and_restores_fixture() {
    let mut f = Fixture::new();
    let keeper = f.file("keep", b"native disposable duplicate");
    let remove = f.file("copy", b"native disposable duplicate");
    f.index();
    let report = f.report();
    let plan = f.preview(&report, &keeper, &remove);
    let result = cleanup::execute(
        &f.c,
        approve(&plan),
        &f.policy(),
        &AtomicBool::new(false),
        cleanup::native_trash,
        |_| true,
    )
    .unwrap();
    if let Some(path) = &result.items[0].trash_path {
        if std::path::Path::new(path).exists() && !std::path::Path::new(&remove).exists() {
            fs::rename(path, &remove).unwrap();
        }
    }
    assert_eq!(result.status, "complete", "{result:?}");
    assert_eq!(fs::read(&keeper).unwrap(), b"native disposable duplicate");
    assert_eq!(fs::read(&remove).unwrap(), b"native disposable duplicate");
}
#[test]
fn public_bundle_ids_are_bounded_and_do_not_follow_links() {
    let f = Fixture::new();
    let app = f.home.join("Example.app");
    fs::create_dir_all(app.join("Contents")).unwrap();
    let info = app.join("Contents/Info.plist");
    fs::write(&info,b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleIdentifier</key><string>org.coqui.fixture</string></dict></plist>").unwrap();
    assert_eq!(
        coqui_engine::platform::bundle_identifier(&app).as_deref(),
        Some("org.coqui.fixture")
    );
    fs::remove_file(&info).unwrap();
    let outside = f.home.join("outside.plist");
    fs::write(&outside, "unique fixture").unwrap();
    symlink(&outside, &info).unwrap();
    assert!(coqui_engine::platform::bundle_identifier(&app).is_none());
}
#[test]
fn clone_relationships_are_qualified_and_credentials_are_not_retained() {
    let mut f = Fixture::new();
    for name in ["project-one", "project-two"] {
        let root = f.home.join("Documents").join(name);
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(root.join(".gitignore"), "/target/\n").unwrap();
        fs::write(root.join("target/cache"), "fixture").unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["add", "Cargo.toml", ".gitignore"],
            vec!["commit", "-m", "fixture"],
            vec![
                "remote",
                "add",
                "origin",
                "https://user:fixture-secret@example.invalid/project.git",
            ],
        ] {
            let out = std::process::Command::new("/usr/bin/git")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@invalid",
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-C",
                ])
                .arg(&root)
                .args(args)
                .output()
                .unwrap();
            assert!(out.status.success());
        }
    }
    f.index();
    let report =
        coqui_engine::developer::analyze(&f.c, "scan", &[], &AtomicBool::new(false), |_| true)
            .unwrap();
    assert_eq!(report.artifacts.len(), 2);
    let r = resources::analyze(&f.c, "scan", &[], &AtomicBool::new(false)).unwrap();
    assert!(r
        .evidence
        .iter()
        .any(|e| e.label == "Possible related clones" && e.state == CapabilityState::Unknown));
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("fixture-secret") && !json.contains("example.invalid"));
    let r = resources::analyze(
        &f.c,
        "scan",
        &[f.home.join("Documents/project-one/.git")],
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(!r
        .evidence
        .iter()
        .any(|e| e.label == "Possible related clones"));
}

#[test]
fn content_changed_between_hashing_and_group_verification_is_not_labeled_with_a_stale_digest() {
    use sha2::{Digest, Sha256};
    let mut f = Fixture::new();
    let paths = (0..20)
        .map(|n| f.file(&format!("copy-{n:02}"), b"same bytes"))
        .collect::<Vec<_>>();
    f.index();
    let mut changed = false;
    let report = duplicates::analyze(&f.c, "scan", &f.policy(), &AtomicBool::new(false), |r| {
        if r.examined >= 10 && r.status == "running" && !changed {
            for p in &paths {
                fs::write(p, b"after edit").unwrap();
            }
            changed = true;
        }
        true
    })
    .unwrap();
    assert!(changed);
    assert_eq!(report.status, "partial");
    for g in &report.groups {
        for file in &g.files {
            assert_eq!(
                g.digest,
                format!("{:x}", Sha256::digest(fs::read(&file.path).unwrap()))
            );
        }
    }
    assert!(!report.gaps.is_empty());
}
