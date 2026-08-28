use std::fmt;

use uuid::Uuid;

pub const MAX_TERMINAL_INPUT_BYTES: usize = 64 * 1024;
const MAX_TERMINAL_DIMENSION: u16 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerminalSessionId(String);

impl TerminalSessionId {
    pub fn parse(value: &str) -> Result<Self, TerminalSessionError> {
        Uuid::parse_str(value)
            .map(|id| Self(id.to_string()))
            .map_err(|_| TerminalSessionError::InvalidSessionId)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl TerminalSize {
    pub fn new(cols: u16, rows: u16) -> Result<Self, TerminalSessionError> {
        if cols == 0 || rows == 0 || cols > MAX_TERMINAL_DIMENSION || rows > MAX_TERMINAL_DIMENSION
        {
            return Err(TerminalSessionError::InvalidSize { cols, rows });
        }

        Ok(Self { cols, rows })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartTerminalRequest {
    pub session_id: TerminalSessionId,
    pub size: TerminalSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    Output(Vec<u8>),
    Exited {
        exit_code: Option<u32>,
        signal: Option<String>,
    },
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalSessionError {
    InvalidSessionId,
    InvalidSize { cols: u16, rows: u16 },
    InputTooLarge { bytes: usize },
    SessionAlreadyExists(String),
    SessionNotFound(String),
    Pty(String),
    Transport(String),
    ShutdownTimedOut,
}

impl fmt::Display for TerminalSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSessionId => write!(formatter, "terminal session id must be a UUID"),
            Self::InvalidSize { cols, rows } => write!(
                formatter,
                "terminal size must be between 1 and {MAX_TERMINAL_DIMENSION}: {cols}x{rows}"
            ),
            Self::InputTooLarge { bytes } => write!(
                formatter,
                "terminal input exceeds the {MAX_TERMINAL_INPUT_BYTES} byte limit: {bytes}"
            ),
            Self::SessionAlreadyExists(session_id) => {
                write!(formatter, "terminal session already exists: {session_id}")
            }
            Self::SessionNotFound(session_id) => {
                write!(formatter, "terminal session was not found: {session_id}")
            }
            Self::Pty(message) => write!(formatter, "PTY operation failed: {message}"),
            Self::Transport(message) => {
                write!(formatter, "terminal transport failed: {message}")
            }
            Self::ShutdownTimedOut => write!(formatter, "terminal shutdown timed out"),
        }
    }
}

impl std::error::Error for TerminalSessionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_session_ids_and_terminal_sizes() {
        assert!(TerminalSessionId::parse("6f2d3b84-14d8-4a08-8c52-1ec979fde360").is_ok());
        assert!(TerminalSessionId::parse("not-a-session").is_err());
        assert_eq!(
            TerminalSize::new(80, 24),
            Ok(TerminalSize { cols: 80, rows: 24 })
        );
        assert!(TerminalSize::new(0, 24).is_err());
        assert!(TerminalSize::new(80, 1_001).is_err());
    }
}
