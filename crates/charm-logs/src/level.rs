use std::fmt;
use std::str::FromStr;

/// Severity of a log entry, ordered from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Level {
    /// Verbose diagnostic information.
    #[default]
    Debug = 0,
    /// Normal application event.
    Info = 1,
    /// Unexpected condition that does not stop execution.
    Warn = 2,
    /// Failure.
    Error = 3,
}

impl Level {
    /// Lowercase level name; matches Go's `String()` method.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    /// Short three-letter label used in the rendered entry line.
    pub const fn short(self) -> &'static str {
        match self {
            Self::Debug => "DBG",
            Self::Info => "INF",
            Self::Warn => "WRN",
            Self::Error => "ERR",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned when parsing an unknown level name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLevelError(pub String);

impl fmt::Display for ParseLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "charm-logs: invalid level {:?}", self.0)
    }
}
impl std::error::Error for ParseLevelError {}

impl FromStr for Level {
    type Err = ParseLevelError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(ParseLevelError(input.to_owned())),
        }
    }
}

/// Parses a level name, accepting `debug`, `info`, `warn`, `warning`, `error`.
pub fn parse_level(input: &str) -> Result<Level, ParseLevelError> {
    input.parse()
}
