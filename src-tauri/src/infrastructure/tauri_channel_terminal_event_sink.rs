use tauri::ipc::Channel;

use crate::{
    domain::terminal_session::{TerminalEvent, TerminalSessionError},
    ports::terminal_session_port::TerminalEventSink,
};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TauriTerminalEvent {
    Output {
        data: Vec<u8>,
    },
    Exited {
        #[serde(rename = "exitCode")]
        exit_code: Option<u32>,
        signal: Option<String>,
    },
    Error {
        message: String,
    },
}

pub struct TauriChannelTerminalEventSink {
    channel: Channel<TauriTerminalEvent>,
}

impl TauriChannelTerminalEventSink {
    pub fn new(channel: Channel<TauriTerminalEvent>) -> Self {
        Self { channel }
    }
}

impl TerminalEventSink for TauriChannelTerminalEventSink {
    fn send(&self, event: TerminalEvent) -> Result<(), TerminalSessionError> {
        let event = match event {
            TerminalEvent::Output(data) => TauriTerminalEvent::Output { data },
            TerminalEvent::Exited { exit_code, signal } => {
                TauriTerminalEvent::Exited { exit_code, signal }
            }
            TerminalEvent::Error(message) => TauriTerminalEvent::Error { message },
        };

        self.channel
            .send(event)
            .map_err(|error| TerminalSessionError::Transport(error.to_string()))
    }
}
