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

/// Optional application-update transport. It never follows redirects or obtains
/// credentials, downloads packages, or launches returned source content.
#[cfg(target_os = "macos")]
pub(crate) fn fetch_app_update(
    target: &crate::app_updates::FetchTarget,
    cancel: &std::sync::atomic::AtomicBool,
) -> crate::app_updates::FetchResult {
    use crate::app_updates::{public_address, FetchFailure, FetchResponse, MAX_RESPONSE_BYTES};
    use std::{net::IpAddr, sync::atomic::Ordering};
    let deadline = Instant::now() + Duration::from_secs(15);
    // Pinning a destination cannot safely delegate DNS to a configured proxy.
    // Preserve that policy by withholding this direct checker, never bypassing it.
    if [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ]
    .iter()
    .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()))
    {
        return Err(FetchFailure::Unavailable);
    }
    let proxies = update_read_command(
        "/usr/sbin/scutil",
        &["--proxy".into()],
        64 * 1024,
        (Instant::now() + Duration::from_secs(3)).min(deadline),
        cancel,
    )?;
    if !update_proxy_configuration_clear(&proxies) {
        return Err(FetchFailure::Unavailable);
    }
    let resolved = update_read_command(
        "/usr/bin/dscacheutil",
        &[
            "-q".into(),
            "host".into(),
            "-a".into(),
            "name".into(),
            target.host().into(),
        ],
        64 * 1024,
        (Instant::now() + Duration::from_secs(5)).min(deadline),
        cancel,
    )?;
    let resolved = std::str::from_utf8(&resolved).map_err(|_| FetchFailure::Unavailable)?;
    let mut addresses = Vec::new();
    for line in resolved.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim() == "ip_address" || key.trim() == "ipv6_address" {
            let address: IpAddr = value
                .trim()
                .parse()
                .map_err(|_| FetchFailure::Unavailable)?;
            if !public_address(address) {
                return Err(FetchFailure::Unavailable);
            }
            if !addresses.contains(&address) {
                addresses.push(address);
            }
        }
    }
    if cancel.load(Ordering::SeqCst) {
        return Err(FetchFailure::Cancelled);
    }
    // Keep the complete validated set in one pinned DNS entry. Selecting only
    // the first IPv6 address prevents curl's normal IPv4/IPv6 fallback when that
    // family is unreachable, even though DNS supplied a reachable public peer.
    let pin = update_resolve_pin(target.host(), &addresses)?;
    let remaining = deadline
        .saturating_duration_since(Instant::now())
        .as_secs_f64();
    if remaining <= 0. {
        return Err(FetchFailure::Unavailable);
    }
    let args = vec![
        "-q".into(),
        "--silent".into(),
        "--show-error".into(),
        "--proto".into(),
        "=https".into(),
        "--proto-redir".into(),
        "=https".into(),
        "--max-redirs".into(),
        "0".into(),
        "--connect-timeout".into(),
        "5".into(),
        "--max-time".into(),
        format!("{remaining:.3}"),
        "--max-filesize".into(),
        MAX_RESPONSE_BYTES.to_string(),
        "--resolve".into(),
        pin,
        "--request".into(),
        "GET".into(),
        "--user-agent".into(),
        "CoquiPCCleaner".into(),
        "--write-out".into(),
        "\n%{http_code}".into(),
        "--url".into(),
        target.url().into(),
    ];
    let output = update_read_command(
        "/usr/bin/curl",
        &args,
        MAX_RESPONSE_BYTES + 4,
        deadline,
        cancel,
    )?;
    let split = output
        .len()
        .checked_sub(4)
        .ok_or(FetchFailure::Unavailable)?;
    if output[split] != b'\n' {
        return Err(FetchFailure::Unavailable);
    }
    let status = std::str::from_utf8(&output[split + 1..])
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or(FetchFailure::Unavailable)?;
    Ok(FetchResponse {
        status,
        body: output[..split].to_vec(),
    })
}

#[cfg(target_os = "macos")]
fn update_resolve_pin(
    host: &str,
    addresses: &[std::net::IpAddr],
) -> std::result::Result<String, crate::app_updates::FetchFailure> {
    use crate::app_updates::{public_address, FetchFailure};
    use std::net::IpAddr;
    if addresses.is_empty() || addresses.iter().any(|address| !public_address(*address)) {
        return Err(FetchFailure::Unavailable);
    }
    let peers = addresses
        .iter()
        .map(|address| match address {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => format!("[{ip}]"),
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{host}:443:{peers}"))
}

#[cfg(target_os = "macos")]
fn update_proxy_configuration_clear(bytes: &[u8]) -> bool {
    let Ok(source) = std::str::from_utf8(bytes) else {
        return false;
    };
    let lines = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.first() != Some(&"<dictionary> {") || lines.last() != Some(&"}") {
        return false;
    }
    let mut in_exceptions = false;
    for line in &lines[1..lines.len() - 1] {
        if *line == "ExceptionsList : <array> {" && !in_exceptions {
            in_exceptions = true;
            continue;
        }
        if *line == "}" && in_exceptions {
            in_exceptions = false;
            continue;
        }
        let Some((key, value)) = line.split_once(" : ") else {
            return false;
        };
        if in_exceptions {
            if key.parse::<u32>().is_err() || value.is_empty() || value.contains(['{', '}']) {
                return false;
            }
            continue;
        }
        match key {
            "HTTPEnable"
            | "HTTPSEnable"
            | "SOCKSEnable"
            | "FTPEnable"
            | "ProxyAutoConfigEnable"
            | "ProxyAutoDiscoveryEnable" => {
                if value != "0" {
                    return false;
                }
            }
            "FTPPassive" | "ExcludeSimpleHostnames" => {
                if value != "0" && value != "1" {
                    return false;
                }
            }
            _ => return false,
        }
    }
    !in_exceptions
}

#[cfg(all(test, target_os = "macos"))]
mod update_transport_tests {
    use super::*;
    #[test]
    fn proxy_policy_is_never_bypassed_and_unknown_outputs_fail_closed() {
        assert!(update_proxy_configuration_clear(b"<dictionary> {\n  ExceptionsList : <array> {\n    0 : *.local\n    1 : 169.254/16\n  }\n  FTPPassive : 1\n}"));
        assert!(update_proxy_configuration_clear(
            b"<dictionary> {\nHTTPSEnable : 0\n}"
        ));
        for field in [
            "HTTPEnable",
            "HTTPSEnable",
            "SOCKSEnable",
            "ProxyAutoConfigEnable",
            "ProxyAutoDiscoveryEnable",
        ] {
            assert!(!update_proxy_configuration_clear(
                format!("<dictionary> {{\n{field} : 1\n}}").as_bytes()
            ));
        }
        for output in [
            b"".as_slice(),
            b"unexpected",
            b"<dictionary> {\nHTTPProxy : secret.internal\n}",
            b"<dictionary> {\n__SCOPED__ : <dictionary> {\n}\n}",
            b"<dictionary> {\nHTTPSEnable : perhaps\n}",
        ] {
            assert!(!update_proxy_configuration_clear(output));
        }
    }
    #[test]
    fn complete_public_dns_set_preserves_dual_stack_fallback_without_unpinning() {
        let peers = [
            "2600:1406:5e00:39d::2a1".parse().unwrap(),
            "2600:1406:5e00:384::2a1".parse().unwrap(),
            "23.61.212.30".parse().unwrap(),
        ];
        assert_eq!(
            update_resolve_pin("itunes.apple.com", &peers).unwrap(),
            "itunes.apple.com:443:[2600:1406:5e00:39d::2a1],[2600:1406:5e00:384::2a1],23.61.212.30"
        );
        assert!(update_resolve_pin("itunes.apple.com", &[]).is_err());
        let mixed = [
            "23.61.212.30".parse().unwrap(),
            "127.0.0.1".parse().unwrap(),
        ];
        assert!(update_resolve_pin("itunes.apple.com", &mixed).is_err());
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn fetch_app_update(
    _: &crate::app_updates::FetchTarget,
    _: &std::sync::atomic::AtomicBool,
) -> crate::app_updates::FetchResult {
    Err(crate::app_updates::FetchFailure::Unavailable)
}

#[cfg(target_os = "macos")]
fn update_read_command(
    program: &str,
    args: &[String],
    limit: usize,
    deadline: Instant,
    cancel: &std::sync::atomic::AtomicBool,
) -> std::result::Result<Vec<u8>, crate::app_updates::FetchFailure> {
    use crate::app_updates::FetchFailure;
    use std::{os::fd::AsRawFd, sync::atomic::Ordering};
    if cancel.load(Ordering::SeqCst) {
        return Err(FetchFailure::Cancelled);
    }
    if ![
        "/usr/bin/curl",
        "/usr/bin/dscacheutil",
        "/usr/sbin/scutil",
        "/usr/sbin/system_profiler",
    ]
    .contains(&program)
        || Instant::now() >= deadline
    {
        return Err(FetchFailure::Unavailable);
    }
    let mut child = Command::new(program)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| FetchFailure::Unavailable)?;
    let Some(mut output) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(FetchFailure::Unavailable);
    };
    let fd = output.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        let _ = child.kill();
        let _ = child.wait();
        return Err(FetchFailure::Unavailable);
    }
    let mut bytes = Vec::new();
    let result = loop {
        if cancel.load(Ordering::SeqCst) {
            break Err(FetchFailure::Cancelled);
        }
        if Instant::now() >= deadline {
            break Err(FetchFailure::Unavailable);
        }
        let drain =
            |output: &mut dyn Read, bytes: &mut Vec<u8>| -> std::result::Result<(), FetchFailure> {
                let mut buffer = [0u8; 8192];
                for _ in 0..16 {
                    match output.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            bytes.extend_from_slice(&buffer[..n]);
                            if bytes.len() > limit {
                                return Err(FetchFailure::Unavailable);
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(_) => return Err(FetchFailure::Unavailable),
                    }
                }
                Ok(())
            };
        if let Err(error) = drain(&mut output, &mut bytes) {
            break Err(error);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if let Err(error) = drain(&mut output, &mut bytes) {
                    break Err(error);
                }
                break if status.success() {
                    Ok(bytes)
                } else {
                    Err(FetchFailure::Unavailable)
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => break Err(FetchFailure::Unavailable),
        }
    };
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    result
}

pub(crate) fn open_app_update_handoff(handoff: crate::app_updates::Handoff) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("/usr/bin/open");
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        match handoff {
            crate::app_updates::Handoff::AppStore => {
                command.arg("macappstore://showUpdatesPage");
            }
            crate::app_updates::Handoff::VendorApplication(path) => {
                command.arg("-a").arg(path);
            }
        }
        anyhow::ensure!(
            command.status()?.success(),
            "Native update handoff was unavailable"
        );
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = handoff;
        anyhow::bail!("Native application-update handoff requires macOS")
    }
}

/// Space available to this process on the filesystem that will hold scan metadata.
pub(crate) fn available_storage(path: &Path) -> Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(path.as_os_str().as_bytes())?;
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stats) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    (stats.f_bavail as u64)
        .checked_mul(stats.f_frsize as u64)
        .filter(|_| stats.f_frsize > 0)
        .context("Available scan storage could not be measured")
}

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
        "/usr/bin/pmset",
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

/// Bounded, cancellable system-only observation for the explicit scan workflow.
/// Startup/app metadata are separately selected checks, never implicit here.
pub fn scan_system(cancel: &std::sync::atomic::AtomicBool) -> Result<MachineSnapshot> {
    use std::sync::atomic::Ordering;
    anyhow::ensure!(!cancel.load(Ordering::SeqCst), "System check cancelled");
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_list(sysinfo::CpuRefreshKind::everything());
    anyhow::ensure!(
        sys.total_memory() > 0 && !sys.cpus().is_empty(),
        "Basic system inventory unavailable"
    );
    let mut model = System::name().unwrap_or_else(|| "Unavailable".into());
    let mut gpu = "Unavailable".to_owned();
    let mut capabilities = vec![];
    #[cfg(target_os = "macos")]
    {
        let output = update_read_command(
            "/usr/sbin/system_profiler",
            &[
                "-json".into(),
                "-detailLevel".into(),
                "mini".into(),
                "SPHardwareDataType".into(),
                "SPDisplaysDataType".into(),
            ],
            4 * 1024 * 1024,
            Instant::now() + Duration::from_secs(12),
            cancel,
        );
        match output.ok().and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()) {
            Some(value) => {
                if let Some(value) = json_string(&value, &["machine_model"]) { model = value; }
                if let Some(display) = value.get("SPDisplaysDataType") { gpu = json_string(display, &["sppci_model", "_name"]).unwrap_or(gpu); }
            }
            None => capabilities.push(SystemCapability { name: "Extended inventory".into(), state: CapabilityState::TemporarilyUnavailable, explanation: "The bounded hardware report was unavailable, timed out or cancelled. Basic observed fields remain available.".into() }),
        }
    }
    anyhow::ensure!(!cancel.load(Ordering::SeqCst), "System check cancelled");
    capabilities.push(SystemCapability { name: "Scan check scope".into(), state: CapabilityState::ManualOnly, explanation: "System details only. Startup and application metadata have separate recorded check states. Firmware/security audits, sensor readings and network updates are separate tools.".into() });
    Ok(MachineSnapshot {
        observed_at: now(),
        device_name: System::host_name().unwrap_or_else(|| "This computer".into()),
        model,
        os_version: System::long_os_version().unwrap_or_else(|| "Unavailable".into()),
        architecture: std::env::consts::ARCH.into(),
        cpu: sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Unavailable".into()),
        core_count: sys.cpus().len() as u32,
        memory_bytes: sys.total_memory() as f64,
        gpu,
        disks: Disks::new_with_refreshed_list()
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                mount: disk.mount_point().to_string_lossy().into_owned(),
                filesystem: disk.file_system().to_string_lossy().into_owned(),
                total_bytes: disk.total_space() as f64,
                available_bytes: disk.available_space() as f64,
                removable: disk.is_removable(),
            })
            .collect(),
        evidence: vec![DiagnosticEvidence::observed(
            "Installed memory",
            sys.total_memory().to_string(),
            "bytes",
            "Operating-system inventory",
        )],
        capabilities,
        startup_items: vec![],
        applications: vec![],
    })
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

/// Bounded, non-following public bundle metadata; absence never proves an app is uninstalled.
pub fn bundle_identifier(bundle: &Path) -> Option<String> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let path = bundle.join("Contents/Info.plist");
    for p in path.ancestors() {
        let m = fs::symlink_metadata(p).ok()?;
        if m.file_type().is_symlink() || crate::storage::placeholder(p, &m) {
            return None;
        }
    }
    let m = fs::symlink_metadata(&path).ok()?;
    if !m.is_file() || m.len() > 256 * 1024 {
        return None;
    }
    let f = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&path)
        .ok()?;
    let before = f.metadata().ok()?;
    if before.dev() != m.dev() || before.ino() != m.ino() {
        return None;
    }
    let mut bytes = Vec::new();
    (&f).take(256 * 1024 + 1).read_to_end(&mut bytes).ok()?;
    let after = f.metadata().ok()?;
    if bytes.len() > 256 * 1024
        || before.mtime() != after.mtime()
        || before.ctime() != after.ctime()
    {
        return None;
    }
    let p = plist::Value::from_reader(std::io::Cursor::new(bytes)).ok()?;
    p.as_dictionary()?
        .get("CFBundleIdentifier")?
        .as_string()
        .filter(|s| !s.is_empty() && s.len() <= 256)
        .map(str::to_owned)
}
