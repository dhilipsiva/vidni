//! Playing a take back, through cpal.

use crate::take;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, SupportedStreamConfig};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Where takes are played: an output device of the system's.
pub struct Speaker {
    device: cpal::Device,
    name: String,
}

fn name_of(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_else(|_| "an output with no name".to_string())
}

/// A take that is playing. Dropping it stops the sound.
pub struct Playing {
    done: Arc<AtomicBool>,
    #[allow(dead_code)]
    stream: cpal::Stream,
}

impl Playing {
    /// Whether the take has played to its end.
    pub fn finished(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }
}

impl Speaker {
    /// The output to play through: the one whose name holds `wanted`, or
    /// the system's default output when none is asked for.
    pub fn find(wanted: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = match wanted {
            None => host
                .default_output_device()
                .ok_or("there is nothing to play through: the system has no default output")?,
            Some(wanted) => host
                .output_devices()
                .map_err(|e| format!("the system's outputs can't be listed: {e}"))?
                .find(|device| name_of(device).contains(wanted))
                .ok_or_else(|| format!("no output's name holds {wanted:?}"))?,
        };
        let name = name_of(&device);
        Ok(Self { device, name })
    }

    /// What it is, for people.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Plays the take at `path` from its start. The sound goes on, on a
    /// thread of the output's own, until what is returned is dropped.
    pub fn play(&self, path: &Path) -> Result<Playing, String> {
        let (samples, rate) = take::read(path)?;
        // At the take's own rate when the output offers it; otherwise as
        // the output prefers, with the take stretched to it.
        let config = self
            .device
            .supported_output_configs()
            .ok()
            .and_then(|configs| {
                configs
                    .filter(|c| writable(c.sample_format()))
                    .filter_map(|c| c.try_with_sample_rate(rate))
                    .min_by_key(SupportedStreamConfig::channels)
            })
            .or_else(|| self.device.default_output_config().ok())
            .ok_or_else(|| format!("{} offers no way to play through it", self.name))?;
        let done = Arc::new(AtomicBool::new(false));
        let stream = match config.sample_format() {
            SampleFormat::I16 => self.build::<i16>(&config, samples, rate, done.clone()),
            SampleFormat::F32 => self.build::<f32>(&config, samples, rate, done.clone()),
            SampleFormat::I32 => self.build::<i32>(&config, samples, rate, done.clone()),
            SampleFormat::U16 => self.build::<u16>(&config, samples, rate, done.clone()),
            other => Err(format!(
                "{} takes its samples as {other}, which vidni doesn't write",
                self.name
            )),
        }?;
        stream.play().map_err(|e| format!("{}: {e}", self.name))?;
        Ok(Playing { done, stream })
    }

    /// An output stream of samples of type `T` that plays `samples`,
    /// recorded at `rate`, on every channel, and then says it is done.
    fn build<T>(
        &self,
        config: &SupportedStreamConfig,
        samples: Vec<i16>,
        rate: u32,
        done: Arc<AtomicBool>,
    ) -> Result<cpal::Stream, String>
    where
        T: SizedSample + FromSample<f32>,
    {
        let channels = usize::from(config.channels()).max(1);
        // How far the take moves on for each frame the output asks for.
        let step = f64::from(rate) / f64::from(config.sample_rate().max(1));
        let mut at = 0f64;
        let failed = done.clone();
        self.device
            .build_output_stream(
                &config.config(),
                move |frames: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in frames.chunks_mut(channels) {
                        // Between two samples, a point on the line from
                        // one to the next.
                        let (whole, part) = (at as usize, at.fract() as f32);
                        let value = match (samples.get(whole), samples.get(whole + 1)) {
                            (Some(a), Some(b)) => {
                                let (a, b) = (f32::from(*a), f32::from(*b));
                                (a + (b - a) * part) / 32768.0
                            }
                            (Some(a), None) => f32::from(*a) / 32768.0,
                            _ => {
                                done.store(true, Ordering::Relaxed);
                                0.0
                            }
                        };
                        frame.fill(T::from_sample_(value));
                        at += step;
                    }
                },
                // An output that fails has nothing more to play.
                move |_| failed.store(true, Ordering::Relaxed),
                None,
            )
            .map_err(|e| format!("{}: {e}", self.name))
    }
}

/// Whether vidni writes samples of this format.
fn writable(format: SampleFormat) -> bool {
    matches!(
        format,
        SampleFormat::I16 | SampleFormat::F32 | SampleFormat::I32 | SampleFormat::U16
    )
}
