//! Optional, nonprivileged Apple Silicon audits. No install, reboot or firmware writer.
use crate::{model::*, records};
use anyhow::Result;
use rusqlite::Connection;
use serde_json::Value;
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub const UPDATE_GUIDE: &str = "https://support.apple.com/en-us/108382";
pub const POLICY_GUIDE: &str = "https://support.apple.com/guide/security/startup-disk-security-policy-control-sec7d92dc49f/web";
pub const RECOVERY_GUIDE: &str = "https://support.apple.com/en-us/108900";
pub const SOURCE_REVIEWED: &str = "2026-10-22";

#[derive(Clone, Copy, Debug, PartialEq)]
enum ReadCommand {
    Hardware,
    Version,
    Build,
    Sip,
    AuthenticatedRoot,
    FileVault,
    Updates,
}
impl ReadCommand {
    fn spec(self) -> (&'static str, &'static [&'static str], u64) {
        match self {
            Self::Hardware => (
                "/usr/sbin/system_profiler",
                &["-json", "-detailLevel", "mini", "SPHardwareDataType"],
                35,
            ),
            Self::Version => ("/usr/bin/sw_vers", &["-productVersion"], 4),
            Self::Build => ("/usr/bin/sw_vers", &["-buildVersion"], 4),
            Self::Sip => ("/usr/bin/csrutil", &["status"], 4),
            Self::AuthenticatedRoot => ("/usr/bin/csrutil", &["authenticated-root", "status"], 4),
            Self::FileVault => ("/usr/bin/fdesetup", &["status"], 4),
            Self::Updates => ("/usr/sbin/softwareupdate", &["--list"], 120),
        }
    }
    fn source(self) -> String {
        let (p, a, _) = self.spec();
        format!("{p} {}", a.join(" "))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum ReadFailure {
    Cancelled,
    Permission,
    Timeout,
    Failed,
    TooLarge,
}
impl ReadFailure {
    fn detail(self) -> &'static str {
        match self {
        Self::Cancelled => "Cancelled; no result established.",
        Self::Permission => "macOS denied access. Review the corresponding Apple interface or contact your administrator; Coqui requests no elevation.",
        Self::Timeout => "The utility timed out. Retry explicitly or use the Apple interface.",
        Self::Failed => "The utility failed or returned an unrecognized response. No healthy or current status is inferred.",
        Self::TooLarge => "The utility exceeded the bounded output limit; its result is unavailable.",
    }
    }
    fn state(self) -> CapabilityState {
        if self == Self::Permission {
            CapabilityState::RequiresPermission
        } else {
            CapabilityState::TemporarilyUnavailable
        }
    }
}

/// Nonblocking pipes prevent a blocked reader from defeating timeout/cancellation.
/// Only literal read operations above can reach this process launcher.
fn run(which: ReadCommand, cancel: &AtomicBool) -> std::result::Result<Vec<u8>, ReadFailure> {
    use std::os::fd::AsRawFd;
    if cancel.load(Ordering::SeqCst) {
        return Err(ReadFailure::Cancelled);
    }
    let (program, args, seconds) = which.spec();
    let mut cmd = Command::new(program);
    cmd.args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Ok(home) = crate::platform::home() {
        cmd.env("HOME", home);
    }
    let mut child = cmd.spawn().map_err(|_| ReadFailure::Failed)?;
    let mut out = child.stdout.take().ok_or(ReadFailure::Failed)?;
    let mut err = child.stderr.take().ok_or(ReadFailure::Failed)?;
    let flags = |fd| unsafe {
        let f = libc::fcntl(fd, libc::F_GETFL);
        f >= 0 && libc::fcntl(fd, libc::F_SETFL, f | libc::O_NONBLOCK) >= 0
    };
    if !flags(out.as_raw_fd()) || !flags(err.as_raw_fd()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(ReadFailure::Failed);
    }
    let start = Instant::now();
    let mut bytes = Vec::new();
    let mut errors = Vec::new();
    let result = loop {
        if cancel.load(Ordering::SeqCst) {
            break Err(ReadFailure::Cancelled);
        }
        if start.elapsed() >= Duration::from_secs(seconds) {
            break Err(ReadFailure::Timeout);
        }
        let drain = |r: &mut dyn Read, v: &mut Vec<u8>| -> std::result::Result<(), ReadFailure> {
            let mut buf = [0u8; 8192];
            // Bound each drain turn as well as the complete output.
            for _ in 0..16 {
                match r.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        v.extend_from_slice(&buf[..n]);
                        if v.len() > 1024 * 1024 {
                            return Err(ReadFailure::TooLarge);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => return Err(ReadFailure::Failed),
                }
            }
            Ok(())
        };
        if let Err(e) = drain(&mut out, &mut bytes).and_then(|_| drain(&mut err, &mut errors)) {
            break Err(e);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                // Consume bounded residual bytes after exit, without waiting for inherited pipes.
                if let Err(e) =
                    drain(&mut out, &mut bytes).and_then(|_| drain(&mut err, &mut errors))
                {
                    break Err(e);
                }
                if status.success() {
                    break Ok(bytes);
                }
                let e = String::from_utf8_lossy(&errors).to_ascii_lowercase();
                break Err(
                    if e.contains("permission")
                        || e.contains("not permitted")
                        || e.contains("must be root")
                        || e.contains("administrator")
                    {
                        ReadFailure::Permission
                    } else {
                        ReadFailure::Failed
                    },
                );
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(30)),
            Err(_) => break Err(ReadFailure::Failed),
        }
    };
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    result
}
fn text(bytes: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(bytes).ok()?.trim();
    (!s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)).then(|| s.to_owned())
}
fn field(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().and_then(|s| text(s.as_bytes()))
}
fn hardware(bytes: &[u8]) -> Option<Value> {
    let v: Value = serde_json::from_slice(bytes).ok()?;
    v.get("SPHardwareDataType")?
        .as_array()?
        .first()
        .filter(|v| v.is_object())
        .cloned()
}
fn compatibility(h: Option<&Value>, version: &str, build: &str) -> PlatformCompatibility {
    let model = h
        .and_then(|h| field(h, "machine_model"))
        .unwrap_or_else(|| "Unavailable".into());
    let chip = h
        .and_then(|h| field(h, "chip_type"))
        .unwrap_or_else(|| "Unavailable".into());
    let eligible = cfg!(target_os = "macos")
        && std::env::consts::ARCH == "aarch64"
        && chip.starts_with("Apple M")
        && model != "Unavailable"
        && version
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .is_some_and(|v| v >= 14);
    PlatformCompatibility { manufacturer:if chip.starts_with("Apple "){"Apple"}else{"Unconfirmed"}.into(),model,chip,architecture:std::env::consts::ARCH.into(),os_version:version.into(),os_build:build.into(),eligible,qualification:if eligible {"Apple Silicon/macOS 14+ adapter match from this run. Model-family compatibility is not proof that every property is exposed or physically tested; only Mac16,8 has local hardware validation."} else {"Model, Apple Silicon target or macOS 14+ eligibility could not be established. Advanced audits and update checks are withheld; use the model-appropriate Apple tools."}.into() }
}
fn capabilities(p: &PlatformCompatibility) -> Vec<SystemCapability> {
    [
        ("Firmware inventory",if p.eligible{CapabilityState::Supported}else{CapabilityState::Unknown},"Selected system_profiler fields only. A version is not an integrity or latest-firmware verdict."),
        ("Configuration audit",if p.eligible{CapabilityState::Supported}else{CapabilityState::Unknown},"SIP, authenticated-root and FileVault status, when readable. No comprehensive security certification."),
        ("Update availability",if p.eligible{CapabilityState::Supported}else{CapabilityState::Unknown},"Explicit network consent per run; softwareupdate --list only. Listings are macOS offerings, not a standalone firmware catalog."),
        ("Startup security policy",CapabilityState::ManualOnly,"Inspect the selected startup disk in Recovery. SIP and authenticated-root status do not establish Full or Reduced Security."),
        ("Firmware installation / recovery",CapabilityState::ManualOnly,"Apple-managed Software Update or model-specific recovery. Coqui has no firmware writer, install, restart or restore command."),
        ("PC BIOS / XMP / EXPO / firmware password",CapabilityState::Unsupported,"These controls do not apply to the Apple Silicon adapter. Intel and Windows require separate compatibility work."),
    ].into_iter().map(|(n,s,e)|SystemCapability{name:n.into(),state:s,explanation:e.into()}).collect()
}
fn unavailable(label: &str, command: ReadCommand, failure: ReadFailure) -> DiagnosticEvidence {
    let mut e = DiagnosticEvidence::unavailable(label, failure.detail());
    e.source = command.source();
    e.state = failure.state();
    e
}
fn security_reading(which: ReadCommand, bytes: &[u8]) -> DiagnosticEvidence {
    let (label, on, off) = match which {
        ReadCommand::Sip => (
            "System Integrity Protection",
            "System Integrity Protection status: enabled.",
            "System Integrity Protection status: disabled.",
        ),
        ReadCommand::AuthenticatedRoot => (
            "Authenticated root",
            "Authenticated Root status: enabled.",
            "Authenticated Root status: disabled.",
        ),
        _ => ("FileVault", "FileVault is On.", "FileVault is Off."),
    };
    let s = std::str::from_utf8(bytes).unwrap_or("").trim();
    let value = if s == on || s == on.trim_end_matches('.') {
        "Enabled"
    } else if s == off || s == off.trim_end_matches('.') {
        "Disabled"
    } else if which == ReadCommand::FileVault
        && (s.starts_with("Encryption in progress:") || s.starts_with("Decryption in progress:"))
    {
        "Transition in progress"
    } else {
        "Unrecognized status"
    };
    let mut e = DiagnosticEvidence::observed(label, value.into(), "", &which.source());
    e.qualification="Point-in-time reading only; does not establish startup security policy, all volumes, hardware integrity or a complete security posture.".into();
    if value == "Unrecognized status" {
        e.state = CapabilityState::Unknown;
        e.qualification="Unexpected, localized or custom configuration output. Review manually; raw output is not retained.".into();
    }
    e
}
/// Parse only complete, recognized list records. Unknown/localized/partial output never means current.
fn update_list(bytes: &[u8]) -> (String, Vec<FirmwareUpdate>) {
    let Ok(s) = std::str::from_utf8(bytes) else {
        return ("unknown".into(), vec![]);
    };
    if s.lines().any(|l| l.trim() == "No new software available.")
        && !s.contains("* Label:")
        && !s.to_ascii_lowercase().contains("error")
    {
        return ("noneOffered".into(), vec![]);
    }
    if s.to_ascii_lowercase().contains("error") {
        return ("unknown".into(), vec![]);
    }
    let mut updates = vec![];
    let mut pending = None;
    let mut malformed = false;
    for line in s.lines() {
        let l = line.trim();
        if let Some(label) = l.strip_prefix("* Label: ") {
            if pending.is_some() {
                malformed = true;
            }
            pending = text(label.as_bytes());
            if pending.is_none() {
                malformed = true;
            }
        } else if let Some(details) = l.strip_prefix("Title: ") {
            if let Some(label) = pending.take() {
                let Some((title, tail)) = details.split_once(", Version: ") else {
                    malformed = true;
                    continue;
                };
                let Some((version, _)) = tail.split_once(", Size: ") else {
                    malformed = true;
                    continue;
                };
                if let (Some(title), Some(version)) =
                    (text(title.as_bytes()), text(version.as_bytes()))
                {
                    updates.push(FirmwareUpdate{label,title,version,restart:details.split(',').any(|v|v.trim()=="Action: restart"),qualification:"Offered by macOS at the recorded time. Firmware inclusion/version and installation eligibility must be confirmed in Software Update; restart may be required even if not listed here.".into()});
                } else {
                    malformed = true;
                }
            } else {
                malformed = true;
            }
        }
        if updates.len() > 100 {
            return ("unknown".into(), vec![]);
        }
    }
    if pending.is_some() {
        malformed = true;
    }
    (
        if updates.is_empty() {
            "unknown"
        } else if malformed {
            "partial"
        } else {
            "available"
        }
        .into(),
        updates,
    )
}
fn save(c: &Connection, r: &FirmwareReport) -> Result<()> {
    records::save(
        c,
        "firmware",
        &r.id,
        &r.created_at,
        &r.status,
        "Firmware & security audit",
        r,
    )
}
fn checkpoint(
    c: &Connection,
    r: &FirmwareReport,
    emit: &mut impl FnMut(FirmwareReport) -> bool,
    cancel: &AtomicBool,
) -> Result<()> {
    save(c, r)?;
    if !emit(r.clone()) {
        cancel.store(true, Ordering::SeqCst);
    }
    Ok(())
}
pub fn audit(
    c: &Connection,
    request: FirmwareRequest,
    cancel: &AtomicBool,
    emit: impl FnMut(FirmwareReport) -> bool,
) -> Result<FirmwareReport> {
    audit_with(c, request, cancel, emit, run)
}
fn audit_with(
    c: &Connection,
    request: FirmwareRequest,
    cancel: &AtomicBool,
    mut emit: impl FnMut(FirmwareReport) -> bool,
    mut collect: impl FnMut(ReadCommand, &AtomicBool) -> std::result::Result<Vec<u8>, ReadFailure>,
) -> Result<FirmwareReport> {
    anyhow::ensure!(
        request.inventory || request.security || request.updates,
        "Select at least one optional audit."
    );
    anyhow::ensure!(
        !request.updates || request.network_acknowledged,
        "A macOS update check requires explicit network acknowledgement for this run."
    );
    let mut r=FirmwareReport{id:uuid::Uuid::new_v4().to_string(),created_at:now(),finished_at:None,status:"running".into(),stage:"Identify platform".into(),request,platform:compatibility(None,"Unavailable","Unavailable"),evidence:vec![],capabilities:vec![],findings:vec![],updates:vec![],update_state:"notRequested".into(),update_checked_at:None,errors:vec![],qualification:"Optional read operations selected for this run. Serial numbers, hardware UUIDs and raw command output are discarded. Update checks contact Apple through macOS and may refresh OS-owned catalogs/logs; cancelling Coqui stops its client, but cannot promise cancellation of the system service. No update is downloaded or installed by Coqui. Version comparison is not an integrity check; unavailable readings are never healthy readings.".into()};
    r.platform.qualification =
        "Platform identification is pending. Optional properties have not been read yet.".into();
    if r.request.updates {
        r.update_state = "pending".into();
    }
    checkpoint(c, &r, &mut emit, cancel)?;
    if !cancel.load(Ordering::SeqCst) {
        let h = match collect(ReadCommand::Hardware, cancel) {
            Ok(b) => {
                let h = hardware(&b);
                if h.is_none() {
                    r.errors
                        .push("Hardware report format was unrecognized.".into())
                }
                h
            }
            Err(e) => {
                r.errors
                    .push(format!("Platform identification: {}", e.detail()));
                None
            }
        };
        let mut read_text = |cmd| {
            collect(cmd, cancel)
                .ok()
                .and_then(|b| text(&b))
                .unwrap_or_else(|| "Unavailable".into())
        };
        let version = read_text(ReadCommand::Version);
        let build = read_text(ReadCommand::Build);
        r.platform = compatibility(h.as_ref(), &version, &build);
        r.capabilities = capabilities(&r.platform);
        if !r.platform.eligible {
            if r.request.updates {
                r.update_state = "withheld".into();
            }
            r.errors.push(r.platform.qualification.clone());
        }
        if r.platform.eligible && r.request.inventory && !cancel.load(Ordering::SeqCst) {
            r.stage = "Read firmware properties".into();
            for (label, keys) in [
                (
                    "System firmware",
                    &["boot_rom_version", "system_firmware_version"][..],
                ),
                ("OS loader", &["os_loader_version"][..]),
            ] {
                let value = h
                    .as_ref()
                    .and_then(|h| keys.iter().find_map(|k| field(h, k)));
                let mut e = if let Some(v) = value {
                    DiagnosticEvidence::observed(label, v, "", &ReadCommand::Hardware.source())
                } else {
                    DiagnosticEvidence::unavailable(label,"This model/report did not expose this property. No version or integrity verdict is inferred.")
                };
                e.source = ReadCommand::Hardware.source();
                if e.state == CapabilityState::Supported {
                    e.qualification="Reported version only. Coqui has no authoritative per-model latest-firmware mapping or attestation.".into();
                }
                r.evidence.push(e);
            }
            checkpoint(c, &r, &mut emit, cancel)?;
        }
        if r.platform.eligible && r.request.security {
            for (cmd, label) in [
                (ReadCommand::Sip, "System Integrity Protection"),
                (ReadCommand::AuthenticatedRoot, "Authenticated root"),
                (ReadCommand::FileVault, "FileVault"),
            ] {
                if cancel.load(Ordering::SeqCst) {
                    break;
                }
                r.stage = format!("Read {label}");
                checkpoint(c, &r, &mut emit, cancel)?;
                if cancel.load(Ordering::SeqCst) {
                    break;
                }
                let e = match collect(cmd, cancel) {
                    Ok(b) => security_reading(cmd, &b),
                    Err(e) => unavailable(label, cmd, e),
                };
                r.evidence.push(e);
            }
            let mut e=DiagnosticEvidence::unavailable("Startup security policy","Full/Reduced Security must be inspected for the selected startup disk in Recovery. Other security readings do not establish this policy.");
            e.state = CapabilityState::ManualOnly;
            e.source = POLICY_GUIDE.into();
            r.evidence.push(e);
        }
        if r.platform.eligible && r.request.updates && !cancel.load(Ordering::SeqCst) {
            r.stage = "Ask macOS for update availability (up to 120 seconds)".into();
            r.update_state = "checking".into();
            checkpoint(c, &r, &mut emit, cancel)?;
            if !cancel.load(Ordering::SeqCst) {
                match collect(ReadCommand::Updates, cancel) {
                    Ok(b) => {
                        (r.update_state, r.updates) = update_list(&b);
                        r.update_checked_at = Some(now());
                        if r.update_state == "unknown" || r.update_state == "partial" {
                            r.errors.push("macOS returned an incomplete or unrecognized update listing. Check Software Update manually.".into());
                        }
                    }
                    Err(e) => {
                        r.update_state = if e == ReadFailure::Permission {
                            "requiresPermission"
                        } else if e == ReadFailure::Cancelled {
                            "cancelled"
                        } else {
                            "unavailable"
                        }
                        .into();
                        r.errors.push(format!("Update check: {}", e.detail()));
                    }
                }
            }
        }
    }
    for e in &r.evidence {
        if e.state == CapabilityState::Supported
            && (e.value == "Disabled" || e.value == "Transition in progress")
        {
            r.findings.push(DiagnosticFinding{id:uuid::Uuid::new_v4().to_string(),title:format!("{}: {}",e.label,e.value),explanation:"Review the reason with the device owner or administrator. This observation does not establish compromise or authorize a settings change.".into(),severity:"attention".into(),confidence:"Reported point-in-time status".into(),kind:"firmwareSecurity".into(),evidence:vec![e.clone()]});
        }
    }
    if !r.updates.is_empty() {
        r.findings.push(DiagnosticFinding{id:uuid::Uuid::new_v4().to_string(),title:"macOS offers software updates".into(),explanation:format!("{} offering(s) at this check. This is not a standalone firmware update verdict; review compatibility, backup and restart requirements in Software Update.",r.updates.len()),severity:"info".into(),confidence:"macOS listing at the recorded time".into(),kind:"firmwareUpdate".into(),evidence:vec![DiagnosticEvidence::observed("Offered update count",r.updates.len().to_string(),"updates",&ReadCommand::Updates.source())]});
    }
    if cancel.load(Ordering::SeqCst) {
        r.status = "cancelled".into();
        if r.update_state == "checking" || r.update_state == "pending" {
            r.update_state = "cancelled".into();
        }
    } else if !r.errors.is_empty()
        || r.evidence.iter().any(|e| {
            !matches!(
                e.state,
                CapabilityState::Supported | CapabilityState::ManualOnly
            )
        })
    {
        r.status = "partial".into();
    } else {
        r.status = "complete".into();
    }
    r.stage = "Audit finished; inspect coverage and timestamps".into();
    r.finished_at = Some(now());
    checkpoint(c, &r, &mut emit, cancel)?;
    Ok(r)
}

fn save_plan(c: &Connection, p: &FirmwarePlan) -> Result<()> {
    records::save(
        c,
        "firmwarePlan",
        &p.id,
        &p.created_at,
        &p.status,
        &p.title,
        p,
    )
}
pub fn plan(c: &Connection, report_id: &str, kind: FirmwarePlanKind) -> Result<FirmwarePlan> {
    let r: FirmwareReport = records::get(c, "firmware", report_id)?;
    anyhow::ensure!(
        r.platform.eligible && matches!(r.status.as_str(), "complete" | "partial"),
        "A finished eligible platform audit is required. Retry the audit first."
    );
    let age = chrono::Utc::now()
        - chrono::DateTime::parse_from_rfc3339(&r.created_at)?.with_timezone(&chrono::Utc);
    anyhow::ensure!(
        age >= chrono::Duration::zero() && age < chrono::Duration::hours(24),
        "This audit is over 24 hours old; collect a fresh audit before planning."
    );
    let (title,source,steps,cautions)=match kind {
        FirmwarePlanKind::SoftwareUpdate=>("Review an Apple software update",UPDATE_GUIDE,vec!["Back up your Mac and confirm app/workload compatibility and organizational policy.","Open General → Software Update. macOS determines the compatible offerings and can check the network again.","Read the current Apple requirements; choose whether to install entirely in Apple's interface. Coqui cannot install or restart.","Keep power connected and allow Apple's installation and restarts to finish. Return to Coqui afterwards for new local readings."],vec!["Installation can interrupt work, restart repeatedly or fail. A prior listing may be stale or managed/deferred.","No exact firmware version target or rollback is promised; macOS may not offer a separate firmware package."]),
        FirmwarePlanKind::StartupPolicyReview=>("Inspect startup security in Recovery",POLICY_GUIDE,vec!["Save work and read Apple's instructions for this Mac. Arrange administrator/owner assistance for managed devices.","Use the Apple guide to enter Recovery manually and inspect the policy for the intended startup disk.","Record your observations outside Coqui if needed. This is an inspection plan; no lowering of security is recommended or authorized.","Restart normally using Apple's interface and return for readable local status observations."],vec!["Entering Recovery interrupts the current session. Coqui does not initiate shutdown or authenticate for you.","Full/Reduced Security cannot be verified by Coqui after return. SIP is a different property."]),
        FirmwarePlanKind::RecoveryReview=>("Review Apple firmware recovery requirements",RECOVERY_GUIDE,vec!["Use this guide only for the firmware/startup symptoms Apple describes; a version number alone is not a reason to revive or restore.","Consult the current Apple guide for the exact model, DFU port, compatible cable, power, host Mac and network requirements.","Apple distinguishes Revive from Restore. Review the effects and consult support before any operation.","Coqui provides this reading checklist only. Any recovery operation and consent occur separately in Apple's tools."],vec!["Restore erases the affected Mac. Revive is a different procedure; neither is executed or verified by Coqui.","Recovery can require another Mac and model-specific steps. Do not interrupt Apple's process; recovery or data restoration is not guaranteed."]),
    };
    let p=FirmwarePlan{id:uuid::Uuid::new_v4().to_string(),created_at:now(),report_id:r.id,kind,status:"awaitingAcknowledgement".into(),title:title.into(),before:r.platform,previous:r.evidence,steps:steps.into_iter().map(str::to_owned).collect(),cautions:cautions.into_iter().map(str::to_owned).collect(),source_url:source.into(),source_reviewed_at:SOURCE_REVIEWED.into(),acknowledged_at:None,outcome:"No action performed by Coqui. Guidance is a bundled snapshot; verify current instructions at Apple before proceeding.".into(),after_report_id:None,recovery:"If an update or startup fails, use current Apple support and recovery instructions. Restore can erase data. Coqui cannot promise rollback, initiate recovery, or establish a successful reboot/firmware operation from a user acknowledgement.".into()};
    save_plan(c, &p)?;
    Ok(p)
}
pub fn acknowledge(c: &Connection, id: &str, accepted: bool) -> Result<FirmwarePlan> {
    let mut p: FirmwarePlan = records::get(c, "firmwarePlan", id)?;
    anyhow::ensure!(
        p.status == "awaitingAcknowledgement",
        "This plan has already been acknowledged or declined."
    );
    let age = chrono::Utc::now()
        - chrono::DateTime::parse_from_rfc3339(&p.created_at)?.with_timezone(&chrono::Utc);
    anyhow::ensure!(
        !accepted || (age >= chrono::Duration::zero() && age < chrono::Duration::minutes(20)),
        "This review expired. Create a new plan from a fresh audit."
    );
    p.status = if accepted {
        "awaitingManualAction"
    } else {
        "declined"
    }
    .into();
    p.acknowledged_at = accepted.then(now);
    p.outcome=if accepted{"Guidance acknowledged; no operation or reboot initiated. Complete any manual action in Apple's tools, then request new observations."}else{"Plan declined; no operation performed."}.into();
    save_plan(c, &p)?;
    Ok(p)
}
pub fn observe(
    c: &Connection,
    id: &str,
    performed: bool,
    cancel: &AtomicBool,
    emit: impl FnMut(FirmwareReport) -> bool,
) -> Result<FirmwarePlan> {
    let mut p: FirmwarePlan = records::get(c, "firmwarePlan", id)?;
    anyhow::ensure!(
        p.status == "awaitingManualAction",
        "A separately acknowledged plan is required."
    );
    if !performed {
        p.status = "declined".into();
        p.outcome = "User recorded no manual action. No operation performed by Coqui.".into();
        save_plan(c, &p)?;
        return Ok(p);
    }
    let after = audit(
        c,
        FirmwareRequest {
            inventory: true,
            security: true,
            updates: false,
            network_acknowledged: false,
        },
        cancel,
        emit,
    )?;
    p.after_report_id = Some(after.id.clone());
    if after.status == "cancelled" {
        p.outcome =
            "New observations were cancelled; manual outcome remains unverified. Retry explicitly."
                .into();
        save_plan(c, &p)?;
        return Ok(p);
    }
    p.status = "unverified".into();
    let same = after.platform.eligible
        && after.platform.model == p.before.model
        && after.platform.chip == p.before.chip;
    p.outcome = if !same {
        "Platform identity could not be matched to the original audit; no before/after verification is claimed.".into()
    } else {
        format!("User reports completing a manual step. OS build before: {}; after: {}. New local readings are retained in audit {}. These observations do not verify a successful installation, reboot, restore or startup security policy. Firmware versions may be unchanged after a software update.",p.before.os_build,after.platform.os_build,after.id)
    };
    save_plan(c, &p)?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(inventory: bool, security: bool, updates: bool) -> FirmwareRequest {
        FirmwareRequest {
            inventory,
            security,
            updates,
            network_acknowledged: updates,
        }
    }
    fn fixture(cmd: ReadCommand, _: &AtomicBool) -> std::result::Result<Vec<u8>, ReadFailure> {
        Ok(match cmd {
            ReadCommand::Hardware=>br#"{"SPHardwareDataType":[{"machine_model":"Mac16,8","chip_type":"Apple M4 Pro","boot_rom_version":"18000.161.9","os_loader_version":"18000.161.9","serial_number":"DO_NOT_RETAIN","platform_UUID":"PRIVATE"}]}"#.to_vec(),
            ReadCommand::Version=>b"26.6.1\n".to_vec(),ReadCommand::Build=>b"25G123\n".to_vec(),
            ReadCommand::Sip=>b"System Integrity Protection status: enabled.\n".to_vec(),
            ReadCommand::AuthenticatedRoot=>b"Authenticated Root status: enabled.\n".to_vec(),
            ReadCommand::FileVault=>b"FileVault is Off.\n".to_vec(),
            ReadCommand::Updates=>b"Software Update Tool\nFinding available software\nSoftware Update found the following new or updated software:\n* Label: macOS Example-26.7\n\tTitle: macOS Example, Version: 26.7, Size: 12345KiB, Recommended: YES, Action: restart,\n".to_vec(),
        })
    }
    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE records(id TEXT PRIMARY KEY,kind TEXT,created_at TEXT,status TEXT,title TEXT,json TEXT);").unwrap();
        c
    }
    fn fixture_report(c: &Connection) -> FirmwareReport {
        audit_with(
            c,
            request(true, true, true),
            &AtomicBool::new(false),
            |_| true,
            fixture,
        )
        .unwrap()
    }
    #[test]
    fn no_optional_scope_or_network_consent_is_rejected_before_collection() {
        let c = db();
        let cancel = AtomicBool::new(false);
        assert!(audit_with(
            &c,
            request(false, false, false),
            &cancel,
            |_| true,
            |_, _| panic!("must not collect")
        )
        .is_err());
        let mut r = request(false, false, true);
        r.network_acknowledged = false;
        assert!(audit_with(
            &c,
            r,
            &cancel,
            |_| true,
            |_, _| panic!("must not contact network")
        )
        .is_err());
    }
    #[test]
    fn command_surface_is_fixed_and_read_only() {
        assert_eq!(
            ReadCommand::Updates.spec(),
            ("/usr/sbin/softwareupdate", &["--list"][..], 120)
        );
        assert_eq!(ReadCommand::Sip.spec().1, &["status"]);
        assert_eq!(
            ReadCommand::AuthenticatedRoot.spec().1,
            &["authenticated-root", "status"]
        );
        assert_eq!(ReadCommand::FileVault.spec().1, &["status"]);
        assert_eq!(
            ReadCommand::Hardware.spec().1,
            &["-json", "-detailLevel", "mini", "SPHardwareDataType"]
        );
    }
    #[test]
    fn exact_statuses_custom_output_and_missing_fields_remain_distinct() {
        assert_eq!(
            security_reading(
                ReadCommand::AuthenticatedRoot,
                b"Authenticated Root status: enabled\n"
            )
            .value,
            "Enabled"
        );
        assert_eq!(
            security_reading(
                ReadCommand::AuthenticatedRoot,
                b"Authenticated Root status: disabled\n"
            )
            .value,
            "Disabled"
        );
        assert_eq!(
            security_reading(
                ReadCommand::Sip,
                b"System Integrity Protection status: enabled."
            )
            .value,
            "Enabled"
        );
        assert_eq!(
            security_reading(
                ReadCommand::Sip,
                b"System Integrity Protection status: disabled."
            )
            .value,
            "Disabled"
        );
        for b in [
            b"System Integrity Protection status: enabled (Custom Configuration).".as_slice(),
            b"enabled",
            b"",
            b"Etat: active",
        ] {
            assert_eq!(
                security_reading(ReadCommand::Sip, b).state,
                CapabilityState::Unknown
            );
        }
        assert_eq!(
            security_reading(
                ReadCommand::FileVault,
                b"Encryption in progress: Percent completed = 40"
            )
            .value,
            "Transition in progress"
        );
        assert!(hardware(b"{\"SPHardwareDataType\":[]}").is_none());
        assert!(hardware(b"bad json").is_none());
        assert!(field(
            &serde_json::json!({"boot_rom_version":""}),
            "boot_rom_version"
        )
        .is_none());
    }
    #[test]
    fn update_listing_requires_complete_recognized_records() {
        let (state, u) =
            update_list(&fixture(ReadCommand::Updates, &AtomicBool::new(false)).unwrap());
        assert_eq!(state, "available");
        assert_eq!(u.len(), 1);
        assert!(u[0].restart);
        assert_eq!(u[0].version, "26.7");
        assert_eq!(
            update_list(b"Software Update Tool\nNo new software available.\n").0,
            "noneOffered"
        );
        for s in [
            "",
            "No updates",
            "Aucune mise a jour",
            "* Label: unfinished",
            "No new software available.\nError: network failed",
            "Title: x, Version: 1, Size: 1KiB",
        ] {
            assert_eq!(update_list(s.as_bytes()).0, "unknown");
        }
        let mut bytes = fixture(ReadCommand::Updates, &AtomicBool::new(false)).unwrap();
        bytes.extend(b"* Label: unfinished\n");
        assert_eq!(update_list(&bytes).0, "partial");
    }
    #[test]
    fn compatibility_does_not_claim_unknown_intel_or_older_os_support() {
        let h =
            hardware(&fixture(ReadCommand::Hardware, &AtomicBool::new(false)).unwrap()).unwrap();
        assert!(!compatibility(Some(&h), "13.6", "x").eligible);
        assert!(!compatibility(None, "26.6", "x").eligible);
        assert!(
            !compatibility(
                Some(&serde_json::json!({"machine_model":"iMac20,1","cpu_type":"Intel"})),
                "26.6",
                "x"
            )
            .eligible
        );
        assert!(!compatibility(Some(&h), "unknown", "x").eligible);
        let c = capabilities(&compatibility(Some(&h), "26.6", "x"));
        assert_eq!(c[3].state, CapabilityState::ManualOnly);
        assert_eq!(c[5].state, CapabilityState::Unsupported);
    }
    #[test]
    fn scoped_inventory_excludes_security_network_and_sensitive_raw_fields() {
        let c = db();
        let mut calls = vec![];
        let r = audit_with(
            &c,
            request(true, false, false),
            &AtomicBool::new(false),
            |_| true,
            |cmd, cancel| {
                calls.push(cmd);
                fixture(cmd, cancel)
            },
        )
        .unwrap();
        assert!(!calls.contains(&ReadCommand::Updates));
        assert!(!calls.contains(&ReadCommand::FileVault));
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains("DO_NOT_RETAIN"));
        assert!(!json.contains("PRIVATE"));
        assert!(!json.contains("serial_number"));
        assert_eq!(r.update_state, "notRequested");
    }
    #[test]
    fn audit_records_evidence_findings_and_true_stages() {
        let c = db();
        let mut stages = vec![];
        let r = audit_with(
            &c,
            request(true, true, true),
            &AtomicBool::new(false),
            |r| {
                stages.push(r.stage);
                true
            },
            fixture,
        )
        .unwrap();
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            assert_eq!(r.status, "complete");
            assert!(r.platform.eligible);
            assert!(r.findings.iter().any(|f| f.title == "FileVault: Disabled"));
            assert_eq!(r.update_state, "available");
            assert!(r.update_checked_at.is_some());
            assert!(stages.len() >= 6);
            assert_eq!(
                r.evidence
                    .iter()
                    .find(|e| e.label == "Startup security policy")
                    .unwrap()
                    .state,
                CapabilityState::ManualOnly
            );
        }
        let saved: FirmwareReport = records::get(&c, "firmware", &r.id).unwrap();
        assert_eq!(saved.status, r.status);
    }
    #[test]
    fn permission_timeout_and_missing_properties_preserve_partial_results() {
        let c = db();
        let r=audit_with(&c,request(true,true,true),&AtomicBool::new(false),|_|true,|cmd,cancel|match cmd {ReadCommand::FileVault=>Err(ReadFailure::Permission),ReadCommand::Updates=>Err(ReadFailure::Timeout),ReadCommand::Hardware=>Ok(br#"{"SPHardwareDataType":[{"machine_model":"Mac16,8","chip_type":"Apple M4 Pro"}]}"#.to_vec()),_=>fixture(cmd,cancel)}).unwrap();
        assert_eq!(r.status, "partial");
        if r.platform.eligible {
            assert_eq!(
                r.evidence
                    .iter()
                    .find(|e| e.label == "FileVault")
                    .unwrap()
                    .state,
                CapabilityState::RequiresPermission
            );
            assert_eq!(r.update_state, "unavailable");
            assert!(r.update_checked_at.is_none());
            assert_eq!(r.evidence[0].state, CapabilityState::Unknown);
        }
    }
    #[test]
    fn cancellation_and_stream_disconnect_do_not_start_more_utilities() {
        let c = db();
        let cancel = AtomicBool::new(false);
        let r = audit_with(
            &c,
            request(true, true, true),
            &cancel,
            |_| false,
            |_, _| panic!("cancelled before command"),
        )
        .unwrap();
        assert_eq!(r.status, "cancelled");
        let cancel = AtomicBool::new(false);
        let r = audit_with(
            &c,
            request(true, true, true),
            &cancel,
            |r| {
                if r.stage.starts_with("Read System Integrity") {
                    cancel.store(true, Ordering::SeqCst);
                }
                true
            },
            fixture,
        )
        .unwrap();
        if r.platform.eligible {
            assert_eq!(r.status, "cancelled");
            assert!(r.evidence.iter().any(|e| e.label == "System firmware"));
            assert!(!r.evidence.iter().any(|e| e.label == "FileVault"));
            assert_eq!(r.update_state, "cancelled");
        }
    }
    #[test]
    fn plans_are_separate_expiring_and_never_authorize_mutation() {
        let c = db();
        let mut r = fixture_report(&c);
        // Fixture compatibility independent of host target; production computes it from the actual run.
        r.platform.eligible = true;
        r.status = "complete".into();
        save(&c, &r).unwrap();
        for kind in [
            FirmwarePlanKind::SoftwareUpdate,
            FirmwarePlanKind::StartupPolicyReview,
            FirmwarePlanKind::RecoveryReview,
        ] {
            let p = plan(&c, &r.id, kind).unwrap();
            assert_eq!(p.status, "awaitingAcknowledgement");
            assert!(observe(&c, &p.id, true, &AtomicBool::new(true), |_| true).is_err());
            let p = acknowledge(&c, &p.id, true).unwrap();
            assert_eq!(p.status, "awaitingManualAction");
            assert!(acknowledge(&c, &p.id, true).is_err());
            let p = observe(&c, &p.id, false, &AtomicBool::new(false), |_| {
                panic!("no action means no collector")
            })
            .unwrap();
            assert_eq!(p.status, "declined");
            assert!(p.after_report_id.is_none());
        }
        let mut p = plan(&c, &r.id, FirmwarePlanKind::RecoveryReview).unwrap();
        assert!(p.cautions.iter().any(|s| s.contains("erases")));
        p.created_at = (chrono::Utc::now() - chrono::Duration::minutes(21)).to_rfc3339();
        save_plan(&c, &p).unwrap();
        assert!(acknowledge(&c, &p.id, true).is_err());
        assert_eq!(acknowledge(&c, &p.id, false).unwrap().status, "declined");
        r.created_at = (chrono::Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
        save(&c, &r).unwrap();
        assert!(plan(&c, &r.id, FirmwarePlanKind::SoftwareUpdate).is_err());
    }
    #[test]
    fn audit_and_guidance_survive_reopen_and_follow_retention_and_purge() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let c = crate::database::open(&path).unwrap();
        let mut r = fixture_report(&c);
        r.platform.eligible = true;
        r.status = "complete".into();
        save(&c, &r).unwrap();
        let p = plan(&c, &r.id, FirmwarePlanKind::SoftwareUpdate).unwrap();
        drop(c);
        let c = crate::database::open(&path).unwrap();
        let saved: FirmwarePlan = records::get(&c, "firmwarePlan", &p.id).unwrap();
        assert_eq!(saved.report_id, r.id);
        c.execute("UPDATE records SET status='running',json=json_set(json,'$.status','running') WHERE id=?1",[&r.id]).unwrap();
        crate::database::recover_interrupted(&c).unwrap();
        assert_eq!(
            records::get::<FirmwareReport>(&c, "firmware", &r.id)
                .unwrap()
                .status,
            "interrupted"
        );
        let old = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
        c.execute("UPDATE records SET created_at=?1", [old])
            .unwrap();
        crate::database::retain(&c, 30).unwrap();
        assert!(records::list(&c, None, 0).unwrap().is_empty());
        fixture_report(&c);
        crate::database::purge(&c).unwrap();
        assert!(records::list(&c, None, 0).unwrap().is_empty());
    }
}
