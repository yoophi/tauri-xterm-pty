use crate::{
    domain::terminal_environment::{TerminalEnvironment, TerminalEnvironmentError},
    ports::terminal_environment_provider::TerminalEnvironmentProvider,
};

pub struct GetTerminalEnvironment<P> {
    provider: P,
}

impl<P> GetTerminalEnvironment<P>
where
    P: TerminalEnvironmentProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn execute(&self) -> Result<TerminalEnvironment, TerminalEnvironmentError> {
        self.provider.detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTerminalEnvironmentProvider;

    impl TerminalEnvironmentProvider for FakeTerminalEnvironmentProvider {
        fn detect(&self) -> Result<TerminalEnvironment, TerminalEnvironmentError> {
            Ok(TerminalEnvironment {
                default_shell: "/bin/test-shell".into(),
                platform: "test-os".into(),
                architecture: "test-arch".into(),
                working_directory: "/test/workspace".into(),
            })
        }
    }

    #[test]
    fn returns_environment_from_the_outbound_port() {
        let use_case = GetTerminalEnvironment::new(FakeTerminalEnvironmentProvider);

        let environment = use_case.execute().expect("environment should be detected");

        assert_eq!(environment.default_shell, "/bin/test-shell");
        assert_eq!(environment.platform, "test-os");
    }
}
