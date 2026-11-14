//! Where a command's words go: stdout and stderr, or, when the command is
//! run inside another program, to that program's listener.

use std::cell::RefCell;

/// Which of the two streams a line is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// What the command was asked for.
    Out,
    /// How it is going, and what went wrong.
    Err,
}

type Listener = Box<dyn FnMut(Stream, &str)>;

thread_local! {
    static LISTENER: RefCell<Option<Listener>> = const { RefCell::new(None) };
}

/// Sends what commands say on this thread to `listener` until the guard
/// that is returned is dropped. A command run by [`crate::run`] on the
/// thread then prints nothing.
pub fn listen(listener: impl FnMut(Stream, &str) + 'static) -> Listening {
    LISTENER.with(|slot| *slot.borrow_mut() = Some(Box::new(listener)));
    Listening(())
}

/// The guard of [`listen`]: while it lives, the thread's listener hears
/// every line.
pub struct Listening(());

impl Drop for Listening {
    fn drop(&mut self) {
        LISTENER.with(|slot| *slot.borrow_mut() = None);
    }
}

/// Says `text`, a line, on `stream`.
pub fn line(stream: Stream, text: &str) {
    let heard = LISTENER.with(|slot| match slot.borrow_mut().as_mut() {
        Some(listener) => {
            listener(stream, text);
            true
        }
        None => false,
    });
    if !heard {
        use std::io::Write as _;
        // A reader that has gone away isn't an error.
        let _ = match stream {
            Stream::Out => writeln!(std::io::stdout(), "{text}"),
            Stream::Err => writeln!(std::io::stderr(), "{text}"),
        };
    }
}
