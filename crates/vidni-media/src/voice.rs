//! The voice chain (TODO.md, P4-VIDNI-03): what every line's clip goes
//! through before the narration is brought to its loudness, and the room
//! tone that fills the pauses.
//!
//! Each clip:
//! 1. **High-pass,** 80 Hz: rumble below the voice.
//! 2. **De-click:** a sample or two that stand far out of their
//!    neighbourhood, such as a mouth click, are drawn over.
//! 3. **De-essing:** the band where "s" hisses is turned down while it is loud.
//!
//! Then the clips are laid on the timeline, and the whole narration:
//! 4. **Level:** one gain brings it to the target loudness.
//! 5. **Compression:** syllables that stand above that loudness are
//!    brought nearer it, and the gain is set again for what that took.
//! 6. **Limit:** the few peaks still above the ceiling are held under it.
//! 7. **Room tone:** every stretch of digital silence is filled with the
//!    room the takes were recorded in, when the mic check kept it, or
//!    else with a faint noise made from a fixed seed.
//!
//! All of it is arithmetic on samples, in a fixed order, with the
//! transcendental functions of the `libm` crate, which are the same on
//! every machine: the same clips give the same narration, byte for byte (T1).

use std::path::Path;
use vidni_core::timeline::Timeline;

/// Counts the changes to what the chain does to a sample. A change makes
/// every earlier build's record out of date.
pub const VERSION: u32 = 1;

/// The high-pass corner, in Hz.
const HIGH_PASS: f64 = 80.0;
/// A click stands this many times above the roughness around it.
const CLICK: f64 = 10.0;
/// The longest run drawn over as a click, in seconds.
const CLICK_RUN: f64 = 0.002;
/// The compressor: its threshold in dB above the target loudness, its
/// ratio, its knee in dB, and its attack and release in seconds.
const THRESHOLD: f64 = 4.0;
const RATIO: f64 = 2.5;
const KNEE: f64 = 6.0;
const ATTACK: f64 = 0.008;
const RELEASE: f64 = 0.15;
/// The de-esser: its band's centre in Hz, the band's threshold in dBFS,
/// its ratio, and its attack and release in seconds.
const ESS: f64 = 6500.0;
const ESS_THRESHOLD: f64 = -32.0;
const ESS_RATIO: f64 = 3.0;
const ESS_ATTACK: f64 = 0.001;
const ESS_RELEASE: f64 = 0.06;
/// A run of zeros at least this long is a pause to fill, in seconds, and
/// the tone fades in and out of it over as long.
const PAUSE: f64 = 0.005;
/// The ceiling no sample goes above, in dBFS, and how far ahead the
/// limiter looks, in seconds. The true peak, between samples and after
/// encoding, is a little above the highest sample.
const CEILING: f64 = -2.5;
const AHEAD: f64 = 0.005;
/// The level of the noise that stands in for a room, in dBFS (RMS).
const FLOOR: f64 = -70.0;

/// A second-order filter, in the direct form.
struct Biquad {
    b: [f64; 3],
    a: [f64; 2],
    x: [f64; 2],
    y: [f64; 2],
}

impl Biquad {
    fn new(b: [f64; 3], a: [f64; 3]) -> Self {
        Self {
            b: [b[0] / a[0], b[1] / a[0], b[2] / a[0]],
            a: [a[1] / a[0], a[2] / a[0]],
            x: [0.0; 2],
            y: [0.0; 2],
        }
    }

    /// A Butterworth high-pass at `hz`.
    fn high_pass(hz: f64, rate: f64) -> Self {
        let w = std::f64::consts::TAU * hz / rate;
        let (sin, cos) = (libm::sin(w), libm::cos(w));
        let alpha = sin / (2.0 * std::f64::consts::FRAC_1_SQRT_2);
        Self::new(
            [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    /// A band-pass around `hz`, with a peak gain of one.
    fn band_pass(hz: f64, q: f64, rate: f64) -> Self {
        let w = std::f64::consts::TAU * hz / rate;
        let (sin, cos) = (libm::sin(w), libm::cos(w));
        let alpha = sin / (2.0 * q);
        Self::new([alpha, 0.0, -alpha], [1.0 + alpha, -2.0 * cos, 1.0 - alpha])
    }

    fn step(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

/// Follows a signal's level: up over `attack` seconds, down over `release`.
struct Envelope {
    up: f64,
    down: f64,
    level: f64,
}

impl Envelope {
    fn new(attack: f64, release: f64, rate: f64) -> Self {
        Self {
            up: 1.0 - libm::exp(-1.0 / (attack * rate)),
            down: 1.0 - libm::exp(-1.0 / (release * rate)),
            level: 0.0,
        }
    }

    fn step(&mut self, x: f64) -> f64 {
        let x = x.abs();
        let rate = if x > self.level { self.up } else { self.down };
        self.level += rate * (x - self.level);
        self.level
    }
}

/// The gain, as a factor, that a compressor gives a level: none below the
/// threshold, `ratio` to one above it, and a knee of `knee` dB between.
fn reduction(level: f64, threshold: f64, ratio: f64, knee: f64) -> f64 {
    if level <= 0.0 {
        return 1.0;
    }
    let over = 20.0 * libm::log10(level) - threshold;
    let slope = 1.0 / ratio - 1.0;
    let db = if 2.0 * over <= -knee {
        return 1.0;
    } else if 2.0 * over < knee {
        slope * (over + knee / 2.0) * (over + knee / 2.0) / (2.0 * knee)
    } else {
        slope * over
    };
    libm::pow(10.0, db / 20.0)
}

fn high_pass(x: &mut [f64], rate: f64) {
    let mut filter = Biquad::high_pass(HIGH_PASS, rate);
    for s in x.iter_mut() {
        *s = filter.step(*s);
    }
}

/// Draws a straight line over each short run of samples whose roughness
/// (how far a sample is from the line through its two neighbours) stands
/// [`CLICK`] times above the roughness of the 20 ms around it.
fn declick(x: &mut [f64], rate: f64) {
    let n = x.len();
    if n < 8 {
        return;
    }
    let rough: Vec<f64> = (0..n)
        .map(|i| match i {
            0 => 0.0,
            i if i + 1 == n => 0.0,
            i => (x[i] - (x[i - 1] + x[i + 1]) / 2.0).abs(),
        })
        .collect();
    let half = ((0.01 * rate) as usize).max(4);
    // The roughness summed up to each sample, to take a window's sum from.
    let mut before = Vec::with_capacity(n + 1);
    let mut sum = 0.0;
    before.push(sum);
    for r in &rough {
        sum += r;
        before.push(sum);
    }
    let longest = ((CLICK_RUN * rate) as usize).max(2);
    let mut i = 1;
    while i + 1 < n {
        // The roughness around the sample, without its own.
        let (from, to) = (i.saturating_sub(half), (i + half).min(n - 1));
        let around = (before[to + 1] - before[from] - rough[i]).max(0.0) / (to - from) as f64;
        // Quieter than one step of a 16-bit sample is nothing to mend.
        if rough[i] <= CLICK * around || rough[i] < 1.0 / 32768.0 {
            i += 1;
            continue;
        }
        // The run goes on while the roughness stays high, with a sample of margin.
        let mut end = i;
        while end + 1 < n - 1 && end - i < longest && rough[end + 1] > CLICK * around / 2.0 {
            end += 1;
        }
        if end - i < longest {
            let (a, b) = (i - 1, (end + 1).min(n - 1));
            for j in i..=end {
                let t = (j - a) as f64 / (b - a) as f64;
                x[j] = x[a] + (x[b] - x[a]) * t;
            }
        }
        i = end + 2;
    }
}

/// Compresses what is louder than `threshold` dBFS.
fn compress(x: &mut [f64], rate: f64, threshold: f64) {
    let mut envelope = Envelope::new(ATTACK, RELEASE, rate);
    for s in x.iter_mut() {
        *s *= reduction(envelope.step(*s), threshold, RATIO, KNEE);
    }
}

/// Holds every sample at or under `ceiling` (a factor of full scale). The
/// gain comes down over [`AHEAD`] before a peak and goes back up over as
/// long after it, so nothing is clipped and nothing clicks.
fn limit(x: &mut [f64], rate: f64, ceiling: f64) {
    let n = x.len();
    let ahead = ((AHEAD * rate) as usize).max(1);
    // The gain each sample needs by itself.
    let needed: Vec<f64> = x
        .iter()
        .map(|s| {
            if s.abs() > ceiling {
                ceiling / s.abs()
            } else {
                1.0
            }
        })
        .collect();
    if needed.iter().all(|g| *g == 1.0) {
        return;
    }
    // The least gain needed from each sample to `ahead` samples on...
    let mut least = vec![1.0; n];
    let mut window: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for i in (0..n).rev() {
        while window.back().is_some_and(|j| needed[*j] >= needed[i]) {
            window.pop_back();
        }
        window.push_back(i);
        while window.front().is_some_and(|j| *j > i + ahead) {
            window.pop_front();
        }
        least[i] = needed[*window.front().expect("the sample itself")];
    }
    // ...and its mean over the `ahead` samples up to each one: every value
    // in that mean already allows for this sample, so the mean does too.
    let mut sum = 0.0;
    for i in 0..n {
        sum += least[i];
        if i > ahead {
            sum -= least[i - ahead - 1];
        }
        let count = (i + 1).min(ahead + 1);
        x[i] *= (sum / count as f64).min(needed[i]);
    }
}

/// Turns the band around [`ESS`] down while it is loud. A clip sampled too
/// slowly to hold that band is left as it is.
fn deess(x: &mut [f64], rate: f64) {
    if ESS >= 0.45 * rate {
        return;
    }
    let mut band = Biquad::band_pass(ESS, 1.2, rate);
    let mut envelope = Envelope::new(ESS_ATTACK, ESS_RELEASE, rate);
    for s in x.iter_mut() {
        let hiss = band.step(*s);
        let keep = reduction(envelope.step(hiss), ESS_THRESHOLD, ESS_RATIO, 0.0);
        *s -= (1.0 - keep) * hiss;
    }
}

/// One clip through the chain.
pub fn clean(samples: &[i16], rate: u32) -> Vec<i16> {
    let rate = f64::from(rate);
    let mut x: Vec<f64> = samples.iter().map(|s| f64::from(*s) / 32768.0).collect();
    high_pass(&mut x, rate);
    declick(&mut x, rate);
    deess(&mut x, rate);
    quantized(&x)
}

/// A faint noise to stand in for a room: `len` samples at [`FLOOR`], the
/// same every time. It is white noise from a fixed seed, with its treble
/// taken off, as a quiet room's is.
pub fn floor(len: usize, rate: u32) -> Vec<i16> {
    // xorshift64*, from a fixed seed.
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        // Its top 53 bits, as a number from -1 to 1.
        (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 52) as f64 - 1.0
    };
    // One pole at 1 kHz.
    let pole = libm::exp(-std::f64::consts::TAU * 1000.0 / f64::from(rate));
    let mut y = 0.0;
    let noise: Vec<f64> = (0..len)
        .map(|_| {
            y = pole * y + (1.0 - pole) * next();
            y
        })
        .collect();
    let rms = (noise.iter().map(|s| s * s).sum::<f64>() / len.max(1) as f64).sqrt();
    let scale = libm::pow(10.0, FLOOR / 20.0) / rms.max(f64::MIN_POSITIVE);
    noise
        .iter()
        .map(|s| {
            let s = (s * scale * 32768.0).round();
            // A sample of noise is never nothing: a zero would be the
            // silence the noise is there to replace.
            if s == 0.0 { 1 } else { s as i16 }
        })
        .collect()
}

/// `tone` as long as `len`, repeated, each repeat fading into the next
/// over a tenth of a second.
fn looped(tone: &[i16], len: usize, rate: u32) -> Vec<i16> {
    let fade = ((0.1 * f64::from(rate)) as usize).min(tone.len() / 4);
    let step = tone.len() - fade;
    (0..len)
        .map(|i| {
            let at = i % step;
            if i >= step && at < fade {
                // The end of the repeat before, fading out under this one's start.
                let t = at as f64 / fade as f64;
                let (old, new) = (f64::from(tone[step + at]), f64::from(tone[at]));
                (old * (1.0 - t) + new * t).round() as i16
            } else {
                tone[at]
            }
        })
        .collect()
}

/// Fills every run of zeros in `track` that lasts [`PAUSE`] or longer with
/// `tone`, fading in and out at the run's ends.
pub fn fill(track: &mut [i16], tone: &[i16], rate: u32) {
    if tone.len() < 8 {
        return;
    }
    let bed = looped(tone, track.len(), rate);
    let ramp = ((PAUSE * f64::from(rate)) as usize).max(1);
    let mut i = 0;
    while i < track.len() {
        if track[i] != 0 {
            i += 1;
            continue;
        }
        let start = i;
        while i < track.len() && track[i] == 0 {
            i += 1;
        }
        if i - start < ramp {
            continue;
        }
        for j in start..i {
            // The distance to the nearer end of the run, counted from one,
            // so that the first sample of the tone is already sound.
            let edge = (j - start + 1).min(i - j);
            let weight = (edge as f64 / ramp as f64).min(1.0);
            let s = (f64::from(bed[j]) * weight).round() as i16;
            // A filled sample is never nothing, however faint the tone.
            track[j] = if s == 0 { 1 } else { s };
        }
    }
}

/// The loudness of `track` played on both channels of a stereo file, in
/// LUFS: integrated loudness as ITU-R BS.1770 and EBU R 128 define it,
/// with their K weighting and their two gates. `None` for a track with
/// nothing above the gates in it.
pub fn loudness(track: &[i16], rate: u32) -> Option<f64> {
    let fs = f64::from(rate);
    // The K weighting: a shelf that lifts the treble, as a head does, and
    // a high-pass. The constants are the standard's filters, restated for
    // any sample rate.
    let shelf = {
        let k = libm::tan(std::f64::consts::PI * 1681.974450955533 / fs);
        let q = 0.7071752369554196;
        let vh = libm::pow(10.0, 3.999843853973347 / 20.0);
        let vb = libm::pow(vh, 0.4996667741545416);
        Biquad::new(
            [
                vh + vb * k / q + k * k,
                2.0 * (k * k - vh),
                vh - vb * k / q + k * k,
            ],
            [
                1.0 + k / q + k * k,
                2.0 * (k * k - 1.0),
                1.0 - k / q + k * k,
            ],
        )
    };
    let high = {
        let k = libm::tan(std::f64::consts::PI * 38.13547087602444 / fs);
        let q = 0.5003270373238773;
        Biquad::new(
            [1.0, -2.0, 1.0],
            [
                1.0 + k / q + k * k,
                2.0 * (k * k - 1.0),
                1.0 - k / q + k * k,
            ],
        )
    };
    let (mut shelf, mut high) = (shelf, high);
    let squares: Vec<f64> = track
        .iter()
        .map(|s| {
            let y = high.step(shelf.step(f64::from(*s) / 32768.0));
            y * y
        })
        .collect();
    // Blocks of 400 ms, a new one every 100 ms. The same sound on two
    // channels has twice the power of one.
    let (block, step) = ((0.4 * fs) as usize, (0.1 * fs) as usize);
    if squares.len() < block || step == 0 {
        return None;
    }
    let powers: Vec<f64> = (0..=(squares.len() - block) / step)
        .map(|j| 2.0 * squares[j * step..j * step + block].iter().sum::<f64>() / block as f64)
        .collect();
    let lufs = |power: f64| -0.691 + 10.0 * libm::log10(power);
    let mean = |powers: &[f64]| powers.iter().sum::<f64>() / powers.len() as f64;
    // The first gate drops what is under -70 LUFS; the second, what is
    // more than 10 LU under the loudness of the rest.
    let heard: Vec<f64> = powers
        .iter()
        .copied()
        .filter(|p| lufs(*p) > -70.0)
        .collect();
    if heard.is_empty() {
        return None;
    }
    let gate = lufs(mean(&heard)) - 10.0;
    let kept: Vec<f64> = heard.into_iter().filter(|p| lufs(*p) > gate).collect();
    (!kept.is_empty()).then(|| lufs(mean(&kept)))
}

fn quantized(x: &[f64]) -> Vec<i16> {
    x.iter()
        .map(|s| (s * 32768.0).round().clamp(-32768.0, 32767.0) as i16)
        .collect()
}

/// Brings `track` to `target` LUFS: levelled, compressed, levelled again
/// and limited. Returns the gain it was given in all, as a factor, for
/// the room tone to be given too; or `None` for a track with no sound.
pub fn master(track: &mut [i16], rate: u32, target: f64) -> Option<f64> {
    let fs = f64::from(rate);
    let factor = |db: f64| libm::pow(10.0, db / 20.0);
    let first = factor(target - loudness(track, rate)?);
    let mut x: Vec<f64> = track
        .iter()
        .map(|s| f64::from(*s) / 32768.0 * first)
        .collect();
    compress(&mut x, fs, target + THRESHOLD);
    // What compression took from the loudness is given back. The measure
    // is taken of the samples as they would be written.
    let second = factor(target - loudness(&quantized(&x), rate)?);
    for s in x.iter_mut() {
        *s *= second;
    }
    limit(&mut x, fs, factor(CEILING));
    // Holding peaks takes a little loudness too. It is given back, and
    // the peaks held again, until the narration is within a tenth of an
    // LU of its target, or three times at most.
    let mut gain = first * second;
    for _ in 0..3 {
        let short = target - loudness(&quantized(&x), rate)?;
        if short.abs() <= 0.1 {
            break;
        }
        let more = factor(short);
        for s in x.iter_mut() {
            *s *= more;
        }
        limit(&mut x, fs, factor(CEILING));
        gain *= more;
    }
    track.copy_from_slice(&quantized(&x));
    Some(gain)
}

/// The samples of the 16-bit mono WAV at `path`, and its rate.
pub fn samples(path: &Path) -> Result<(Vec<i16>, u32), String> {
    read(path, None)
}

fn read(clip: &Path, rate: Option<u32>) -> Result<(Vec<i16>, u32), String> {
    let mut reader =
        hound::WavReader::open(clip).map_err(|e| format!("{}: {e}", clip.display()))?;
    let spec = reader.spec();
    if spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(format!(
            "{}: the narration needs 16-bit mono clips",
            clip.display()
        ));
    }
    if let Some(rate) = rate
        && spec.sample_rate != rate
    {
        return Err(format!(
            "{}: sample rate {} differs from {rate}",
            clip.display(),
            spec.sample_rate
        ));
    }
    let data = reader
        .samples::<i16>()
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", clip.display()))?;
    Ok((data, spec.sample_rate))
}

/// The room tone to fill a narration at `rate` with: the samples of
/// `room`, when it is a recording at that rate with something in it.
pub fn room(room: &Path, rate: u32) -> Option<Vec<i16>> {
    let (tone, _) = read(room, Some(rate)).ok()?;
    (tone.len() as f64 >= 0.5 * f64::from(rate) && tone.iter().any(|s| *s != 0)).then_some(tone)
}

/// Music to play under a narration (TODO.md, P6-VIDNI-06).
#[derive(Debug, Clone, PartialEq)]
pub struct Bed {
    /// The track, as one channel at the narration's rate.
    pub samples: Vec<i16>,
    /// How far under the narration it plays, in LU.
    pub under: f64,
}

/// How long music takes to come in at the start of a cut and to go out at
/// its end, in seconds.
const MUSIC_IN: f64 = 1.0;
const MUSIC_OUT: f64 = 2.0;

/// The 16-bit WAV at `path` as one channel at `rate`: its channels
/// averaged, and its samples taken along straight lines between its own
/// when its rate is another.
pub fn track(path: &Path, rate: u32) -> Result<Vec<i16>, String> {
    let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let mut reader = hound::WavReader::open(path).map_err(|e| shown(&e))?;
    let spec = reader.spec();
    if spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
        || !(1..=2).contains(&spec.channels)
    {
        return Err(shown(&"music is a 16-bit WAV with one or two channels"));
    }
    let all: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<_, _>>()
        .map_err(|e| shown(&e))?;
    let mono: Vec<f64> = all
        .chunks(usize::from(spec.channels))
        .map(|frame| frame.iter().map(|s| f64::from(*s)).sum::<f64>() / frame.len() as f64)
        .collect();
    if mono.len() < 2 || spec.sample_rate == 0 {
        return Err(shown(&"there is no music in it"));
    }
    if spec.sample_rate == rate {
        return Ok(mono.iter().map(|s| s.round() as i16).collect());
    }
    let step = f64::from(spec.sample_rate) / f64::from(rate);
    let len = ((mono.len() - 1) as f64 / step) as usize + 1;
    Ok((0..len)
        .map(|i| {
            let at = i as f64 * step;
            let (before, part) = (at as usize, at.fract());
            let after = (before + 1).min(mono.len() - 1);
            (mono[before] * (1.0 - part) + mono[after] * part).round() as i16
        })
        .collect())
}

/// Plays `bed` under `narration`, which is at `target` LUFS: repeated to
/// the narration's length, `bed.under` LU quieter than it, coming in over
/// [`MUSIC_IN`] and going out over [`MUSIC_OUT`].
fn underlay(narration: &mut [i16], bed: &Bed, rate: u32, target: f64) {
    if bed.samples.len() < 8 {
        return;
    }
    let music = looped(&bed.samples, narration.len(), rate);
    let Some(now) = loudness(&music, rate) else {
        return;
    };
    let gain = libm::pow(10.0, (target - bed.under - now) / 20.0);
    let (fade_in, fade_out) = (MUSIC_IN * f64::from(rate), MUSIC_OUT * f64::from(rate));
    let len = narration.len();
    for (i, (s, m)) in narration.iter_mut().zip(&music).enumerate() {
        let fade = (i as f64 / fade_in)
            .min((len - i) as f64 / fade_out)
            .min(1.0);
        let mixed = f64::from(*s) + f64::from(*m) * gain * fade;
        *s = mixed.round().clamp(-32768.0, 32767.0) as i16;
    }
}

/// Writes the narration of `timeline` to `out`, at `target` LUFS: each
/// line's clip from `clips`, through the chain, at its start on the
/// timeline, with the pauses filled by `tone`, or by the stand-in noise
/// when there is none; and `music` under it all, when there is any.
pub fn narrate(
    timeline: &Timeline,
    clips: &[&Path],
    tone: Option<&[i16]>,
    music: Option<&Bed>,
    target: f64,
    out: &Path,
) -> Result<(), String> {
    if clips.len() != timeline.sentences.len() {
        return Err(format!(
            "{} clips for {} lines",
            clips.len(),
            timeline.sentences.len()
        ));
    }
    let rate = timeline.rate;
    let mut track: Vec<i16> = vec![0; (timeline.total * f64::from(rate)) as usize];
    for (entry, clip) in timeline.sentences.iter().zip(clips) {
        let (data, _) = read(clip, Some(rate))?;
        let data = clean(&data, rate);
        let at = (entry.start * f64::from(rate)) as usize;
        if track.len() < at + data.len() {
            track.resize(at + data.len(), 0);
        }
        track[at..at + data.len()].copy_from_slice(&data);
    }
    let gain = master(&mut track, rate, target).ok_or_else(|| {
        format!(
            "{}: there is no sound in it to bring to a loudness",
            out.display()
        )
    })?;
    match tone {
        // The room is as much louder as the takes it was recorded with.
        Some(tone) => {
            let tone: Vec<f64> = tone
                .iter()
                .map(|s| f64::from(*s) / 32768.0 * gain)
                .collect();
            fill(&mut track, &quantized(&tone), rate);
        }
        None => {
            // Two seconds of the stand-in, repeated.
            let noise = floor(2 * rate as usize, rate);
            fill(&mut track, &noise, rate);
        }
    }
    if let Some(bed) = music {
        underlay(&mut track, bed, rate, target);
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(out, spec).map_err(|e| format!("{}: {e}", out.display()))?;
    for s in track {
        writer.write_sample(s).map_err(|e| e.to_string())?;
    }
    writer
        .finalize()
        .map_err(|e| format!("{}: {e}", out.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const RATE: u32 = 48000;

    /// A second of a sine at `hz`, `db` below full scale.
    fn sine(hz: f64, db: f64) -> Vec<i16> {
        let level = libm::pow(10.0, db / 20.0);
        (0..RATE)
            .map(|n| {
                let t = f64::from(n) / f64::from(RATE);
                (libm::sin(std::f64::consts::TAU * hz * t) * level * 32767.0) as i16
            })
            .collect()
    }

    /// The level of the second half of a clip, in dBFS (RMS): past where
    /// the filters and the compressor settle.
    fn level(samples: &[i16]) -> f64 {
        let half = &samples[samples.len() / 2..];
        let power = half.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / half.len() as f64;
        10.0 * libm::log10(power / (32768.0 * 32768.0))
    }

    fn sha(samples: &[i16]) -> String {
        let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    #[test]
    fn a_quiet_voice_passes_and_rumble_doesnt() {
        // 200 Hz at -30 dBFS is under the compressor and over the high-pass.
        let voice = sine(200.0, -30.0);
        let change = level(&clean(&voice, RATE)) - level(&voice);
        assert!(change.abs() < 0.5, "a quiet tone changed by {change:.2} dB");
        // 30 Hz is under the high-pass: down by more than 12 dB.
        let rumble = sine(30.0, -30.0);
        let change = level(&clean(&rumble, RATE)) - level(&rumble);
        assert!(change < -12.0, "rumble changed by {change:.2} dB");
    }

    #[test]
    fn loudness_is_measured_as_the_standard_does() {
        // EBU Tech 3341: a 1 kHz sine at -23 dBFS on both channels is -23 LUFS.
        let measured = loudness(&sine(1000.0, -23.0), RATE).unwrap();
        assert!((measured + 23.0).abs() < 0.1, "{measured:.2} LUFS");
        // 20 dB quieter is 20 LU less, and silence has no loudness.
        let quiet = loudness(&sine(1000.0, -43.0), RATE).unwrap();
        assert!((quiet + 43.0).abs() < 0.1, "{quiet:.2} LUFS");
        assert_eq!(loudness(&vec![0; RATE as usize], RATE), None);
        assert_eq!(loudness(&[100; 1000], RATE), None);
        // A quiet stretch more than 10 LU under the rest is gated out.
        // Ungated, half loud and half quiet would read 3 LU lower; the
        // few blocks that straddle the join cost a fraction of that.
        let mut gated = sine(1000.0, -23.0);
        gated.extend(sine(1000.0, -23.0));
        gated.extend(sine(1000.0, -23.0));
        gated.extend(sine(1000.0, -50.0));
        gated.extend(sine(1000.0, -50.0));
        gated.extend(sine(1000.0, -50.0));
        let measured = loudness(&gated, RATE).unwrap();
        assert!((measured + 23.0).abs() < 0.4, "{measured:.2} LUFS");
    }

    #[test]
    fn a_narration_is_brought_to_its_loudness_under_the_ceiling() {
        // A quiet second and a loud one, far from the target together.
        let mut track = sine(300.0, -40.0);
        track.extend(sine(300.0, -22.0));
        track.extend(sine(300.0, -40.0));
        let before = loudness(&track, RATE).unwrap();
        let gain = master(&mut track, RATE, -16.0).unwrap();
        let after = loudness(&track, RATE).unwrap();
        assert!(
            (after + 16.0).abs() < 0.3,
            "{before:.1} became {after:.2} LUFS"
        );
        assert!(gain > 1.0);
        let ceiling = (libm::pow(10.0, CEILING / 20.0) * 32768.0) as i32 + 1;
        assert!(track.iter().all(|s| i32::from(*s).abs() <= ceiling));
        // Silence has nothing to bring anywhere.
        assert_eq!(master(&mut vec![0; RATE as usize], RATE, -16.0), None);
    }

    #[test]
    fn loud_is_brought_nearer_quiet_and_peaks_are_held() {
        // 12 dB over the threshold at 2.5 to 1 comes out some 7 dB lower.
        let loud = sine(200.0, -6.0);
        let mut x: Vec<f64> = loud.iter().map(|s| f64::from(*s) / 32768.0).collect();
        compress(&mut x, f64::from(RATE), -18.0);
        let change = level(&quantized(&x)) - level(&loud);
        assert!(
            (-8.5..-5.5).contains(&change),
            "a loud tone changed by {change:.2} dB"
        );
        // Under the threshold and its knee, nothing changes.
        let mut quiet: Vec<f64> = sine(200.0, -30.0)
            .iter()
            .map(|s| f64::from(*s) / 32768.0)
            .collect();
        let same = quiet.clone();
        compress(&mut quiet, f64::from(RATE), -18.0);
        assert_eq!(quiet, same);

        // The limiter: a burst at full scale in a quiet tone comes out at
        // the ceiling, and the tone well away from it is untouched.
        let mut x: Vec<f64> = sine(200.0, -20.0)
            .iter()
            .map(|s| f64::from(*s) / 32768.0)
            .collect();
        for s in x[24000..24480].iter_mut() {
            *s *= 10.0;
        }
        let before = x.clone();
        limit(&mut x, f64::from(RATE), 0.5);
        let peak = x.iter().fold(0.0f64, |m, s| m.max(s.abs()));
        assert!(peak <= 0.5 && peak > 0.49, "the peak is {peak}");
        assert_eq!(x[..23000], before[..23000]);
        assert_eq!(x[26000..], before[26000..]);
        // Nothing over the ceiling, nothing done.
        let mut calm = before[..20000].to_vec();
        limit(&mut calm, f64::from(RATE), 0.5);
        assert_eq!(calm, before[..20000]);
    }

    #[test]
    fn hiss_is_turned_down_more_than_a_vowel_as_loud() {
        // Both at -24 dBFS: the de-esser acts on the one in its band.
        let (vowel, hiss) = (sine(1000.0, -24.0), sine(6500.0, -24.0));
        let on_vowel = level(&clean(&vowel, RATE)) - level(&vowel);
        let on_hiss = level(&clean(&hiss, RATE)) - level(&hiss);
        assert!(
            on_vowel.abs() < 1.0,
            "the vowel changed by {on_vowel:.2} dB"
        );
        assert!(
            on_hiss < on_vowel - 3.0,
            "the hiss changed by {on_hiss:.2} dB"
        );
        // A clip sampled too slowly to hold the band is left alone there.
        let slow: Vec<i16> = sine(1000.0, -24.0).into_iter().step_by(4).collect();
        assert_eq!(clean(&slow, 12000).len(), slow.len());
    }

    #[test]
    fn a_click_is_drawn_over_and_a_voice_without_one_is_untouched_by_it() {
        let voice = sine(200.0, -30.0);
        let mut clicked = voice.clone();
        clicked[30000] += 9000;
        clicked[30001] -= 7000;
        let (plain, mended) = (clean(&voice, RATE), clean(&clicked, RATE));
        let worst = plain
            .iter()
            .zip(&mended)
            .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
            .max()
            .unwrap();
        // The spike was 9,000 steps high; what is left of it is a few.
        assert!(worst < 40, "the click left {worst} steps");
        // Noise is rough everywhere, so nothing in it stands out as a click.
        let noise = floor(RATE as usize, RATE);
        let mut x: Vec<f64> = noise.iter().map(|s| f64::from(*s) / 32768.0).collect();
        let before = x.clone();
        declick(&mut x, f64::from(RATE));
        assert_eq!(x, before);
    }

    #[test]
    fn the_chain_and_the_stand_in_noise_are_the_same_bytes_everywhere() {
        // A sweep of levels and pitches through every stage. The hashes
        // were taken on one machine; CI checks them on another.
        let mut clip = sine(180.0, -10.0);
        clip.extend(sine(6500.0, -20.0));
        clip.extend(sine(40.0, -15.0));
        clip[20000] = 30000;
        assert_eq!(clean(&clip, RATE), clean(&clip, RATE));
        assert_eq!(
            sha(&clean(&clip, RATE)),
            "0043895b3f0fdde6e73bb0e4b572d0affec84a60b891179ff7fcc846f91d6c34"
        );
        let noise = floor(4800, RATE);
        assert_eq!(
            sha(&noise),
            "f3f54e5390dba9a6524fa8c1f360c40164f943d35f66da960b278b68cdaee811"
        );
        // The noise is at its level, and never nothing.
        let rms = level(&noise);
        assert!((rms - FLOOR).abs() < 1.5, "the noise is at {rms:.1} dBFS");
        assert!(noise.iter().all(|s| *s != 0));
    }

    /// The longest run of zeros, in samples.
    fn longest_silence(track: &[i16]) -> usize {
        let (mut longest, mut run) = (0, 0);
        for s in track {
            run = if *s == 0 { run + 1 } else { 0 };
            longest = longest.max(run);
        }
        longest
    }

    #[test]
    fn pauses_are_filled_and_sound_is_left_alone() {
        let tone: Vec<i16> = (0..RATE as i32).map(|n| 20 + (n % 7) as i16).collect();
        // Sound, a pause of half a second, sound, a gap of 2 ms, sound.
        let mut track = vec![1000i16; 4800];
        track.extend(vec![0; 24000]);
        track.extend(vec![-1000; 4800]);
        track.extend(vec![0; 96]);
        track.extend(vec![500; 4800]);
        let before = track.clone();
        fill(&mut track, &tone, RATE);
        // Every sample of the pause is sound now, faint at its ends and
        // the tone itself in the middle.
        assert!(track[4800..28800].iter().all(|s| *s != 0));
        assert!(track[4800].abs() <= 2 && track[28799].abs() <= 2);
        assert_eq!(
            track[16000..16010],
            looped(&tone, 38496, RATE)[16000..16010]
        );
        // The sound, and the gap too short to be a pause, are as they were.
        for span in [0..4800, 28800..33600, 33600..33696, 33696..38496] {
            assert_eq!(track[span.clone()], before[span]);
        }
    }

    #[test]
    fn a_narration_has_no_digital_silence() {
        use vidni_core::timeline::Entry;
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("clip.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        // A clip with silence of its own in the middle, as a draft voice's may have.
        let mut writer = hound::WavWriter::create(&clip, spec).unwrap();
        let mut samples = sine(200.0, -20.0);
        samples.extend(vec![0; 9600]);
        samples.extend(sine(300.0, -20.0));
        for s in &samples {
            writer.write_sample(*s).unwrap();
        }
        writer.finalize().unwrap();
        let line = |start: f64| Entry {
            k: 0,
            segment: "s".into(),
            label: String::new(),
            index: 0,
            start,
            end: start + 2.2,
            wav: String::new(),
            show: String::new(),
            say: String::new(),
            card: None,
            cues: Vec::new(),
        };
        let timeline = Timeline {
            total: 8.0,
            rate: RATE,
            sentences: vec![line(0.6), line(4.0)],
        };
        let read_back = |tone: Option<&[i16]>| {
            let out = dir.path().join("narration.wav");
            narrate(&timeline, &[&clip, &clip], tone, None, -16.0, &out).unwrap();
            read(&out, Some(RATE)).unwrap().0
        };
        // With no room tone, the stand-in noise fills every pause.
        let track = read_back(None);
        assert_eq!(track.len(), 8 * RATE as usize);
        // 50 ms is 2,400 samples; the longest silence left is a sample or two.
        assert!(
            longest_silence(&track) < 48,
            "{} samples",
            longest_silence(&track)
        );
        // The same again is the same bytes.
        assert_eq!(read_back(None), track);
        // With a room's tone, the pauses hold the room.
        // With a room's tone, the pauses hold the room, as much louder as
        // the clips were made.
        let tone = vec![37i16; RATE as usize];
        let roomy = read_back(Some(&tone));
        let held = roomy[RATE as usize * 7];
        assert!(held != 37 && held > 0, "the room is at {held}");
        assert!(
            roomy[RATE as usize * 7..RATE as usize * 7 + 100]
                .iter()
                .all(|s| *s == held)
        );
        assert!(longest_silence(&roomy) < 48);
        // And the narration is at its loudness.
        let lufs = loudness(&track, RATE).unwrap();
        assert!((lufs + 16.0).abs() < 0.5, "{lufs:.2} LUFS");
    }

    #[test]
    fn music_plays_under_the_narration_and_fades_at_both_ends() {
        // Ten seconds with nothing in them, and a three-second track,
        // which is repeated to fill them.
        let mut narration = vec![0i16; 10 * RATE as usize];
        let bed = Bed {
            samples: [sine(440.0, -10.0), sine(550.0, -10.0), sine(660.0, -10.0)].concat(),
            under: 20.0,
        };
        underlay(&mut narration, &bed, RATE, -16.0);
        // In the middle it is 20 LU under the narration's -16 LUFS.
        let middle = loudness(&narration[3 * RATE as usize..7 * RATE as usize], RATE).unwrap();
        assert!((middle + 36.0).abs() < 0.5, "{middle:.2} LUFS");
        // It comes in over a second and goes out over two.
        let peak =
            |span: std::ops::Range<usize>| narration[span].iter().map(|s| s.abs()).max().unwrap();
        let full = peak(4 * RATE as usize..5 * RATE as usize);
        assert_eq!(narration[0], 0);
        assert!(peak(0..4800) < full / 5, "{} of {full}", peak(0..4800));
        let end = narration.len();
        assert!(peak(end - 4800..end) < full / 10);
        assert!(peak(end - 2 * RATE as usize - 4800..end - 2 * RATE as usize) > full * 9 / 10);
        // Over a narration, the two are summed.
        let mut spoken = sine(200.0, -16.0);
        let alone = spoken.clone();
        underlay(&mut spoken, &bed, RATE, -16.0);
        assert_ne!(spoken, alone);
        // A track with no sound adds none.
        let mut still = alone.clone();
        let silent = Bed {
            samples: vec![0; RATE as usize],
            under: 20.0,
        };
        underlay(&mut still, &silent, RATE, -16.0);
        assert_eq!(still, alone);
    }

    #[test]
    fn a_track_is_read_as_one_channel_at_the_narrations_rate() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, channels: u16, rate: u32, bits: u16, frames: &[[i16; 2]]| {
            let path = dir.path().join(name);
            let spec = hound::WavSpec {
                channels,
                sample_rate: rate,
                bits_per_sample: bits,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::create(&path, spec).unwrap();
            for frame in frames {
                for s in &frame[..usize::from(channels)] {
                    match bits {
                        16 => writer.write_sample(*s).unwrap(),
                        _ => writer.write_sample(i32::from(*s)).unwrap(),
                    }
                }
            }
            writer.finalize().unwrap();
            path
        };
        // Two channels are averaged.
        let stereo = write("stereo.wav", 2, 48000, 16, &[[100, 300], [-50, 50], [8, 9]]);
        assert_eq!(track(&stereo, 48000).unwrap(), [200, 0, 9]);
        // At half the rate, each sample in between is halfway between two.
        let slow = write("slow.wav", 1, 24000, 16, &[[0, 0], [100, 0], [300, 0]]);
        assert_eq!(track(&slow, 48000).unwrap(), [0, 50, 100, 200, 300]);
        // At twice the rate, every other sample is taken.
        assert_eq!(track(&slow, 12000).unwrap(), [0, 300]);
        // Another sample format isn't read.
        let deep = write("deep.wav", 1, 48000, 24, &[[1, 0], [2, 0]]);
        let error = track(&deep, 48000).unwrap_err();
        assert!(
            error.ends_with("music is a 16-bit WAV with one or two channels"),
            "{error}"
        );
        assert!(track(&dir.path().join("none.wav"), 48000).is_err());
    }

    #[test]
    fn a_room_is_a_recording_at_the_tracks_rate_with_something_in_it() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, rate: u32, samples: &[i16]| {
            let path = dir.path().join(name);
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::create(&path, spec).unwrap();
            for s in samples {
                writer.write_sample(*s).unwrap();
            }
            writer.finalize().unwrap();
            path
        };
        let good = write("tone.wav", RATE, &vec![5; RATE as usize]);
        assert_eq!(room(&good, RATE).map(|t| t.len()), Some(RATE as usize));
        // Another rate, too short, all zeros, or not there: no room.
        assert_eq!(room(&good, 22050), None);
        assert_eq!(room(&write("short.wav", RATE, &[5; 1000]), RATE), None);
        assert_eq!(
            room(&write("dead.wav", RATE, &vec![0; RATE as usize]), RATE),
            None
        );
        assert_eq!(room(&dir.path().join("none.wav"), RATE), None);
    }
}
