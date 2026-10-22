//! Explicit bounded content analysis. Indexed sizes only select candidates; they never prove equality.
use crate::{cleanup::Policy, database, model::*, records, storage};
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_TOTAL: u64 = 2 * 1024 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Stamp {
    dev: u64,
    ino: u64,
    size: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
}
impl Stamp {
    fn of(m: &fs::Metadata) -> Self {
        Self {
            dev: m.dev(),
            ino: m.ino(),
            size: m.len(),
            modified: m.mtime(),
            modified_ns: m.mtime_nsec(),
            changed: m.ctime(),
            changed_ns: m.ctime_nsec(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CopySeal {
    pub keeper: String,
    pub digest: String,
    stamp: Stamp,
    keeper_stamp: Stamp,
}
fn open(path: &Path, policy: &Policy) -> Result<(File, Stamp)> {
    anyhow::ensure!(
        path.is_absolute() && !policy.exclusions.iter().any(|p| path.starts_with(p)),
        "Excluded or non-absolute path"
    );
    for a in path.ancestors() {
        let m = fs::symlink_metadata(a)?;
        anyhow::ensure!(
            !m.file_type().is_symlink() && !storage::placeholder(a, &m),
            "Linked or cloud-only location"
        );
    }
    anyhow::ensure!(
        !path.components().any(|c| matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )),
        "Path traversal is unsupported"
    );
    anyhow::ensure!(
        !path.components().any(|c| [".git", ".hg", ".svn", ".jj"]
            .iter()
            .any(|n| c.as_os_str() == *n)),
        "Version-control metadata is excluded from content comparison"
    );
    let m = fs::symlink_metadata(path)?;
    anyhow::ensure!(
        m.is_file()
            && m.nlink() == 1
            && m.uid() == unsafe { libc::geteuid() }
            && m.len() <= MAX_FILE,
        "Only user-owned, non-hardlinked regular files up to 256 MiB are supported"
    );
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let stamp = Stamp::of(&m);
    anyhow::ensure!(
        Stamp::of(&file.metadata()?) == stamp,
        "File changed while opening"
    );
    Ok((file, stamp))
}
fn hash(file: &mut File, cancel: &AtomicBool) -> Result<String> {
    let mut h = Sha256::new();
    let mut b = [0; 65536];
    loop {
        anyhow::ensure!(!cancel.load(Ordering::SeqCst), "Cancelled");
        let n = file.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn digest(path: &Path, policy: &Policy, cancel: &AtomicBool) -> Result<(String, Stamp)> {
    let (mut f, s) = open(path, policy)?;
    let h = hash(&mut f, cancel)?;
    anyhow::ensure!(
        Stamp::of(&f.metadata()?) == s && Stamp::of(&fs::symlink_metadata(path)?) == s,
        "File changed during hashing"
    );
    Ok((h, s))
}
pub(crate) fn confirm(
    path: &Path,
    keeper: &Path,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<CopySeal> {
    anyhow::ensure!(path != keeper, "A keeper cannot be removed");
    let (digest, stamp) = digest(path, policy, cancel)?;
    let (kdigest, keeper_stamp) = self::digest(keeper, policy, cancel)?;
    anyhow::ensure!(
        stamp.dev != keeper_stamp.dev || stamp.ino != keeper_stamp.ino,
        "Hardlinked copies are protected"
    );
    anyhow::ensure!(digest == kdigest, "Contents differ");
    let (mut a, sa) = open(path, policy)?;
    let (mut b, sb) = open(keeper, policy)?;
    let mut x = [0; 65536];
    let mut y = [0; 65536];
    loop {
        anyhow::ensure!(!cancel.load(Ordering::SeqCst), "Cancelled");
        let n = a.read(&mut x)?;
        b.read_exact(&mut y[..n])?;
        anyhow::ensure!(x[..n] == y[..n], "Byte comparison differs");
        if n == 0 {
            let mut tail = [0];
            anyhow::ensure!(b.read(&mut tail)? == 0, "Byte comparison length differs");
            break;
        }
    }
    anyhow::ensure!(
        sa == stamp
            && sb == keeper_stamp
            && Stamp::of(&a.metadata()?) == stamp
            && Stamp::of(&b.metadata()?) == keeper_stamp,
        "Copy changed during verification"
    );
    Ok(CopySeal {
        keeper: keeper
            .to_str()
            .context("Unrepresentable keeper path")?
            .into(),
        digest,
        stamp,
        keeper_stamp,
    })
}
pub(crate) fn revalidate(
    path: &Path,
    seal: &CopySeal,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<()> {
    anyhow::ensure!(
        confirm(path, Path::new(&seal.keeper), policy, cancel)? == *seal,
        "Copy or keeper changed since review"
    );
    Ok(())
}
fn photo_hash(path: &Path, policy: &Policy) -> Result<u64> {
    let (mut f, s) = open(path, policy)?;
    anyhow::ensure!(
        s.size <= 32 * 1024 * 1024,
        "Photo exceeds 32 MiB analysis bound"
    );
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    anyhow::ensure!(Stamp::of(&f.metadata()?) == s, "Photo changed");
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12000);
    limits.max_image_height = Some(12000);
    limits.max_alloc = Some(96 * 1024 * 1024);
    reader.limits(limits);
    let img = reader
        .decode()?
        .resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();
    let mut value = 0;
    for y in 0..8 {
        for x in 0..8 {
            value = (value << 1) | u64::from(img.get_pixel(x, y)[0] > img.get_pixel(x + 1, y)[0]);
        }
    }
    Ok(value)
}
pub fn get(c: &Connection, id: &str) -> Result<DuplicateReport> {
    records::get(c, "duplicates", id)
}
fn save(c: &Connection, r: &DuplicateReport) -> Result<()> {
    records::save(
        c,
        "duplicates",
        &r.id,
        &r.created_at,
        &r.status,
        "Duplicate content analysis",
        r,
    )
}
pub fn analyze(
    c: &Connection,
    session_id: &str,
    policy: &Policy,
    cancel: &AtomicBool,
    mut emit: impl FnMut(DuplicateReport) -> bool,
) -> Result<DuplicateReport> {
    let session = database::session(c, session_id)?;
    anyhow::ensure!(session.status != "running", "Wait for the storage scan");
    let mut r=DuplicateReport{id:uuid::Uuid::new_v4().to_string(),session_id:session_id.into(),created_at:now(),status:"running".into(),stage:"Reading indexed candidates".into(),examined:0,total:0,hashed_bytes:0.,groups:vec![],photo_matches:vec![],gaps:vec![],qualification:"Explicit content reads. At most 2,000 indexed files, 256 MiB per file, 2 GiB of initial hashing and 120 seconds. PNG/JPEG visual candidates: at most 150 images up to 32 MiB and 12,000px; dHash distance ≤6 is only a similarity hint, never duplicate authority. Links/cloud placeholders/exclusions skipped. Intentional backups and project copies may be valuable. APFS clones/shared blocks mean sizes are not reclaimable-space promises.".into()};
    save(c, &r)?;
    let mut q=c.prepare("SELECT path,logical,allocated,modified FROM files WHERE session_id=?1 AND directory=0 AND logical>0 ORDER BY logical DESC,path LIMIT 2001")?;
    let mut files = q
        .query_map([session_id], |row| {
            Ok(DuplicateFile {
                review_supported: false,
                review_qualification: String::new(),
                path: row.get(0)?,
                logical_bytes: row.get(1)?,
                allocated_bytes: row.get(2)?,
                modified_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if files.is_empty() && session.file_count > 0 {
        r.gaps.push("This saved scan no longer has a retained file index; rescan the scope before comparing contents.".into());
        r.status = "partial".into();
    }
    if files.len() > 2000 {
        files.truncate(2000);
        r.gaps.push(
            "More than 2,000 files; this report covers the largest indexed candidates only.".into(),
        );
    }
    r.total = files.len() as u32;
    let mut sizes = BTreeMap::<u64, usize>::new();
    for f in &files {
        *sizes.entry(f.logical_bytes as u64).or_default() += 1;
    }
    let start = Instant::now();
    let mut hashes = BTreeMap::<(u64, String), Vec<DuplicateFile>>::new();
    let mut photos = Vec::<(String, String, u64)>::new();
    let mut seen = HashSet::new();
    let mut errors = 0;
    for mut f in files {
        match crate::cleanup::duplicate_scope(Path::new(&f.path), policy) {
            Ok(()) => {
                f.review_supported = true;
                f.review_qualification =
                    "Personal-folder copy; a fresh content/keeper review is still required.".into()
            }
            Err(e) => {
                f.review_supported = false;
                f.review_qualification = format!("Analysis only: {e}");
            }
        }
        if cancel.load(Ordering::SeqCst) {
            r.status = "cancelled".into();
            break;
        }
        if start.elapsed() > Duration::from_secs(120) || r.hashed_bytes >= MAX_TOTAL as f64 {
            r.gaps.push(
                "Content-analysis time/byte bound reached; remaining files were not read.".into(),
            );
            r.status = "partial".into();
            break;
        }
        r.examined += 1;
        r.stage = "Comparing file contents".into();
        let result = (|| -> Result<()> {
            let p = Path::new(&f.path);
            let (opened, s) = open(p, policy)?;
            anyhow::ensure!(
                s.size == f.logical_bytes as u64,
                "File size changed since the metadata scan; rescan this scope"
            );
            let metadata = opened.metadata()?;
            f.allocated_bytes = metadata.blocks() as f64 * 512.;
            f.modified_at = metadata
                .modified()
                .ok()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_default();
            if !seen.insert((s.dev, s.ino)) {
                return Ok(());
            }
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let is_photo = ["png", "jpg", "jpeg"].contains(&ext.as_str()) && photos.len() < 150;
            if sizes.get(&(f.logical_bytes as u64)).copied().unwrap_or(0) > 1 {
                anyhow::ensure!(
                    r.hashed_bytes + s.size as f64 <= MAX_TOTAL as f64,
                    "Hashing byte bound"
                );
                let (h, hashed_stamp) = digest(p, policy, cancel)?;
                anyhow::ensure!(
                    hashed_stamp == s,
                    "File changed between inspection and hashing"
                );
                r.hashed_bytes += s.size as f64;
                hashes.entry((s.size, h)).or_default().push(f.clone());
            }
            if is_photo {
                let h = photo_hash(p, policy)?;
                photos.push((
                    f.path.clone(),
                    p.file_stem().and_then(|s| s.to_str()).unwrap_or("").into(),
                    h,
                ));
            }
            Ok(())
        })();
        if let Err(e) = result {
            errors += 1;
            if r.gaps.len() < 20 {
                r.gaps.push(format!("{}: {e:#}", f.path));
            }
        }
        if r.examined % 10 == 0 {
            save(c, &r)?;
        }
        if r.examined % 10 == 0 && !emit(r.clone()) {
            cancel.store(true, Ordering::SeqCst);
        }
    }
    let mut verification_bytes = 0u64;
    for ((_, h), candidates) in hashes {
        if candidates.len() < 2 {
            continue;
        }
        let mut confirmed = vec![candidates[0].clone()];
        for f in candidates.iter().skip(1) {
            let cost = (f.logical_bytes as u64).saturating_mul(4);
            if verification_bytes.saturating_add(cost) > MAX_TOTAL
                || start.elapsed() > Duration::from_secs(120)
            {
                r.gaps.push(
                    "Byte-verification bound reached; remaining potential groups are unconfirmed."
                        .into(),
                );
                r.status = "partial".into();
                break;
            }
            verification_bytes += cost;
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            match confirm(
                Path::new(&f.path),
                Path::new(&candidates[0].path),
                policy,
                cancel,
            ) {
                Ok(seal) if seal.digest == h => confirmed.push(f.clone()),
                _ => {
                    errors += 1;
                    if r.gaps.len() < 20 {
                        r.gaps.push(format!("{}: copy changed between hashing and byte verification; rescan and compare again",f.path));
                    }
                }
            }
        }
        if confirmed.len() > 1 {
            r.groups.push(DuplicateGroup {
                id: uuid::Uuid::new_v4().to_string(),
                digest: h,
                files: confirmed,
            });
        }
    }
    'pairs: for i in 0..photos.len() {
        for j in i + 1..photos.len() {
            if r.photo_matches.len() >= 200 {
                break 'pairs;
            }
            let d = (photos[i].2 ^ photos[j].2).count_ones();
            if d <= 6
                && !r.groups.iter().any(|g| {
                    g.files.iter().any(|f| f.path == photos[i].0)
                        && g.files.iter().any(|f| f.path == photos[j].0)
                })
            {
                r.photo_matches.push(PhotoMatch{left:photos[i].0.clone(),right:photos[j].0.clone(),distance:d,classification:if photos[i].1==photos[j].1{"Possible modified copy"}else{"Visually similar candidate"}.into(),qualification:"Perceptual grayscale hash; collisions and false matches are possible. Inspect originals in Finder. No cleanup is authorized for this pair.".into()});
            }
        }
    }
    if cancel.load(Ordering::SeqCst) {
        r.status = "cancelled".into()
    } else if r.status == "running" {
        r.status = if errors > 0 { "partial" } else { "complete" }.into()
    }
    r.groups.sort_by(|a, b| {
        b.files[0]
            .logical_bytes
            .total_cmp(&a.files[0].logical_bytes)
    });
    r.stage = "Content analysis finished".into();
    if errors > 20 {
        r.gaps.push(format!(
            "{errors} files/copies could not be fully verified."
        ));
    }
    save(c, &r)?;
    emit(r.clone());
    Ok(r)
}

pub(crate) fn verify_moved(destination: &Path, seal: &CopySeal, policy: &Policy) -> Result<()> {
    let cancel = AtomicBool::new(false);
    let (h, _) = digest(destination, policy, &cancel)?;
    anyhow::ensure!(h == seal.digest, "Trash copy contents differ");
    let (h, stamp) = digest(Path::new(&seal.keeper), policy, &cancel)?;
    anyhow::ensure!(
        h == seal.digest && stamp == seal.keeper_stamp,
        "Keeper changed or disappeared during the move"
    );
    Ok(())
}
