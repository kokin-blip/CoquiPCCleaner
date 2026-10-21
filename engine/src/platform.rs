use crate::model::*;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use sysinfo::{Disks, System};

pub fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("Home directory is unavailable")
}

/// All callers use fixed executable paths and literal arguments; never a shell.
pub fn command(program: &str, args: &[&str], timeout: Duration) -> Result<Vec<u8>> {
    const ALLOWED: &[&str] = &[
        "/usr/sbin/system_profiler",
        "/usr/sbin/diskutil",
        "/usr/bin/csrutil",
        "/usr/bin/fdesetup",
        "/usr/sbin/sysctl",
    ];
    anyhow::ensure!(ALLOWED.contains(&program), "Command is not allowlisted");
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().context("Missing command output")?;
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        stdout
            .take(4 * 1024 * 1024)
            .read_to_end(&mut out)
            .map(|_| out)
    });
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let output = reader
                .join()
                .map_err(|_| anyhow::anyhow!("Output reader stopped"))??;
            anyhow::ensure!(
                status.success(),
                "System utility did not return a successful result"
            );
            return Ok(output);
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            bail!("System utility timed out");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub fn json_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(m) => {
            for k in keys {
                if let Some(s) = m.get(*k).and_then(Value::as_str) {
                    if !s.is_empty() {
                        return Some(s.to_owned());
                    }
                }
            }
            m.values().find_map(|v| json_string(v, keys))
        }
        Value::Array(a) => a.iter().find_map(|v| json_string(v, keys)),
        _ => None,
    }
}

pub fn startup_from_plist(path: &Path, value: &plist::Value) -> Option<StartupItem> {
    let d = value.as_dictionary()?;
    let label = d.get("Label").and_then(plist::Value::as_string)?;
    let disabled = d.get("Disabled").and_then(plist::Value::as_boolean);
    Some(StartupItem {name:label.into(),path:path.to_string_lossy().into_owned(),mechanism:if path.to_string_lossy().contains("LaunchDaemons") {"LaunchDaemon"}else{"LaunchAgent"}.into(),state:if disabled==Some(true){"Disabled in plist; runtime override unknown"}else{"Configured; runtime enabled state unknown"}.into(),explanation:"Configuration detected on disk. This does not establish launchd runtime state, startup timing, or all modern login items.".into()})
}

pub fn snapshot() -> Result<MachineSnapshot> {
    let sys = System::new_all();
    let disks = Disks::new_with_refreshed_list()
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().into_owned(),
            mount: d.mount_point().to_string_lossy().into_owned(),
            filesystem: d.file_system().to_string_lossy().into_owned(),
            total_bytes: d.total_space() as f64,
            available_bytes: d.available_space() as f64,
            removable: d.is_removable(),
        })
        .collect::<Vec<_>>();
    let mut evidence = vec![
        DiagnosticEvidence::observed(
            "Architecture",
            std::env::consts::ARCH.into(),
            "",
            "Rust target",
        ),
        DiagnosticEvidence::observed(
            "Installed memory",
            sys.total_memory().to_string(),
            "bytes",
            "sysinfo / macOS",
        ),
    ];
    let mut capabilities = vec![SystemCapability {
        name: "Hardware inventory".into(),
        state: CapabilityState::Supported,
        explanation: "Read-only operating-system inventory. Unsupported fields remain unavailable."
            .into(),
    }];
    let mut model = System::name().unwrap_or_else(|| "Mac".into());
    let mut gpu = "Unavailable".to_owned();
    let mut cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_owned())
        .unwrap_or_else(|| "Unavailable".into());
    if cfg!(target_os = "macos") {
        match command("/usr/sbin/system_profiler",&["-json","-detailLevel","mini","SPHardwareDataType","SPDisplaysDataType","SPPowerDataType","SPNVMeDataType"],Duration::from_secs(35)).and_then(|v|Ok(serde_json::from_slice::<Value>(&v)?)) {
            Ok(v)=> {
                if let Some(s)=json_string(&v,&["machine_model"]){model=s;}
                if let Some(s)=json_string(&v,&["chip_type","cpu_type"]){cpu=s;}
                if let Some(display)=v.get("SPDisplaysDataType") {gpu=json_string(display,&["sppci_model","_name"]).unwrap_or(gpu);}
                for (label,keys) in [("System firmware",vec!["boot_rom_version","system_firmware_version"]),("OS loader",vec!["os_loader_version"]),("Internal SSD SMART status",vec!["smart_status"]),("Battery condition",vec!["sppower_battery_health","sppower_battery_condition"])] {
                    evidence.push(match json_string(&v,&keys) {Some(s)=>DiagnosticEvidence::observed(label,s,"","system_profiler (selected fields)"),None=>DiagnosticEvidence::unavailable(label,"Not exposed by this model or system report. Absence does not establish health.")});
                }
            },
            Err(_)=>capabilities.push(SystemCapability{name:"Extended inventory".into(),state:CapabilityState::TemporarilyUnavailable,explanation:"The system report was unavailable or timed out. Basic inventory remains available.".into()}),
        }
        for (label, prog, args, prefix) in [
            (
                "System Integrity Protection",
                "/usr/bin/csrutil",
                vec!["status"],
                "System Integrity Protection status:",
            ),
            (
                "FileVault",
                "/usr/bin/fdesetup",
                vec!["status"],
                "FileVault",
            ),
        ] {
            let reading = command(prog, &args, Duration::from_secs(4))
                .ok()
                .and_then(|o| String::from_utf8(o).ok())
                .and_then(|s| s.lines().find(|l| l.starts_with(prefix)).map(str::to_owned));
            evidence.push(match reading {Some(s)=>DiagnosticEvidence::observed(label,s,"",prog),None=>DiagnosticEvidence::unavailable(label,"The operating system did not expose a readable status. Review System Settings.")});
        }
    }
    for (name,explanation) in [("Secure Boot policy","Review Startup Security Utility or system information. No conventional PC BIOS controls are provided on Apple Silicon."),("Thermal sensors","No supported sensor reader is integrated. Temperatures are not estimated."),("GPU utilization","GPU identity is reported; utilization is not exposed by this adapter."),("SSD wear / remaining life","SMART status, if available, is a reported status rather than a wear forecast."),("Startup coverage","Readable launch configurations are listed. Modern registered login items and managed entries may require manual inspection.")] {
        capabilities.push(SystemCapability{name:name.into(),state:if name=="Secure Boot policy"||name=="Startup coverage"{CapabilityState::ManualOnly}else{CapabilityState::Unsupported},explanation:explanation.into()});
    }
    let mut startup_items = vec![];
    let user_home = home()?;
    for dir in [
        user_home.join("Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchDaemons"),
    ] {
        match fs::read_dir(&dir) {
            Ok(entries) => {
                for path in entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "plist"))
                {
                    if let Ok(value) = plist::Value::from_file(&path) {
                        if let Some(item) = startup_from_plist(&path, &value) {
                            startup_items.push(item);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => capabilities.push(SystemCapability {
                name: "Startup directory".into(),
                state: CapabilityState::RequiresPermission,
                explanation: format!("Cannot inspect {}", dir.display()),
            }),
        }
    }
    startup_items.sort_by(|a, b| a.name.cmp(&b.name));
    let mut applications = vec![];
    for root in [
        PathBuf::from("/Applications"),
        user_home.join("Applications"),
    ] {
        // Application bundles are leaves: do not descend into their contents.
        let mut iter = walkdir::WalkDir::new(root)
            .max_depth(3)
            .follow_links(false)
            .into_iter();
        while let Some(entry) = iter.next() {
            if let Ok(e) = entry {
                if e.path().extension().is_some_and(|x| x == "app") {
                    applications.push(InstalledApplication {
                        name: e
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        path: e.path().to_string_lossy().into_owned(),
                    });
                    iter.skip_current_dir();
                }
            }
        }
    }
    applications.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    applications.dedup_by(|a, b| a.path == b.path);
    Ok(MachineSnapshot {
        observed_at: now(),
        device_name: System::host_name().unwrap_or_else(|| "This Mac".into()),
        model,
        os_version: System::long_os_version().unwrap_or_else(|| "Unavailable".into()),
        architecture: std::env::consts::ARCH.into(),
        cpu,
        core_count: sys.cpus().len() as u32,
        memory_bytes: sys.total_memory() as f64,
        gpu,
        disks,
        evidence,
        capabilities,
        startup_items,
        applications,
    })
}

pub fn metrics(sys: &mut System) -> LiveMetrics {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let cores = sys.cpus().len().max(1) as f32;
    let mut processes = sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcessSample {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu_percent: (p.cpu_usage() / cores).clamp(0., 100.),
            memory_bytes: p.memory() as f64,
        })
        .collect::<Vec<_>>();
    let (mut reads, mut writes) = (0., 0.);
    for p in sys.processes().values() {
        let d = p.disk_usage();
        reads += d.read_bytes as f64;
        writes += d.written_bytes as f64;
    }
    processes.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
    processes.truncate(40);
    LiveMetrics {observed_at:now(),cpu_percent:sys.global_cpu_usage().clamp(0.,100.),used_memory_bytes:sys.used_memory() as f64,total_memory_bytes:sys.total_memory() as f64,swap_bytes:sys.used_swap() as f64,disk_read_bytes:reads,disk_written_bytes:writes,processes,qualification:"Two-second sampling. Process CPU is normalized across logical cores. I/O is the sum of readable process counters, not physical disk throughput. Used memory and swap alone do not establish a bottleneck.".into()}
}
