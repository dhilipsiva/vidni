//! The compositor: frames of a cut from its captures, cards, overlays and
//! timeline (TODO.md, P3-VIDNI-02). It reproduces the reference's
//! compositor, so its frames match the golden cuts.

pub mod card;
pub mod cut;
pub mod image;
pub mod pillow;
pub mod text;

use cut::{Composer, FrameKey};
use sha2::{Digest, Sha256};

/// Renders every frame of a cut in order, handing each to `sink`. A frame
/// equal to the one before (the same key) is handed over again without
/// being drawn again. Returns the SHA-256 of the whole rgb24 stream, which
/// is the cut's T1 frame hash, and the number of frames drawn.
pub fn render(
    composer: &mut Composer,
    mut sink: impl FnMut(u64, &[u8]) -> Result<(), String>,
) -> Result<(String, u64), String> {
    let mut hash = Sha256::new();
    let mut last: Option<(FrameKey, Vec<u8>)> = None;
    let mut drawn = 0;
    for f in 0..composer.frames() {
        let key = composer.key(f);
        if last.as_ref().is_none_or(|(k, _)| *k != key) {
            last = Some((key, composer.frame(f)?.data));
            drawn += 1;
        }
        let bytes = &last.as_ref().expect("a frame").1;
        hash.update(bytes);
        sink(f, bytes)?;
    }
    let hex = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok((hex, drawn))
}
