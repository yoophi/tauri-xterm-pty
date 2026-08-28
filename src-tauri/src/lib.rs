pub mod application;
pub mod domain;
mod inbound;
mod infrastructure;
pub mod ports;

use application::get_terminal_environment::GetTerminalEnvironment;
use application::terminal_session_service::TerminalSessionService;
use inbound::tauri_commands::{
    get_terminal_environment, resize_terminal, start_terminal, stop_terminal, write_terminal,
};
use infrastructure::{
    portable_pty_session_adapter::PortablePtySessionAdapter,
    system_terminal_environment_provider::SystemTerminalEnvironmentProvider,
};
use tauri::{Manager, WindowEvent};

pub(crate) type ManagedTerminalSessionService = TerminalSessionService<PortablePtySessionAdapter>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let terminal_environment_service =
        GetTerminalEnvironment::new(SystemTerminalEnvironmentProvider);
    let terminal_session_service = TerminalSessionService::new(PortablePtySessionAdapter::new());

    tauri::Builder::default()
        .manage(terminal_environment_service)
        .manage(terminal_session_service)
        .on_window_event(|window, event| {
            if matches!(
                event,
                WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
            ) && let Some(service) = window.try_state::<ManagedTerminalSessionService>()
            {
                let _ = service.stop_all();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_terminal_environment,
            start_terminal,
            write_terminal,
            resize_terminal,
            stop_terminal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
