//! Explicit local validation utility; never included as an app command or privileged endpoint.
use coqui_engine::{api, model::ScanRequest, volumes};
fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("--volumes") => {
            for v in volumes::discover()? {
                println!(
                    "{} · {} · internal={} · readOnly={}",
                    v.mount, v.filesystem, v.internal, v.read_only
                );
            }
        }
        Some("--whole-disk") => {
            let seconds = args
                .get(2)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60)
                .clamp(5, 600);
            let directory = temp_directory()?;
            api::init_engine(directory.to_string_lossy().into_owned())?;
            let plan = api::disk_scan_plan()?;
            let roots = plan
                .volumes
                .iter()
                .filter(|v| v.internal)
                .map(|v| v.mount.clone())
                .collect::<Vec<_>>();
            println!("Reviewed mounted internal roots: {}", roots.join("; "));
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(seconds));
                let _ = api::cancel_scan();
            });
            let mut last = std::time::Instant::now();
            api::scan(
                ScanRequest {
                    plan_id: Some(plan.id),
                    mode: "deep".into(),
                    roots,
                    excluded_paths: vec![directory.to_string_lossy().into_owned()],
                },
                |e| {
                    if last.elapsed().as_secs() >= 10 || e.session.status != "running" {
                        println!(
                            "{} · {} files · {} omissions",
                            e.session.status, e.session.file_count, e.session.skipped_count
                        );
                        last = std::time::Instant::now();
                    }
                    if e.session.status != "running" {
                        for root in e.session.coverage {
                            println!(
                                "{} · {} · {} files · {:?}",
                                root.root, root.state, root.files, root.skips
                            );
                        }
                    }
                    true
                },
            )?;
            let history = api::scan_history(0, 1)?;
            anyhow::ensure!(!history.is_empty(), "No retained result");
            api::init_engine(directory.to_string_lossy().into_owned())?;
            anyhow::ensure!(
                api::scan_history(0, 1)?[0].id == history[0].id,
                "Restart lost retained result"
            );
            api::clear_diagnostic_data()?;
            anyhow::ensure!(api::scan_history(0, 1)?.is_empty(), "Purge failed");
            println!(
                "Restart, coverage retention and diagnostic purge verified in disposable data."
            );
            std::fs::remove_dir_all(directory)?;
        }
        _ => anyhow::bail!("Use --volumes or --whole-disk [5..600 seconds]"),
    }
    Ok(())
}
fn temp_directory() -> anyhow::Result<std::path::PathBuf> {
    let path =
        std::env::temp_dir().join(format!("coqui-native-validation-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&path)?;
    Ok(path.canonicalize()?)
}
