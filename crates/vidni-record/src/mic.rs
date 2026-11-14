//! The microphone, through cpal.

use crate::input::{Failed, Input, Sink, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, SupportedStreamConfig};

/// The rate vidni asks a microphone for: the takes so far were recorded at
/// it, and it is the rate the cuts are encoded at.
pub const RATE: u32 = 48000;

/// A microphone: an input device of the system's.
pub struct Mic {
    device: cpal::Device,
    config: SupportedStreamConfig,
    name: String,
}

fn name_of(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_else(|_| "an input with no name".to_string())
}

impl Mic {
    /// The names of the system's input devices, each once.
    pub fn all() -> Result<Vec<String>, String> {
        let devices = cpal::default_host()
            .input_devices()
            .map_err(|e| format!("the system's inputs can't be listed: {e}"))?;
        let mut names: Vec<String> = Vec::new();
        for name in devices.map(|device| name_of(&device)) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        Ok(names)
    }

    /// The microphone to record from: the input whose name holds `wanted`,
    /// or the system's default input when none is asked for.
    pub fn find(wanted: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = match wanted {
            None => host.default_input_device().ok_or(
                "there is no microphone: the system has no default input; \
                 VIDNI_MIC names another",
            )?,
            Some(wanted) => host
                .input_devices()
                .map_err(|e| format!("the system's inputs can't be listed: {e}"))?
                .find(|device| name_of(device).contains(wanted))
                .ok_or_else(|| format!("no input's name holds {wanted:?}"))?,
        };
        let name = name_of(&device);
        // At 48 kHz when the device can, with as few channels as it has;
        // otherwise as the device prefers.
        let config = device
            .supported_input_configs()
            .ok()
            .and_then(|configs| {
                configs
                    .filter(|c| readable(c.sample_format()))
                    .filter_map(|c| c.try_with_sample_rate(RATE))
                    .min_by_key(SupportedStreamConfig::channels)
            })
            .or_else(|| device.default_input_config().ok())
            .ok_or_else(|| format!("{name} offers no way to record from it"))?;
        if !readable(config.sample_format()) {
            return Err(format!(
                "{name} gives its samples as {}, which vidni doesn't read",
                config.sample_format()
            ));
        }
        Ok(Self {
            device,
            config,
            name,
        })
    }

    /// Starts the device with samples of type `T`, mixed down to one
    /// channel: the mean of each frame's.
    fn build<T>(&self, mut sink: Sink, mut failed: Failed) -> Result<cpal::Stream, String>
    where
        T: SizedSample,
        i16: FromSample<T>,
    {
        let channels = usize::from(self.config.channels()).max(1);
        let mut mono = Vec::new();
        self.device
            .build_input_stream(
                &self.config.config(),
                move |frames: &[T], _: &cpal::InputCallbackInfo| {
                    mono.clear();
                    mono.extend(frames.chunks_exact(channels).map(|frame| {
                        let sum: i32 = frame
                            .iter()
                            .map(|sample| i32::from(<i16 as FromSample<T>>::from_sample_(*sample)))
                            .sum();
                        (sum / channels as i32) as i16
                    }));
                    sink(&mono);
                },
                move |error| failed(error.to_string()),
                None,
            )
            .map_err(|e| format!("{}: {e}", self.name))
    }
}

/// Whether vidni reads samples of this format.
fn readable(format: SampleFormat) -> bool {
    matches!(
        format,
        SampleFormat::I16 | SampleFormat::F32 | SampleFormat::I32 | SampleFormat::U16
    )
}

impl Input for Mic {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> u32 {
        self.config.sample_rate()
    }

    fn start(&self, sink: Sink, failed: Failed) -> Result<Box<dyn Stream>, String> {
        let stream = match self.config.sample_format() {
            SampleFormat::I16 => self.build::<i16>(sink, failed),
            SampleFormat::F32 => self.build::<f32>(sink, failed),
            SampleFormat::I32 => self.build::<i32>(sink, failed),
            SampleFormat::U16 => self.build::<u16>(sink, failed),
            other => Err(format!(
                "{} gives its samples as {other}, which vidni doesn't read",
                self.name
            )),
        }?;
        stream.play().map_err(|e| format!("{}: {e}", self.name))?;
        Ok(Box::new(Running(stream)))
    }
}

/// The device's stream, which stops when it is dropped.
struct Running(#[allow(dead_code)] cpal::Stream);

impl Stream for Running {}
