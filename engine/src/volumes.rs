//! Mounted-volume planning and protected OS firmlink aliases. Does not mount anything.
use crate::{model::*, platform};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};
pub fn friendly(path: &Path) -> PathBuf {
    // These aliases are published by macOS in a root-owned system file, not guessed from user input.
    static ALIASES: OnceLock<Vec<(PathBuf, PathBuf)>> = OnceLock::new();
    let aliases = ALIASES.get_or_init(|| {
        std::fs::read_to_string("/usr/share/firmlinks")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let (visible, data) = line.split_once('\t')?;
                (visible.starts_with('/') && !visible.contains("..") && !data.contains(".."))
                    .then(|| (PathBuf::from(visible), PathBuf::from(data)))
            })
            .collect()
    });
    alias_path(path, aliases)
}
fn alias_path(path: &Path, aliases: &[(PathBuf, PathBuf)]) -> PathBuf {
    let Ok(relative) = path.strip_prefix("/System/Volumes/Data") else {
        return path.to_owned();
    };
    for (visible, data) in aliases {
        if let Ok(tail) = relative.strip_prefix(data) {
            return visible.join(tail);
        }
    }
    path.to_owned()
}
pub fn same_or_below(path: &Path, root: &Path) -> bool {
    friendly(path).starts_with(friendly(root))
}
pub fn discover() -> Result<Vec<DiskVolume>> {
    let mut roots = vec![];
    #[cfg(target_os = "macos")]
    unsafe {
        let count = libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT);
        anyhow::ensure!(count > 0, "Mounted filesystem inventory unavailable");
        let mut mounts: Vec<libc::statfs> = vec![std::mem::zeroed(); count as usize + 16];
        let count = libc::getfsstat(
            mounts.as_mut_ptr(),
            (mounts.len() * std::mem::size_of::<libc::statfs>()) as i32,
            libc::MNT_NOWAIT,
        );
        anyhow::ensure!(
            count >= 0 && count as usize <= mounts.len(),
            "Mounted filesystem inventory changed; retry"
        );
        for m in mounts.iter().take(count as usize) {
            let fs = std::ffi::CStr::from_ptr(m.f_fstypename.as_ptr())
                .to_string_lossy()
                .into_owned();
            if ["devfs", "autofs", "fdesc", "volfs"].contains(&fs.as_str()) {
                continue;
            }
            let mount = std::ffi::CStr::from_ptr(m.f_mntonname.as_ptr())
                .to_string_lossy()
                .into_owned();
            let device = std::ffi::CStr::from_ptr(m.f_mntfromname.as_ptr())
                .to_string_lossy()
                .into_owned();
            let local = m.f_flags & libc::MNT_LOCAL as u32 != 0;
            let info = if local && device.starts_with("/dev/") {
                platform::command(
                    "/usr/sbin/diskutil",
                    &["info", "-plist", &mount],
                    Duration::from_secs(4),
                )
                .ok()
                .and_then(|v| plist::Value::from_reader(std::io::Cursor::new(v)).ok())
            } else {
                None
            };
            let dict = info.as_ref().and_then(plist::Value::as_dictionary);
            let internal = dict
                .and_then(|d| d.get("Internal"))
                .and_then(plist::Value::as_boolean)
                == Some(true);
            let id = dict
                .and_then(|d| d.get("VolumeUUID"))
                .and_then(plist::Value::as_string)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{device}:{mount}"));
            roots.push(DiskVolume{id,mount,device,filesystem:fs,internal,read_only:m.f_flags & libc::MNT_RDONLY as u32 !=0,selected:internal,qualification:if internal{"Mounted internal filesystem; access remains subject to macOS permissions."}else{"External/network or internal identity unconfirmed; separate selection required."}.into()});
        }
    }
    roots.sort_by(|a, b| a.mount.cmp(&b.mount));
    Ok(roots)
}
pub fn plan(c: &rusqlite::Connection) -> Result<DiskScanPlan> {
    let p=DiskScanPlan{id:uuid::Uuid::new_v4().to_string(),created_at:now(),volumes:discover()?,qualification:"Only mounted internal filesystems are selected. No volume or snapshot is mounted automatically. Full Disk Access and administrator authorization do not override all protections. APFS volumes share capacity; indexed allocation is not exact recoverable space.".into()};
    c.execute(
        "INSERT INTO scan_plans VALUES(?1,?2,?3)",
        rusqlite::params![p.id, p.created_at, serde_json::to_string(&p)?],
    )?;
    Ok(p)
}
pub fn get(c: &rusqlite::Connection, id: &str) -> Result<DiskScanPlan> {
    let json: String = c.query_row("SELECT json FROM scan_plans WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    Ok(serde_json::from_str(&json)?)
}
pub fn approved_roots(c: &rusqlite::Connection, request: &ScanRequest) -> Result<Vec<PathBuf>> {
    let id = request.plan_id.as_deref().ok_or_else(|| {
        anyhow::anyhow!("Review the mounted-volume scope before a whole-disk scan.")
    })?;
    let json: String = c.query_row("SELECT json FROM scan_plans WHERE id=?1", [id], |r| {
        r.get(0)
    })?;
    let p: DiskScanPlan = serde_json::from_str(&json)?;
    let age = chrono::Utc::now()
        - chrono::DateTime::parse_from_rfc3339(&p.created_at)?.with_timezone(&chrono::Utc);
    anyhow::ensure!(
        age >= chrono::Duration::zero() && age <= chrono::Duration::minutes(10),
        "Volume scope expired; review it again"
    );
    let current = discover()?;
    validate_scope(&p, &current, &request.roots)
}
fn validate_scope(
    p: &DiskScanPlan,
    current: &[DiskVolume],
    roots: &[String],
) -> Result<Vec<PathBuf>> {
    let mut result = vec![];
    for mount in roots {
        let old = p
            .volumes
            .iter()
            .find(|v| v.internal && &v.mount == mount)
            .ok_or_else(|| {
                anyhow::anyhow!("Location was not in the reviewed internal-volume scope")
            })?;
        anyhow::ensure!(
            current.iter().any(|v| v.internal
                && v.id == old.id
                && v.mount == old.mount
                && v.device == old.device
                && v.filesystem == old.filesystem),
            "A reviewed volume changed or disconnected; review scope again"
        );
        result.push(PathBuf::from(mount));
    }
    anyhow::ensure!(!result.is_empty(), "Select at least one internal volume");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volume_scope_rejects_disconnects_replacements_and_external_locations() {
        let volume = DiskVolume {
            id: "one".into(),
            mount: "/fixture".into(),
            device: "disk1".into(),
            filesystem: "apfs".into(),
            internal: true,
            read_only: false,
            selected: true,
            qualification: String::new(),
        };
        let p = DiskScanPlan {
            id: "scope".into(),
            created_at: now(),
            volumes: vec![volume.clone()],
            qualification: String::new(),
        };
        assert!(validate_scope(&p, std::slice::from_ref(&volume), &["/fixture".into()]).is_ok());
        assert!(validate_scope(&p, &[], &["/fixture".into()]).is_err());
        let mut changed = volume.clone();
        changed.id = "other".into();
        assert!(validate_scope(&p, &[changed], &["/fixture".into()]).is_err());
        assert!(validate_scope(&p, &[volume], &["/Volumes/External".into()]).is_err());
        assert!(validate_scope(&p, &[], &[]).is_err());
    }
    #[test]
    fn aliases_use_component_boundaries_and_preserve_other_volumes() {
        let aliases = vec![
            ("/Users".into(), "Users".into()),
            ("/Applications".into(), "Applications".into()),
        ];
        assert_eq!(
            alias_path(
                Path::new("/System/Volumes/Data/Users/me/Documents"),
                &aliases
            ),
            PathBuf::from("/Users/me/Documents")
        );
        assert_eq!(
            alias_path(Path::new("/System/Volumes/Data/UsersExtra/file"), &aliases),
            PathBuf::from("/System/Volumes/Data/UsersExtra/file")
        );
        assert_eq!(
            alias_path(Path::new("/Volumes/Other/Users/me"), &aliases),
            PathBuf::from("/Volumes/Other/Users/me")
        );
    }
}

/// APFS volume-group aliases can share st_dev. statfs identifies the mounted filesystem.
pub fn mounted_identity(path: &Path) -> Result<(String, String)> {
    #[cfg(target_os = "macos")]
    unsafe {
        let p = std::ffi::CString::new(
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("Unrepresentable path"))?,
        )?;
        let mut stat: libc::statfs = std::mem::zeroed();
        anyhow::ensure!(
            libc::statfs(p.as_ptr(), &mut stat) == 0,
            "Filesystem identity unavailable"
        );
        Ok((
            std::ffi::CStr::from_ptr(stat.f_mntfromname.as_ptr())
                .to_string_lossy()
                .into_owned(),
            std::ffi::CStr::from_ptr(stat.f_mntonname.as_ptr())
                .to_string_lossy()
                .into_owned(),
        ))
    }
    #[cfg(not(target_os = "macos"))]
    {
        use std::os::unix::fs::MetadataExt;
        Ok((std::fs::metadata(path)?.dev().to_string(), String::new()))
    }
}
