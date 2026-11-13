//! Debugging aid: renders one frame of each cut at a time, as PNGs, to
//! compare with the golden frame at the same time.
//! cargo run --release -p vidni-parity --example compose_frames -- <seconds> <out dir> [cut name]

use std::path::{Path, PathBuf};
use vidni_parity::compose::{jobs, reference_faces};

fn main() {
    let mut args = std::env::args().skip(1);
    let t: f64 = args
        .next()
        .expect("a time in seconds")
        .parse()
        .expect("a number");
    let out = PathBuf::from(args.next().expect("an output folder"));
    let only = args.next();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let faces = reference_faces().unwrap();
    for mut job in jobs(
        &root.join("examples/nibli-syllogism"),
        &root.join("reference"),
        &faces,
    )
    .unwrap()
    {
        if only.as_ref().is_some_and(|o| *o != job.name) {
            continue;
        }
        // ffmpeg's accurate seek gives the first frame at or after t.
        let f = (t * 30.0 - 1e-9).ceil() as u64;
        let frame = job.composer.frame(f).unwrap();
        let path = out.join(format!("{}-{f}.png", job.name));
        frame.save_png(&path).unwrap();
        println!("{}", path.display());
    }
}
