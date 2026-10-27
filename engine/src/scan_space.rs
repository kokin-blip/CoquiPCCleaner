use anyhow::{Context, Result};
use std::{cell::Cell, fmt, path::PathBuf, time::Instant};

// Leave working room for terminal status, SQLite journals and the rest of macOS.
// This is an early-stop threshold, not a reservation or an estimate of index size.
const RESERVE: u64 = 512 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct LowSpace {
    available: u64,
    location: &'static str,
}
impl fmt::Display for LowSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scan storage is low in {} ({} MiB available; 512 MiB stopping threshold). Free working space and retry. A full-disk index can need several GiB", self.location, self.available / 1_048_576)
    }
}
impl std::error::Error for LowSpace {}

pub(crate) struct ScanSpace {
    destinations: [(PathBuf, &'static str); 2],
    checked: Cell<Instant>,
}
impl ScanSpace {
    pub(crate) fn new(database: &std::path::Path) -> Result<Self> {
        let value = Self {
            destinations: [
                (
                    database
                        .parent()
                        .context("Scan database directory is unavailable")?
                        .into(),
                    "Coqui's diagnostic filesystem",
                ),
                (std::env::temp_dir(), "the temporary scan filesystem"),
            ],
            checked: Cell::new(Instant::now()),
        };
        value.check_now()?;
        Ok(value)
    }
    pub(crate) fn check(&self) -> Result<()> {
        if self.checked.get().elapsed() >= std::time::Duration::from_millis(500) {
            self.check_now()?;
        }
        Ok(())
    }
    fn check_now(&self) -> Result<()> {
        for (path, location) in &self.destinations {
            let available = crate::platform::available_storage(path).context(
                "Cannot measure working space for scan metadata; no successful coverage is assumed",
            )?;
            ensure_available(available, location)?;
        }
        self.checked.set(Instant::now());
        Ok(())
    }
}
fn ensure_available(available: u64, location: &'static str) -> Result<()> {
    if available < RESERVE {
        return Err(LowSpace {
            available,
            location,
        }
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn low_space_reports_measured_headroom_without_claiming_completion() {
        assert!(ensure_available(RESERVE, "fixture").is_ok());
        let error = ensure_available(RESERVE - 1, "fixture").unwrap_err();
        assert!(error.downcast_ref::<LowSpace>().is_some());
        assert!(error.to_string().contains("512 MiB stopping threshold"));
        assert!(error.to_string().contains("511 MiB available"));
    }
    #[test]
    fn unavailable_filesystem_measurement_is_an_error() {
        let fixture = tempfile::tempdir().unwrap();
        assert!(crate::platform::available_storage(fixture.path()).is_ok());
        assert!(crate::platform::available_storage(&fixture.path().join("missing")).is_err());
    }
}
