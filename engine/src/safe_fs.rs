//! Descriptor-relative metadata and exclusive moves for the administrator executor.
//! No file contents are read and directory/bundle links are never traversed.
use anyhow::{Context, Result};
use std::{
    ffi::{CStr, CString},
    fs::File,
    os::fd::{AsRawFd, FromRawFd},
    path::{Component, Path, PathBuf},
};

pub(crate) fn directory(path: &Path) -> Result<File> {
    anyhow::ensure!(path.is_absolute(), "An absolute directory is required");
    let mut file = File::open("/")?;
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                use std::os::unix::ffi::OsStrExt;
                let name = CString::new(name.as_bytes())?;
                let fd = unsafe {
                    libc::openat(
                        file.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    )
                };
                anyhow::ensure!(
                    fd >= 0,
                    "Directory changed, linked or inaccessible: {}",
                    std::io::Error::last_os_error()
                );
                file = unsafe { File::from_raw_fd(fd) };
            }
            _ => anyhow::bail!("Relative path components are not permitted"),
        }
    }
    Ok(file)
}
pub(crate) fn parent(path: &Path) -> Result<(File, CString)> {
    use std::os::unix::ffi::OsStrExt;
    Ok((
        directory(path.parent().context("Filesystem roots are protected")?)?,
        CString::new(path.file_name().context("Missing name")?.as_bytes())?,
    ))
}
pub(crate) fn stat(parent: &File, name: &CStr) -> Result<libc::stat> {
    let mut value = unsafe { std::mem::zeroed() };
    anyhow::ensure!(
        unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                &mut value,
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0,
        "Metadata changed or unavailable: {}",
        std::io::Error::last_os_error()
    );
    Ok(value)
}
pub(crate) fn names(dir: &File, mut emit: impl FnMut(CString) -> Result<()>) -> Result<()> {
    let fd = unsafe { libc::dup(dir.as_raw_fd()) };
    anyhow::ensure!(fd >= 0, "Directory descriptor unavailable");
    let raw = unsafe { libc::fdopendir(fd) };
    if raw.is_null() {
        unsafe {
            libc::close(fd);
        }
        anyhow::bail!("Directory enumeration unavailable");
    }
    struct Guard(*mut libc::DIR);
    impl Drop for Guard {
        fn drop(&mut self) {
            unsafe {
                libc::closedir(self.0);
            }
        }
    }
    let guard = Guard(raw);
    loop {
        unsafe {
            *libc::__error() = 0;
        }
        let next = unsafe { libc::readdir(guard.0) };
        if next.is_null() {
            anyhow::ensure!(
                std::io::Error::last_os_error().raw_os_error() == Some(0),
                "Directory enumeration changed or failed"
            );
            break;
        }
        let name = unsafe { CStr::from_ptr((*next).d_name.as_ptr()) };
        if name.to_bytes() != b"." && name.to_bytes() != b".." {
            emit(name.to_owned())?;
        }
    }
    Ok(())
}
pub(crate) fn open_child(dir: &File, name: &CStr, expected: &libc::stat) -> Result<File> {
    use std::os::unix::fs::MetadataExt;
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    anyhow::ensure!(fd >= 0, "Directory changed, linked or inaccessible");
    let file = unsafe { File::from_raw_fd(fd) };
    let m = file.metadata()?;
    anyhow::ensure!(
        m.dev() == expected.st_dev as u64 && m.ino() == expected.st_ino,
        "Directory identity changed"
    );
    Ok(file)
}
pub(crate) fn exclusive_move(
    source: &Path,
    destination: &Path,
    expected: (u64, u64, u32),
    parent_id: Option<(u64, u64)>,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let (from, name) = parent(source)?;
    let (to, target) = parent(destination)?;
    let m = stat(&from, &name)?;
    anyhow::ensure!(
        (m.st_dev as u64, m.st_ino, m.st_uid) == expected,
        "Source identity changed immediately before move"
    );
    if let Some(id) = parent_id {
        let m = to.metadata()?;
        anyhow::ensure!(
            (m.dev(), m.ino()) == id,
            "Original destination parent changed"
        );
    }
    anyhow::ensure!(
        unsafe {
            libc::renameatx_np(
                from.as_raw_fd(),
                name.as_ptr(),
                to.as_raw_fd(),
                target.as_ptr(),
                libc::RENAME_EXCL,
            )
        } == 0,
        "Exclusive move failed: {}",
        std::io::Error::last_os_error()
    );
    Ok(())
}
pub(crate) fn read_bundle_info(path: &Path) -> Result<Vec<u8>> {
    use std::os::darwin::fs::MetadataExt as DarwinMetadataExt;
    use std::{io::Read, os::unix::fs::MetadataExt};
    let bundle = directory(path)?;
    let name = CString::new("Contents")?;
    let contents = stat(&bundle, &name)?;
    let dir = open_child(&bundle, &name, &contents)?;
    let name = CString::new("Info.plist")?;
    let before = stat(&dir, &name)?;
    anyhow::ensure!(
        before.st_mode & libc::S_IFMT == libc::S_IFREG
            && before.st_nlink == 1
            && before.st_size <= 1024 * 1024
            && before.st_flags & 0x40000000 == 0,
        "Bundle metadata linked, oversized or cloud-only"
    );
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    anyhow::ensure!(fd >= 0, "Bundle metadata unavailable");
    let mut file = unsafe { File::from_raw_fd(fd) };
    let m = file.metadata()?;
    anyhow::ensure!(
        m.dev() == before.st_dev as u64
            && m.ino() == before.st_ino
            && m.st_flags() & 0x40000000 == 0,
        "Bundle metadata changed"
    );
    let mut bytes = Vec::new();
    (&mut file).take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    anyhow::ensure!(
        bytes.len() <= 1024 * 1024
            && m.len() == after.len()
            && m.ctime() == after.ctime()
            && m.ctime_nsec() == after.ctime_nsec()
            && m.mtime() == after.mtime()
            && m.mtime_nsec() == after.mtime_nsec(),
        "Bundle metadata changed while reading"
    );
    Ok(bytes)
}
pub(crate) fn path_of(relative: &str, base: &Path) -> PathBuf {
    if relative.is_empty() {
        base.into()
    } else {
        base.join(relative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{symlink, MetadataExt},
    };
    #[test]
    fn linked_parents_and_changed_source_identities_cannot_redirect_moves() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("folder");
        fs::create_dir(&source).unwrap();
        let file = source.join("item");
        fs::write(&file, b"original fixture").unwrap();
        let m = fs::symlink_metadata(&file).unwrap();
        fs::rename(&file, source.join("old")).unwrap();
        fs::write(&file, b"different fixture").unwrap();
        let dest = root.path().join("destination");
        assert!(exclusive_move(&file, &dest, (m.dev(), m.ino(), m.uid()), None).is_err());
        assert_eq!(fs::read(&file).unwrap(), b"different fixture");
        assert!(!dest.exists());
        fs::rename(&source, root.path().join("retained")).unwrap();
        symlink(root.path().join("retained"), &source).unwrap();
        assert!(directory(&source).is_err());
        assert!(exclusive_move(&file, &dest, (m.dev(), m.ino(), m.uid()), None).is_err());
    }
    #[test]
    fn exclusive_destination_and_bounded_bundle_metadata_remain_protected() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("a");
        let dest = root.path().join("b");
        fs::write(&source, b"fixture a").unwrap();
        fs::write(&dest, b"fixture b").unwrap();
        let m = fs::metadata(&source).unwrap();
        assert!(exclusive_move(&source, &dest, (m.dev(), m.ino(), m.uid()), None).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"fixture b");
        let other = root.path().join("new destination");
        assert!(exclusive_move(
            &source,
            &other,
            (m.dev(), m.ino(), m.uid()),
            Some((m.dev(), 0))
        )
        .is_err());
        assert!(!other.exists());
        assert_eq!(fs::read(&source).unwrap(), b"fixture a");
        let app = root.path().join("Example.app");
        fs::create_dir_all(app.join("Contents")).unwrap();
        fs::hard_link(&source, app.join("Contents/Info.plist")).unwrap();
        assert!(read_bundle_info(&app).is_err());
        fs::remove_file(app.join("Contents/Info.plist")).unwrap();
        symlink(&source, app.join("Contents/Info.plist")).unwrap();
        assert!(read_bundle_info(&app).is_err());
    }
}
