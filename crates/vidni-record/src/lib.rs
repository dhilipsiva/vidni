//! vidni's recorder (TODO.md, milestone P4): a take is captured from an
//! input, kept raw as it was captured, checked, and trimmed into the take a
//! cut is built from.
//!
//! - [`input`]: where the sound comes from. [`mic`] is the microphone,
//!   through cpal; [`input::Synthetic`] is a made-up signal, for tests.
//! - [`take`]: a recording into a raw file, with its level meter, and the
//!   candidate that is trimmed from it and kept or discarded.
//! - [`clean`]: the guard against a take with nothing in it, and the trim.
//! - [`room`]: room tone, recorded during the mic check.
//! - [`play`]: playing a take back.

pub mod clean;
pub mod input;
#[cfg(feature = "mic")]
pub mod mic;
#[cfg(feature = "mic")]
pub mod play;
pub mod room;
pub mod take;
