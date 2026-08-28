use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{State, ipc::Channel};

use crate::{
    ManagedTerminalSessionService,
    application::get_terminal_environment::GetTerminalEnvironment,
    domain::terminal_environment::TerminalEnvironment,
    infrastructure::{
        system_terminal_environment_provider::SystemTerminalEnvironmentProvider,
        tauri_channel_terminal_event_sink::{TauriChannelTerminalEventSink, TauriTerminalEvent},
    },
};

type TerminalEnvironmentService = GetTerminalEnvironment<SystemTerminalEnvironmentProvider>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalEnvironmentResponse {
    default_shell: String,
    platform: String,
    architecture: String,
    working_directory: String,
}

impl From<TerminalEnvironment> for TerminalEnvironmentResponse {
    fn from(environment: TerminalEnvironment) -> Self {
        Self {
            default_shell: environment.default_shell,
            platform: environment.platform,
            architecture: environment.architecture,
            working_directory: environment.working_directory,
        }
    }
}

#[tauri::command]
pub fn get_terminal_environment(
    service: State<'_, TerminalEnvironmentService>,
) -> Result<TerminalEnvironmentResponse, String> {
    service
        .execute()
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTerminalInput {
    session_id: String,
    cols: u16,
    rows: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTerminalResponse {
    session_id: String,
}

#[tauri::command]
pub fn start_terminal(
    input: StartTerminalInput,
    on_event: Channel<TauriTerminalEvent>,
    service: State<'_, ManagedTerminalSessionService>,
) -> Result<StartTerminalResponse, String> {
    service
        .start(
            &input.session_id,
            input.cols,
            input.rows,
            Arc::new(TauriChannelTerminalEventSink::new(on_event)),
        )
        .map(|session_id| StartTerminalResponse { session_id })
        .map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTerminalInput {
    session_id: String,
    data: Vec<u8>,
}

#[tauri::command]
pub fn write_terminal(
    input: WriteTerminalInput,
    service: State<'_, ManagedTerminalSessionService>,
) -> Result<(), String> {
    service
        .write(&input.session_id, &input.data)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeTerminalInput {
    session_id: String,
    cols: u16,
    rows: u16,
}

#[tauri::command]
pub fn resize_terminal(
    input: ResizeTerminalInput,
    service: State<'_, ManagedTerminalSessionService>,
) -> Result<(), String> {
    service
        .resize(&input.session_id, input.cols, input.rows)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTerminalInput {
    session_id: String,
}

#[tauri::command]
pub fn stop_terminal(
    input: StopTerminalInput,
    service: State<'_, ManagedTerminalSessionService>,
) -> Result<(), String> {
    service
        .stop(&input.session_id)
        .map_err(|error| error.to_string())
}
