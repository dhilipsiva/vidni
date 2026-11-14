//! A small client for the Chrome DevTools Protocol, over the pipes that
//! Chromium opens with `--remote-debugging-pipe` (P3-VIDNI-04).
//!
//! Chromium reads commands on its file descriptor 3 and writes answers and
//! events on 4, each message ending in a NUL byte. Nothing listens on a
//! port, and Chromium exits when its pipes close, so it can't outlive vidni.
//! A thread reads the messages. Commands are answered in order, and the
//! events vidni acts on are kept until it asks for them. A JavaScript dialog
//! is answered as soon as it opens (dismissed, or accepted when the page
//! asks whether to leave) and kept as an event, so a page can't stall a
//! capture. A renderer that crashes fails the wait at once. Each browser
//! runs on a fresh profile in a temporary folder, which is also its
//! `TMPDIR` when paths allow, and is removed after Chromium has exited.
//!
//! `VIDNI_CDP_DEBUG=1` prints every message from Chromium to stderr.

use command_fds::{CommandFdExt, FdMapping};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long one command may take to answer.
const ANSWER: Duration = Duration::from_secs(60);
/// How long Chromium may take to start answering.
const START: Duration = Duration::from_secs(30);
/// How long Chromium may take to exit once asked.
const EXIT: Duration = Duration::from_secs(10);
/// How often a wait looks for an interruption.
const GLANCE: Duration = Duration::from_millis(100);
/// The events kept for vidni; others are dropped as they arrive.
const KEPT: &[&str] = &[
    "Page.loadEventFired",
    "Page.frameNavigated",
    "Page.javascriptDialogOpening",
    "Runtime.exceptionThrown",
    "Network.requestWillBeSent",
    "Network.loadingFinished",
    "Network.loadingFailed",
];
/// The most events kept; the oldest go first.
const MOST_KEPT: usize = 10_000;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicUsize = AtomicUsize::new(0);

/// Asks every running browser to stop: its next wait fails, so that a
/// capture unwinds, Chromium exits and its profile is removed. Returns false
/// when no browser is running, or when asked a second time; the caller, a
/// signal handler, should then exit at once.
pub fn interrupt() -> bool {
    RUNNING.load(Ordering::SeqCst) > 0 && !INTERRUPTED.swap(true, Ordering::SeqCst)
}

/// Whether [`interrupt`] has stopped a browser.
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// A running Chromium.
pub struct Browser {
    child: Child,
    commands: Option<std::io::PipeWriter>,
    messages: Receiver<Result<Value, String>>,
    said: Arc<Mutex<VecDeque<String>>>,
    /// Closes when Chromium's stderr does: everything it said is in `said`.
    said_all: Receiver<()>,
    next_id: u64,
    events: VecDeque<Value>,
    profile: Option<tempfile::TempDir>,
    debug: bool,
}

impl Browser {
    /// Starts `executable` with `args`, on a fresh profile, and waits until
    /// it answers.
    pub fn launch(executable: &Path, args: &[&str]) -> Result<Self, String> {
        let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", executable.display());
        let profile = tempfile::Builder::new()
            .prefix("vidni-")
            .tempdir()
            .map_err(|e| shown(&e))?;
        // Chromium's temporary files go into the profile, to be removed with
        // it, unless that makes the path of the socket Chromium keeps there
        // too long: a socket's path holds 107 bytes.
        let socket = "/org.chromium.Chromium.XXXXXX/SingletonSocket".len();
        let roomy = profile.path().as_os_str().len() + socket <= 107;
        let (commands_in, commands) = std::io::pipe().map_err(|e| shown(&e))?;
        let (messages_in, messages_out) = std::io::pipe().map_err(|e| shown(&e))?;
        let mut command = Command::new(executable);
        command
            .args(args)
            .arg("--remote-debugging-pipe")
            .arg(format!(
                "--user-data-dir={}",
                profile.path().join("data").display()
            ))
            .arg("about:blank")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if roomy {
            command.env("TMPDIR", profile.path());
        }
        command
            .fd_mappings(vec![
                FdMapping {
                    parent_fd: commands_in.into(),
                    child_fd: 3,
                },
                FdMapping {
                    parent_fd: messages_out.into(),
                    child_fd: 4,
                },
            ])
            .map_err(|e| shown(&e))?;
        let mut child = command.spawn().map_err(|e| shown(&e))?;
        // The command holds the child's ends of the pipes; dropping it closes
        // ours, so that Chromium's exit closes the message pipe.
        drop(command);
        RUNNING.fetch_add(1, Ordering::SeqCst);
        // The last lines Chromium says on stderr, for a failure's message.
        let said = Arc::new(Mutex::new(VecDeque::new()));
        let stderr = child.stderr.take();
        let lines = Arc::clone(&said);
        let (heard, said_all) = channel::<()>();
        std::thread::spawn(move || {
            // Dropped when this thread ends, which closes `said_all`.
            let _heard = heard;
            let Some(stderr) = stderr else { return };
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut said) = lines.lock() {
                    said.push_back(line);
                    if said.len() > 12 {
                        said.pop_front();
                    }
                }
            }
        });
        let (sender, messages) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(messages_in);
            loop {
                let mut message = Vec::new();
                let (sent, done) = match reader.read_until(0, &mut message) {
                    Ok(0) => (
                        sender.send(Err("Chromium closed its pipe".to_string())),
                        true,
                    ),
                    Ok(_) => {
                        message.pop();
                        let parsed = serde_json::from_slice(&message).map_err(|e| e.to_string());
                        (sender.send(parsed), false)
                    }
                    Err(e) => (sender.send(Err(e.to_string())), true),
                };
                if sent.is_err() || done {
                    break;
                }
            }
        });
        let mut browser = Self {
            child,
            commands: Some(commands),
            messages,
            said,
            said_all,
            next_id: 1,
            events: VecDeque::new(),
            profile: Some(profile),
            debug: std::env::var_os("VIDNI_CDP_DEBUG").is_some_and(|v| !v.is_empty()),
        };
        browser
            .call_within(None, "Browser.getVersion", json!({}), START)
            .map_err(|e| shown(&e))?;
        Ok(browser)
    }

    /// Sends `method` to the browser, or to a page's `session`, and returns
    /// its result.
    pub fn call(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Value, String> {
        self.call_within(session, method, params, ANSWER)
    }

    fn call_within(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: Value,
        time: Duration,
    ) -> Result<Value, String> {
        let id = self.send(session, method, params)?;
        let deadline = Instant::now() + time;
        loop {
            let message = self.next(method, deadline)?;
            if message["id"] == id {
                if let Some(error) = message.get("error") {
                    return Err(format!("{method}: {}", error["message"]));
                }
                return Ok(message["result"].clone());
            }
        }
    }

    fn send(&mut self, session: Option<&str>, method: &str, params: Value) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        let mut message = json!({"id": id, "method": method, "params": params});
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        let mut bytes = message.to_string().into_bytes();
        bytes.push(0);
        let pipe = self.commands.as_mut().ok_or("Chromium is closing")?;
        // A Chromium that has stopped may have said why, as one that closes
        // its other pipe may.
        if let Err(e) = pipe.write_all(&bytes).and_then(|_| pipe.flush()) {
            return Err(format!(
                "{method}: Chromium stopped reading: {e}{}",
                self.last_words()
            ));
        }
        Ok(id)
    }

    /// The next message from Chromium before `deadline`, an answer or an
    /// event. Events are handled on the way: a dialog is dismissed, and the
    /// kept ones are queued.
    fn next(&mut self, waiting_for: &str, deadline: Instant) -> Result<Value, String> {
        let message = loop {
            if interrupted() {
                return Err("interrupted".into());
            }
            let left = deadline.saturating_duration_since(Instant::now());
            match self.messages.recv_timeout(left.min(GLANCE)) {
                Ok(Ok(message)) => break message,
                Ok(Err(e)) => return Err(format!("{waiting_for}: {e}{}", self.last_words())),
                Err(RecvTimeoutError::Timeout) if left > GLANCE => continue,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(format!("{waiting_for}: Chromium didn't answer in time"));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(format!(
                        "{waiting_for}: Chromium stopped{}",
                        self.last_words()
                    ));
                }
            }
        };
        if self.debug {
            let text = message.to_string();
            let short: String = text.chars().take(300).collect();
            eprintln!("cdp <- {short}");
        }
        if let Some(method) = message["method"].as_str() {
            if method == "Page.javascriptDialogOpening" {
                // A page asking whether to leave is told yes, so that a
                // navigation goes on; any other dialog is dismissed.
                let leave = message["params"]["type"] == "beforeunload";
                let session = message["sessionId"].as_str().map(str::to_string);
                self.send(
                    session.as_deref(),
                    "Page.handleJavaScriptDialog",
                    json!({"accept": leave}),
                )?;
            }
            if method == "Inspector.targetCrashed" || method == "Target.targetCrashed" {
                return Err("the page's renderer crashed".into());
            }
            if KEPT.contains(&method) {
                if self.events.len() == MOST_KEPT {
                    self.events.pop_front();
                }
                self.events.push_back(message.clone());
            }
        }
        Ok(message)
    }

    /// Waits up to `timeout` for the next `method` event from `session`, and
    /// returns its parameters. Kept events of that method are dropped.
    pub fn event(
        &mut self,
        session: &str,
        method: &str,
        timeout: Duration,
    ) -> Result<Value, String> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut found = self.take(session, method);
            if !found.is_empty() {
                return Ok(found.remove(0));
            }
            self.next(method, deadline)?;
        }
    }

    /// Takes the kept events of `method` from `session`, oldest first, as
    /// their parameters.
    pub fn take(&mut self, session: &str, method: &str) -> Vec<Value> {
        let (taken, kept): (Vec<Value>, Vec<Value>) = self
            .events
            .drain(..)
            .partition(|v| v["method"] == method && v["sessionId"] == session);
        self.events = kept.into();
        taken.into_iter().map(|v| v["params"].clone()).collect()
    }

    /// Reads whatever Chromium sends for `time`, keeping its events.
    pub fn idle(&mut self, time: Duration) -> Result<(), String> {
        let deadline = Instant::now() + time;
        while Instant::now() < deadline {
            match self.next("waiting", deadline) {
                Ok(_) => continue,
                Err(_) if !interrupted() && Instant::now() >= deadline => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Drops every kept event.
    pub fn forget_events(&mut self) {
        self.events.clear();
    }

    /// What Chromium said last, once its pipe has closed. A Chromium that
    /// has stopped closes its stderr too, so the lines still on their way
    /// are waited for, briefly.
    fn last_words(&self) -> String {
        let _ = self.said_all.recv_timeout(Duration::from_millis(500));
        self.last_said()
    }

    fn last_said(&self) -> String {
        match self.said.lock() {
            Ok(said) if !said.is_empty() => format!(
                "; it said:\n{}",
                said.iter().cloned().collect::<Vec<_>>().join("\n")
            ),
            _ => String::new(),
        }
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        // Asked to close, and with its command pipe closed, Chromium shuts
        // its own processes down; it is killed only if it doesn't exit in
        // time. The profile goes once it has exited.
        let _ = self.send(None, "Browser.close", json!({}));
        self.commands = None;
        let deadline = Instant::now() + EXIT;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                _ => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    break;
                }
            }
        }
        self.profile.take();
        RUNNING.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `script` with `sh` where Chromium would run.
    fn launch(script: &str) -> Result<Browser, String> {
        Browser::launch(Path::new("sh"), &["-c", script])
    }

    #[test]
    fn a_browser_that_exits_at_once_is_reported_with_what_it_said() {
        let started = Instant::now();
        let error = launch("echo no display >&2; exit 3").err().unwrap();
        assert!(error.contains("sh: Browser.getVersion"), "{error}");
        assert!(error.contains("no display"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_program_that_isnt_there_is_named() {
        let error = Browser::launch(Path::new("/nonexistent/chromium"), &[])
            .err()
            .unwrap();
        assert!(error.starts_with("/nonexistent/chromium: "), "{error}");
    }
}
