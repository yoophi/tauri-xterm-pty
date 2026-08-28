use std::{
    collections::HashMap,
    io::{self, Read, Write},
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};

use crate::{
    domain::terminal_session::{
        StartTerminalRequest, TerminalEvent, TerminalSessionError, TerminalSessionId, TerminalSize,
    },
    ports::terminal_session_port::{TerminalEventSink, TerminalSessionPort},
};

use super::system_terminal_environment_provider::default_shell;

const SESSION_POLL_INTERVAL: Duration = Duration::from_millis(20);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(unix)]
const GRACEFUL_TERMINATION_TIMEOUT: Duration = Duration::from_millis(200);
const READ_BUFFER_SIZE: usize = 8 * 1024;

type SessionRegistry = Arc<(
    Mutex<HashMap<TerminalSessionId, Sender<SessionCommand>>>,
    Condvar,
)>;

enum SessionCommand {
    Write(Vec<u8>),
    Resize(TerminalSize),
    Stop,
}

pub struct PortablePtySessionAdapter {
    sessions: SessionRegistry,
}

impl PortablePtySessionAdapter {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new((Mutex::new(HashMap::new()), Condvar::new())),
        }
    }

    fn sender_for(
        &self,
        session_id: &TerminalSessionId,
    ) -> Result<Sender<SessionCommand>, TerminalSessionError> {
        lock_registry(&self.sessions)
            .get(session_id)
            .cloned()
            .ok_or_else(|| TerminalSessionError::SessionNotFound(session_id.as_str().into()))
    }

    fn send_command(
        &self,
        session_id: &TerminalSessionId,
        command: SessionCommand,
    ) -> Result<(), TerminalSessionError> {
        self.sender_for(session_id)?
            .send(command)
            .map_err(|_| TerminalSessionError::SessionNotFound(session_id.as_str().into()))
    }
}

impl Default for PortablePtySessionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PortablePtySessionAdapter {
    fn drop(&mut self) {
        let _ = self.stop_all();
    }
}

impl TerminalSessionPort for PortablePtySessionAdapter {
    fn start(
        &self,
        request: StartTerminalRequest,
        event_sink: Arc<dyn TerminalEventSink>,
    ) -> Result<(), TerminalSessionError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(to_pty_size(request.size))
            .map_err(pty_error)?;

        let mut command = CommandBuilder::new(default_shell());
        configure_shell_environment(&mut command);
        command.cwd(std::env::current_dir().map_err(|error| {
            TerminalSessionError::Pty(format!("could not resolve the working directory: {error}"))
        })?);

        let mut child = pair.slave.spawn_command(command).map_err(pty_error)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().map_err(|error| {
            let _ = child.kill();
            let _ = child.wait();
            pty_error(error)
        })?;
        let writer = pair.master.take_writer().map_err(|error| {
            let _ = child.kill();
            let _ = child.wait();
            pty_error(error)
        })?;

        let (command_sender, command_receiver) = mpsc::channel();
        let is_closing = Arc::new(AtomicBool::new(false));
        {
            let mut sessions = lock_registry(&self.sessions);
            if sessions.contains_key(&request.session_id) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalSessionError::SessionAlreadyExists(
                    request.session_id.as_str().into(),
                ));
            }
            sessions.insert(request.session_id.clone(), command_sender.clone());
        }

        let reader_thread = spawn_reader(
            reader,
            Arc::clone(&event_sink),
            command_sender,
            request.session_id.clone(),
            Arc::clone(&is_closing),
        );
        let sessions = Arc::clone(&self.sessions);
        thread::spawn(move || {
            run_session(
                request.session_id,
                pair.master,
                writer,
                child,
                command_receiver,
                reader_thread,
                event_sink,
                sessions,
                is_closing,
            );
        });

        Ok(())
    }

    fn write(
        &self,
        session_id: &TerminalSessionId,
        data: &[u8],
    ) -> Result<(), TerminalSessionError> {
        self.send_command(session_id, SessionCommand::Write(data.to_vec()))
    }

    fn resize(
        &self,
        session_id: &TerminalSessionId,
        size: TerminalSize,
    ) -> Result<(), TerminalSessionError> {
        self.send_command(session_id, SessionCommand::Resize(size))
    }

    fn stop(&self, session_id: &TerminalSessionId) -> Result<(), TerminalSessionError> {
        self.send_command(session_id, SessionCommand::Stop)
    }

    fn stop_all(&self) -> Result<(), TerminalSessionError> {
        let senders = lock_registry(&self.sessions)
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for sender in senders {
            let _ = sender.send(SessionCommand::Stop);
        }

        let (registry, condition) = &*self.sessions;
        let sessions = registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (sessions, _) = condition
            .wait_timeout_while(sessions, SHUTDOWN_TIMEOUT, |sessions| !sessions.is_empty())
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if sessions.is_empty() {
            Ok(())
        } else {
            Err(TerminalSessionError::ShutdownTimedOut)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_session(
    session_id: TerminalSessionId,
    master: Box<dyn MasterPty + Send>,
    mut writer: Box<dyn Write + Send>,
    mut child: Box<dyn Child + Send + Sync>,
    command_receiver: Receiver<SessionCommand>,
    reader_thread: JoinHandle<()>,
    event_sink: Arc<dyn TerminalEventSink>,
    sessions: SessionRegistry,
    is_closing: Arc<AtomicBool>,
) {
    let mut final_status = None;
    let mut should_terminate = false;

    loop {
        match command_receiver.recv_timeout(SESSION_POLL_INTERVAL) {
            Ok(SessionCommand::Write(data)) => {
                if let Err(error) = writer.write_all(&data).and_then(|_| writer.flush()) {
                    send_error(&event_sink, format!("could not write to the PTY: {error}"));
                    should_terminate = true;
                    break;
                }
            }
            Ok(SessionCommand::Resize(size)) => {
                if let Err(error) = master.resize(to_pty_size(size)) {
                    send_error(&event_sink, format!("could not resize the PTY: {error}"));
                }
            }
            Ok(SessionCommand::Stop) | Err(RecvTimeoutError::Disconnected) => {
                should_terminate = true;
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                final_status = Some(status);
                break;
            }
            Ok(None) => {}
            Err(error) => {
                send_error(
                    &event_sink,
                    format!("could not inspect the shell process: {error}"),
                );
                should_terminate = true;
                break;
            }
        }
    }

    is_closing.store(true, Ordering::Release);

    if should_terminate {
        final_status = terminate_child(&mut *child, &*master);
    }

    drop(writer);
    drop(master);
    let _ = reader_thread.join();

    let (exit_code, signal) = status_details(final_status);
    let _ = event_sink.send(TerminalEvent::Exited { exit_code, signal });

    let (_, condition) = &*sessions;
    lock_registry(&sessions).remove(&session_id);
    condition.notify_all();
}

fn configure_shell_environment(command: &mut CommandBuilder) {
    command.env_remove("NO_COLOR");
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("TERM_PROGRAM", "tauri-xterm-pty");
}

fn spawn_reader(
    mut reader: Box<dyn Read + Send>,
    event_sink: Arc<dyn TerminalEventSink>,
    command_sender: Sender<SessionCommand>,
    session_id: TerminalSessionId,
    is_closing: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = vec![0; READ_BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    if event_sink
                        .send(TerminalEvent::Output(buffer[..bytes_read].to_vec()))
                        .is_err()
                    {
                        let _ = command_sender.send(SessionCommand::Stop);
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if is_pty_end_of_stream(&error) => break,
                Err(error) => {
                    if !is_closing.load(Ordering::Acquire) {
                        send_error(
                            &event_sink,
                            format!("could not read PTY output for {session_id:?}: {error}"),
                        );
                        let _ = command_sender.send(SessionCommand::Stop);
                    }
                    break;
                }
            }
        }
    })
}

#[cfg(unix)]
fn is_pty_end_of_stream(error: &io::Error) -> bool {
    error.raw_os_error() == Some(libc::EIO)
}

#[cfg(not(unix))]
fn is_pty_end_of_stream(_error: &io::Error) -> bool {
    false
}

fn send_error(event_sink: &Arc<dyn TerminalEventSink>, message: String) {
    let _ = event_sink.send(TerminalEvent::Error(message));
}

fn status_details(status: Option<ExitStatus>) -> (Option<u32>, Option<String>) {
    match status {
        Some(status) => (
            Some(status.exit_code()),
            status.signal().map(ToOwned::to_owned),
        ),
        None => (None, None),
    }
}

fn terminate_child(
    child: &mut (dyn Child + Send + Sync),
    master: &(dyn MasterPty + Send),
) -> Option<ExitStatus> {
    #[cfg(unix)]
    {
        if let Some(process_group) = master.process_group_leader() {
            // SAFETY: portable-pty only returns positive process-group identifiers here.
            unsafe {
                libc::killpg(process_group, libc::SIGHUP);
            }
        }
        if let Some(process_id) = child.process_id() {
            // SAFETY: the process id comes from the child created by portable-pty.
            unsafe {
                libc::kill(process_id as libc::pid_t, libc::SIGHUP);
            }
        }

        let deadline = std::time::Instant::now() + GRACEFUL_TERMINATION_TIMEOUT;
        while std::time::Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(status)) => return Some(status),
                Ok(None) => thread::sleep(SESSION_POLL_INTERVAL),
                Err(_) => break,
            }
        }
    }

    let _ = child.kill();
    child.wait().ok()
}

fn to_pty_size(size: TerminalSize) -> PtySize {
    PtySize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn pty_error(error: impl std::fmt::Display) -> TerminalSessionError {
    TerminalSessionError::Pty(error.to_string())
}

fn lock_registry(
    sessions: &SessionRegistry,
) -> MutexGuard<'_, HashMap<TerminalSessionId, Sender<SessionCommand>>> {
    sessions
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use std::ffi::OsStr;
    use std::time::Instant;

    use super::*;

    #[derive(Clone, Default)]
    struct RecordingSink {
        events: Arc<(Mutex<Vec<TerminalEvent>>, Condvar)>,
    }

    impl TerminalEventSink for RecordingSink {
        fn send(&self, event: TerminalEvent) -> Result<(), TerminalSessionError> {
            let (events, condition) = &*self.events;
            events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(event);
            condition.notify_all();
            Ok(())
        }
    }

    #[test]
    fn shell_command_enables_color_output() {
        let mut command = CommandBuilder::new(default_shell());
        command.env("NO_COLOR", "1");

        configure_shell_environment(&mut command);

        assert_eq!(command.get_env("TERM"), Some(OsStr::new("xterm-256color")));
        assert_eq!(command.get_env("COLORTERM"), Some(OsStr::new("truecolor")));
        assert_eq!(
            command.get_env("TERM_PROGRAM"),
            Some(OsStr::new("tauri-xterm-pty"))
        );
        assert_eq!(command.get_env("NO_COLOR"), None);
    }

    #[test]
    fn streams_shell_output_and_reports_exit() {
        let adapter = PortablePtySessionAdapter::new();
        let sink = RecordingSink::default();
        let session_id = TerminalSessionId::parse("6f2d3b84-14d8-4a08-8c52-1ec979fde360")
            .expect("valid test session id");

        adapter
            .start(
                StartTerminalRequest {
                    session_id: session_id.clone(),
                    size: TerminalSize::new(80, 24).expect("valid test size"),
                },
                Arc::new(sink.clone()),
            )
            .expect("PTY should start");
        adapter
            .write(&session_id, b"printf '__PTY_SMOKE_OK__\\n'; exit\r")
            .expect("input should be written");

        let deadline = Instant::now() + Duration::from_secs(10);
        let (events, condition) = &*sink.events;
        let mut recorded = events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !recorded
            .iter()
            .any(|event| matches!(event, TerminalEvent::Exited { .. }))
        {
            let now = Instant::now();
            assert!(now < deadline, "PTY did not exit before the test deadline");
            let wait = deadline.saturating_duration_since(now);
            recorded = condition
                .wait_timeout(recorded, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }

        let output = recorded
            .iter()
            .filter_map(|event| match event {
                TerminalEvent::Output(data) => Some(data.as_slice()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert!(
            String::from_utf8_lossy(&output).contains("__PTY_SMOKE_OK__"),
            "shell output should include the smoke-test marker"
        );
        assert!(
            !recorded
                .iter()
                .any(|event| matches!(event, TerminalEvent::Error(_))),
            "a normal shell exit must not be reported as a PTY error"
        );
    }
}
