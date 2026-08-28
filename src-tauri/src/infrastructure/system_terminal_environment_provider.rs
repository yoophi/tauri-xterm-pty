use std::env;

use crate::{
    domain::terminal_environment::{TerminalEnvironment, TerminalEnvironmentError},
    ports::terminal_environment_provider::TerminalEnvironmentProvider,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTerminalEnvironmentProvider;

impl TerminalEnvironmentProvider for SystemTerminalEnvironmentProvider {
    fn detect(&self) -> Result<TerminalEnvironment, TerminalEnvironmentError> {
        let working_directory = env::current_dir()
            .map_err(|error| {
                TerminalEnvironmentError::CurrentDirectoryUnavailable(error.to_string())
            })?
            .to_string_lossy()
            .into_owned();

        Ok(TerminalEnvironment {
            default_shell: default_shell(),
            platform: env::consts::OS.into(),
            architecture: env::consts::ARCH.into(),
            working_directory,
        })
    }
}

pub(super) fn default_shell() -> String {
    #[cfg(target_os = "windows")]
    let shell = env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into());

    #[cfg(not(target_os = "windows"))]
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    shell
}
