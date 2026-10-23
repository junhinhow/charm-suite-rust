use crate::{model::sanitize_text, Entry};
use std::{
    fmt,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

/// Plain text export or filesystem write failure.
#[derive(Debug)]
pub struct ExportError(std::io::Error);
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        Self(e)
    }
}

/// Result message emitted by the `e` export shortcut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedMsg {
    /// Written filename (`logs.txt`).
    pub path: String,
    /// Error message, if the write failed.
    pub error: Option<String>,
}

pub(crate) fn write_entries(w: &mut dyn Write, entries: &[Entry]) -> Result<(), ExportError> {
    for entry in entries {
        let (stamp, offset) = timestamp(entry.time);
        let source = sanitize_text(&entry.source);
        let line = if source.is_empty() {
            format!(
                "{stamp}{offset} {} {}\n",
                entry.level.to_string().to_uppercase(),
                sanitize_text(&entry.message)
            )
        } else {
            format!(
                "{stamp}{offset} {} [{source}] {}\n",
                entry.level.to_string().to_uppercase(),
                sanitize_text(&entry.message)
            )
        };
        w.write_all(line.as_bytes())?;
    }
    Ok(())
}

// SystemTime has no timezone metadata; its UTC representation is equivalent
// to the canonical Go output for UTC instants.
pub(crate) fn timestamp(time: SystemTime) -> (String, &'static str) {
    let secs = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or(i64::MAX);
    let sod = secs % 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (
        format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
            sod / 3600,
            (sod % 3600) / 60,
            sod % 60
        ),
        "Z",
    )
}
