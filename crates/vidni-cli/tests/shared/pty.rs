//! `vidni record` in a pseudo-terminal, for tests: its keys are pressed
//! and its screen is read. The crate that includes this says where the
//! `vidni` binary is, as `crate::vidni_binary()`.

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const READY: &str = "space record · p play the saved take";
pub const REVIEW: &str =
    "enter keep and go on · p play · r record again · esc discard · q keep and quit";

/// Three lines in two sections. The long cut has them all, and the short
/// the last two: `main-2` is in both.
pub const PROJECT: &str = r#"schema = 1

[meta]
title = "A recording"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["main-2", "end-1"]

[sections.main]
label = "The main part"

[sections.end]
label = "The end"

[[lines]]
id = "main-1"
section = "main"
show = "The first line is only in the long cut."

[[lines]]
id = "main-2"
section = "main"
show = "The second line is in both cuts."

[[lines]]
id = "end-1"
section = "end"
show = "The last line ends them both."

[scenes.page]
kind = "web"
url = "https://example.com"
"#;

/// A `vidni record` running in a pseudo-terminal.
pub struct Recorder {
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    keys: Box<dyn Write + Send>,
    /// Everything it has written so far.
    pub written: Arc<Mutex<Vec<u8>>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

/// `text` without its terminal escapes.
fn plain(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        // An escape: `ESC [`, its parameters, and the letter that ends it.
        if chars.next_if_eq(&'[').is_some() {
            while chars.next_if(|c| !c.is_ascii_alphabetic()).is_some() {}
            chars.next();
        }
    }
    out
}

impl Recorder {
    /// Starts `vidni record` on the project in `dir` with `args`, recording
    /// from the made-up `input`, with no listening check.
    pub fn start(dir: &Path, args: &[&str], input: &str) -> Self {
        Self::start_with(dir, args, input, false)
    }

    /// [`Self::start`], with the listening check when `listening`.
    pub fn start_with(dir: &Path, args: &[&str], input: &str, listening: bool) -> Self {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 40,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new(crate::vidni_binary());
        command.arg("record");
        command.arg(dir);
        command.args(args);
        command.env("TERM", "xterm-256color");
        command.env("VIDNI_RECORD_INPUT", input);
        // ALSA's null device takes what is played and makes no sound.
        command.env("VIDNI_SPEAKER", "Discard all samples");
        if !listening {
            // Its model is looked for where there is none.
            command.env_remove("VIDNI_WHISPER_MODEL");
            command.env("XDG_DATA_HOME", dir.join("no-data"));
        }
        let child = pty.slave.spawn_command(command).unwrap();
        drop(pty.slave);
        let mut output = pty.master.try_clone_reader().unwrap();
        let keys = pty.master.take_writer().unwrap();
        let written = Arc::new(Mutex::new(Vec::new()));
        let sink = written.clone();
        let reader = std::thread::spawn(move || {
            // The master stays open for as long as the reader has it.
            let _master = pty.master;
            let mut block = [0u8; 4096];
            while let Ok(n) = output.read(&mut block) {
                if n == 0 {
                    break;
                }
                sink.lock().unwrap().extend_from_slice(&block[..n]);
            }
        });
        Self {
            child,
            keys,
            written,
            reader: Some(reader),
        }
    }

    /// What is on the screen: what was written since it was last cleared.
    pub fn screen(&self) -> String {
        let written = String::from_utf8_lossy(&self.written.lock().unwrap()).into_owned();
        let shown = written.rsplit("\x1b[2J").next().unwrap_or_default();
        plain(shown)
    }

    /// Waits until the screen shows `text`, and returns the screen.
    pub fn wait_for(&self, text: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let screen = self.screen();
            if screen.contains(text) {
                return screen;
            }
            assert!(
                Instant::now() < deadline,
                "the screen never showed {text:?}:\n{screen}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Waits for the review of a new take, and until it takes keys: for a
    /// moment after it appears, keys are ignored.
    pub fn review(&self) -> String {
        let screen = self.wait_for(REVIEW);
        std::thread::sleep(Duration::from_millis(600));
        screen
    }

    pub fn press(&mut self, keys: &str) {
        self.keys.write_all(keys.as_bytes()).unwrap();
        self.keys.flush().unwrap();
    }

    /// Presses space, waits until the take is being recorded, lets it run
    /// for `seconds`, and presses space again.
    pub fn take(&mut self, seconds: f64) {
        self.press(" ");
        self.wait_for("● REC");
        std::thread::sleep(Duration::from_secs_f64(seconds));
        self.press(" ");
    }

    /// Waits for it to exit; its exit code, and everything it wrote.
    pub fn finish(mut self) -> (u32, String) {
        let deadline = Instant::now() + Duration::from_secs(20);
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "it never exited:\n{}",
                self.screen()
            );
            std::thread::sleep(Duration::from_millis(20));
        };
        drop(self.keys);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        let all = plain(&String::from_utf8_lossy(&self.written.lock().unwrap()));
        (status.exit_code(), all)
    }
}
