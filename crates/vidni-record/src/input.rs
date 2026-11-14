//! Where a take's sound comes from: the microphone ([`crate::mic`]), or a
//! signal made up for tests ([`Synthetic`]). The recorder knows only
//! [`Input`], so a test drives it exactly as a microphone does.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Where an input's samples go: each block as it arrives, in order.
pub type Sink = Box<dyn FnMut(&[i16]) + Send>;
/// Where an input says that it has stopped on its own, and why.
pub type Failed = Box<dyn FnMut(String) + Send>;

/// A source of sound with one channel.
pub trait Input {
    /// What it is, for people.
    fn name(&self) -> String;
    /// Its samples a second.
    fn rate(&self) -> u32;
    /// Starts it. Each block of samples goes to `sink` as it arrives, on a
    /// thread of the input's own, until the stream that is returned is
    /// dropped. `failed` is told when the input stops on its own.
    fn start(&self, sink: Sink, failed: Failed) -> Result<Box<dyn Stream>, String>;
}

/// An input that is running. Dropping it stops the input: no sample
/// reaches its sink after that.
pub trait Stream {}

/// A made-up input: a signal worked out sample by sample, handed over in
/// blocks of a hundredth of a second, as a sound card hands them over.
#[derive(Clone)]
pub struct Synthetic {
    rate: u32,
    signal: Arc<dyn Fn(u64) -> i16 + Send + Sync>,
    /// How many samples it gives before it gives no more.
    length: Option<u64>,
    /// What it fails with once it has given them all.
    fails: Option<String>,
    paced: bool,
}

impl Synthetic {
    /// An input whose sample number `n` is `signal(n)`, at `rate`.
    pub fn new(rate: u32, signal: impl Fn(u64) -> i16 + Send + Sync + 'static) -> Self {
        Self {
            rate,
            signal: Arc::new(signal),
            length: None,
            fails: None,
            paced: false,
        }
    }

    /// A room with no sound in it at all.
    pub fn silence(rate: u32) -> Self {
        Self::new(rate, |_| 0)
    }

    /// A tone of `hz` at `level` of full scale.
    pub fn tone(rate: u32, hz: f64, level: f64) -> Self {
        Self::new(rate, move |n| {
            let t = n as f64 / f64::from(rate);
            ((t * hz * std::f64::consts::TAU).sin() * level * 32767.0) as i16
        })
    }

    /// `samples`, and silence after them.
    pub fn samples(rate: u32, samples: Vec<i16>) -> Self {
        Self::new(rate, move |n| {
            usize::try_from(n)
                .ok()
                .and_then(|n| samples.get(n))
                .copied()
                .unwrap_or(0)
        })
    }

    /// The same input, giving `seconds` of its signal and then no more: a
    /// test knows how much it will record.
    pub fn lasting(mut self, seconds: f64) -> Self {
        self.length = Some((seconds * f64::from(self.rate)).round() as u64);
        self
    }

    /// The same input, failing with `why` once it has given what it lasts
    /// for, as a microphone that is unplugged does.
    pub fn failing(mut self, why: &str) -> Self {
        self.fails = Some(why.to_string());
        self
    }

    /// The same input, at the pace of a real one: a second of sound takes
    /// a second. Without this the samples come as fast as they are made.
    pub fn paced(mut self) -> Self {
        self.paced = true;
        self
    }
}

impl Input for Synthetic {
    fn name(&self) -> String {
        "a made-up input".to_string()
    }

    fn rate(&self) -> u32 {
        self.rate
    }

    fn start(&self, mut sink: Sink, mut failed: Failed) -> Result<Box<dyn Stream>, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let (stopped, mut input) = (stop.clone(), self.clone());
        let thread = thread::spawn(move || {
            let block = u64::from(input.rate / 100).max(1);
            let started = Instant::now();
            let mut given = 0u64;
            let mut samples = Vec::with_capacity(block as usize);
            while !stopped.load(Ordering::Relaxed) {
                let left = input.length.map_or(block, |length| length - given);
                if left == 0 {
                    if let Some(why) = input.fails.take() {
                        failed(why);
                    }
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                samples.clear();
                samples.extend((given..given + left.min(block)).map(|n| (input.signal)(n)));
                given += samples.len() as u64;
                sink(&samples);
                if input.paced {
                    let due = Duration::from_secs_f64(given as f64 / f64::from(input.rate));
                    thread::sleep(due.saturating_sub(started.elapsed()));
                }
            }
        });
        Ok(Box::new(Running {
            stop,
            thread: Some(thread),
        }))
    }
}

struct Running {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Stream for Running {}

impl Drop for Running {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
