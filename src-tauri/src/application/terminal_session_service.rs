use std::sync::Arc;

use crate::{
    domain::terminal_session::{
        MAX_TERMINAL_INPUT_BYTES, StartTerminalRequest, TerminalSessionError, TerminalSessionId,
        TerminalSize,
    },
    ports::terminal_session_port::{TerminalEventSink, TerminalSessionPort},
};

pub struct TerminalSessionService<P> {
    port: P,
}

impl<P> TerminalSessionService<P>
where
    P: TerminalSessionPort,
{
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn start(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
        event_sink: Arc<dyn TerminalEventSink>,
    ) -> Result<String, TerminalSessionError> {
        let session_id = TerminalSessionId::parse(session_id)?;
        let size = TerminalSize::new(cols, rows)?;
        let response_id = session_id.as_str().to_owned();

        self.port
            .start(StartTerminalRequest { session_id, size }, event_sink)?;

        Ok(response_id)
    }

    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), TerminalSessionError> {
        if data.len() > MAX_TERMINAL_INPUT_BYTES {
            return Err(TerminalSessionError::InputTooLarge { bytes: data.len() });
        }

        self.port
            .write(&TerminalSessionId::parse(session_id)?, data)
    }

    pub fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TerminalSessionError> {
        self.port.resize(
            &TerminalSessionId::parse(session_id)?,
            TerminalSize::new(cols, rows)?,
        )
    }

    pub fn stop(&self, session_id: &str) -> Result<(), TerminalSessionError> {
        self.port.stop(&TerminalSessionId::parse(session_id)?)
    }

    pub fn stop_all(&self) -> Result<(), TerminalSessionError> {
        self.port.stop_all()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::domain::terminal_session::TerminalEvent;

    #[derive(Default)]
    struct FakeTerminalPort {
        starts: Mutex<Vec<StartTerminalRequest>>,
        writes: Mutex<Vec<Vec<u8>>>,
    }

    impl TerminalSessionPort for FakeTerminalPort {
        fn start(
            &self,
            request: StartTerminalRequest,
            _event_sink: Arc<dyn TerminalEventSink>,
        ) -> Result<(), TerminalSessionError> {
            self.starts.lock().expect("starts lock").push(request);
            Ok(())
        }

        fn write(
            &self,
            _session_id: &TerminalSessionId,
            data: &[u8],
        ) -> Result<(), TerminalSessionError> {
            self.writes.lock().expect("writes lock").push(data.to_vec());
            Ok(())
        }

        fn resize(
            &self,
            _session_id: &TerminalSessionId,
            _size: TerminalSize,
        ) -> Result<(), TerminalSessionError> {
            Ok(())
        }

        fn stop(&self, _session_id: &TerminalSessionId) -> Result<(), TerminalSessionError> {
            Ok(())
        }

        fn stop_all(&self) -> Result<(), TerminalSessionError> {
            Ok(())
        }
    }

    struct NoopSink;

    impl TerminalEventSink for NoopSink {
        fn send(&self, _event: TerminalEvent) -> Result<(), TerminalSessionError> {
            Ok(())
        }
    }

    #[test]
    fn validates_requests_before_delegating_to_the_port() {
        let service = TerminalSessionService::new(FakeTerminalPort::default());
        let session_id = "6f2d3b84-14d8-4a08-8c52-1ec979fde360";

        let started_id = service
            .start(session_id, 100, 30, Arc::new(NoopSink))
            .expect("session should start");
        service
            .write(session_id, b"echo test\r")
            .expect("input should be accepted");

        assert_eq!(started_id, session_id);
        assert_eq!(service.port.starts.lock().expect("starts lock").len(), 1);
        assert_eq!(
            service.port.writes.lock().expect("writes lock").as_slice(),
            [b"echo test\r".to_vec()]
        );
        assert!(
            service
                .start("invalid", 80, 24, Arc::new(NoopSink))
                .is_err()
        );
        let oversized_input = vec![0; MAX_TERMINAL_INPUT_BYTES + 1];
        assert!(service.write(session_id, &oversized_input).is_err());
    }
}
