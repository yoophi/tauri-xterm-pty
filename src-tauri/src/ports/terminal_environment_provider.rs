use crate::domain::terminal_environment::{TerminalEnvironment, TerminalEnvironmentError};

pub trait TerminalEnvironmentProvider: Send + Sync + 'static {
    fn detect(&self) -> Result<TerminalEnvironment, TerminalEnvironmentError>;
}
