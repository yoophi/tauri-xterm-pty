use std::sync::Arc;

use crate::domain::terminal_session::{
    StartTerminalRequest, TerminalEvent, TerminalSessionError, TerminalSessionId, TerminalSize,
};

pub trait TerminalEventSink: Send + Sync + 'static {
    fn send(&self, event: TerminalEvent) -> Result<(), TerminalSessionError>;
}

pub trait TerminalSessionPort: Send + Sync + 'static {
    fn start(
        &self,
        request: StartTerminalRequest,
        event_sink: Arc<dyn TerminalEventSink>,
    ) -> Result<(), TerminalSessionError>;

    fn write(
        &self,
        session_id: &TerminalSessionId,
        data: &[u8],
    ) -> Result<(), TerminalSessionError>;

    fn resize(
        &self,
        session_id: &TerminalSessionId,
        size: TerminalSize,
    ) -> Result<(), TerminalSessionError>;

    fn stop(&self, session_id: &TerminalSessionId) -> Result<(), TerminalSessionError>;

    fn stop_all(&self) -> Result<(), TerminalSessionError>;
}
