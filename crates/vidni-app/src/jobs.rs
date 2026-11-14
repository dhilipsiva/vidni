//! Jobs: a `vidni` command run on a thread of its own, with what it says
//! handed over line by line.

use std::sync::mpsc::{self, Receiver};
use vidni_cli::console::{self, Stream};

enum Event {
    Said(Stream, String),
    Done(u8),
}

/// One command, running or finished.
pub struct Job {
    /// The command's arguments, as they follow `vidni` on a command line.
    pub args: Vec<String>,
    /// What it has said so far.
    pub lines: Vec<(Stream, String)>,
    /// How far its render is: frames done, and frames in all.
    pub progress: Option<(u64, u64)>,
    /// Its exit code, once it has finished.
    pub code: Option<u8>,
    events: Receiver<Event>,
}

/// `done of all` out of a line such as `long: long.mp4: 203 of 2021 frames`.
fn frames(line: &str) -> Option<(u64, u64)> {
    let counted = line.strip_suffix(" frames")?;
    let mut words = counted.rsplit(' ');
    let (all, of, done) = (words.next()?, words.next()?, words.next()?);
    (of == "of").then_some(())?;
    Some((done.parse().ok()?, all.parse().ok()?))
}

impl Job {
    /// Starts `vidni <args>` on a new thread. `wake` is called whenever
    /// the job has something new to show.
    pub fn start(args: Vec<String>, wake: impl Fn() + Send + 'static) -> Self {
        let (tx, events) = mpsc::channel();
        let command = args.clone();
        std::thread::spawn(move || {
            let (said, waker) = (tx.clone(), std::sync::Arc::new(wake));
            let on_line = waker.clone();
            let listening = console::listen(move |stream, line| {
                let _ = said.send(Event::Said(stream, line.to_string()));
                on_line();
            });
            let code = vidni_cli::run(&command);
            drop(listening);
            let _ = tx.send(Event::Done(code));
            waker();
        });
        Self {
            args,
            lines: Vec::new(),
            progress: None,
            code: None,
            events,
        }
    }

    /// Takes in what the job has said since the last call. True when
    /// there was anything.
    pub fn poll(&mut self) -> bool {
        let mut any = false;
        while let Ok(event) = self.events.try_recv() {
            any = true;
            self.take(event);
        }
        any
    }

    /// Waits for the job to finish, taking in everything it says.
    pub fn wait(&mut self) -> u8 {
        while self.code.is_none() {
            match self.events.recv() {
                Ok(event) => self.take(event),
                // The thread is gone without a word: the command panicked.
                Err(_) => self.code = Some(1),
            }
        }
        self.code.unwrap_or(1)
    }

    fn take(&mut self, event: Event) {
        match event {
            Event::Said(stream, line) => {
                if let Some(progress) = frames(&line) {
                    self.progress = Some(progress);
                    // A render says how far it is ten times; the log keeps the last.
                    if self
                        .lines
                        .last()
                        .is_some_and(|(_, last)| frames(last).is_some())
                    {
                        self.lines.pop();
                    }
                }
                self.lines.push((stream, line));
            }
            Event::Done(code) => {
                self.code = Some(code);
                self.progress = None;
            }
        }
    }

    /// Whether the job is still running.
    pub fn running(&self) -> bool {
        self.code.is_none()
    }

    /// The command as it is typed.
    pub fn line(&self) -> String {
        crate::actions::shell(&self.args)
    }
}

/// Runs `vidni <args>` here and now, for a command that answers at once.
/// Returns its exit code and what it said.
pub fn now<S: AsRef<str>>(args: &[S]) -> (u8, Vec<(Stream, String)>) {
    let said = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let sink = said.clone();
    let listening = console::listen(move |stream, line| {
        sink.borrow_mut().push((stream, line.to_string()));
    });
    let code = vidni_cli::run(args);
    drop(listening);
    let lines = said.borrow().clone();
    (code, lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_renders_progress_is_read_off_what_it_says() {
        assert_eq!(
            frames("long: long.mp4: 203 of 2021 frames"),
            Some((203, 2021))
        );
        assert_eq!(frames("long: narration"), None);
        assert_eq!(frames("x.mp4: 2021 frames; 1920x1080 h264"), None);
        assert_eq!(frames("3 of them frames"), None);
    }

    #[test]
    fn a_job_hands_over_what_its_command_says_and_its_code() {
        let woken = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = woken.clone();
        let mut job = Job::start(vec!["platforms".into()], move || {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        assert_eq!(job.line(), "vidni platforms");
        assert_eq!(job.wait(), 0);
        assert!(!job.running());
        assert!(
            job.lines[0].1.starts_with("youtube (YouTube"),
            "{:?}",
            job.lines[0]
        );
        assert!(job.lines.iter().all(|(stream, _)| *stream == Stream::Out));
        // The window is woken for each line, and once more for the end, which
        // the job's thread does after it has handed the code over.
        let began = std::time::Instant::now();
        while woken.load(std::sync::atomic::Ordering::SeqCst) <= job.lines.len() {
            assert!(
                began.elapsed() < std::time::Duration::from_secs(10),
                "the end of the job woke nobody"
            );
            std::thread::yield_now();
        }
        // A command that fails says why on the other stream, with its code.
        let (code, said) = now(&["status", "/nowhere/at/all"]);
        assert_eq!(code, 2);
        assert_eq!(said.len(), 1);
        assert_eq!(said[0].0, Stream::Err);
    }
}
