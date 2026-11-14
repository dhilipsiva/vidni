//! Lists the system's inputs, and says which one vidni would record from
//! and at what rate. It opens no stream, so nothing is recorded.
//!
//! `cargo run -p vidni-record --example inputs [NAME]`

use vidni_record::input::Input;
use vidni_record::mic::Mic;

fn main() -> Result<(), String> {
    let wanted = std::env::args().nth(1);
    for name in Mic::all()? {
        println!("input: {name}");
    }
    let mic = Mic::find(wanted.as_deref())?;
    println!(
        "vidni would record from {} at {} Hz",
        mic.name(),
        mic.rate()
    );
    Ok(())
}
