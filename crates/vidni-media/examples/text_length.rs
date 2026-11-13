//! Debugging aid: prints vidni's measured width of a text, and the caption
//! box the reference would size from it.
//! cargo run -p vidni-media --example text_length -- <font file> <size> <text>
//!
//! With no arguments, reads `font file<TAB>size<TAB>text` rows from stdin
//! and prints one width per row, to compare with Pillow's `getlength`.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;
use vidni_media::compose::text::Fonts;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut fonts = Fonts::new();
    if args.is_empty() {
        let mut loaded = HashMap::new();
        for row in std::io::stdin().lock().lines() {
            let row = row.unwrap();
            let mut cols = row.splitn(3, '\t');
            let (path, size, text) = (
                cols.next().unwrap(),
                cols.next().unwrap(),
                cols.next().unwrap(),
            );
            let font = loaded
                .entry(path.to_string())
                .or_insert_with(|| fonts.load(Path::new(path)).unwrap())
                .clone();
            println!("{:.6}", fonts.length(&font, size.parse().unwrap(), text));
        }
        return;
    }
    let font = fonts.load(Path::new(&args[0])).unwrap();
    let size: f32 = args[1].parse().unwrap();
    let tw = fonts.length(&font, size, &args[2]);
    let bw = (tw + 60.0) as i32;
    println!(
        "ours: getlength {tw} bw {bw} x {} box x {}",
        bw as f32 / 2.0,
        (1920 - bw).div_euclid(2)
    );
}
