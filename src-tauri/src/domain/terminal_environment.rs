use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalEnvironment {
    pub default_shell: String,
    pub platform: String,
    pub architecture: String,
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEnvironmentError {
    CurrentDirectoryUnavailable(String),
}

impl fmt::Display for TerminalEnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectoryUnavailable(message) => {
                write!(formatter, "current directory is unavailable: {message}")
            }
        }
    }
}

impl std::error::Error for TerminalEnvironmentError {}
