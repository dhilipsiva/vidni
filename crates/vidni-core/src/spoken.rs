//! What a line says, and the key its audio is filed under.
//!
//! A line's takes, its synthetic clips and its captions are keyed by its ID
//! plus a hash of its spoken form. The hash is taken over the spoken words
//! alone, so an edit to punctuation or capitals doesn't make a take stale,
//! and an edit to the words does.

use crate::project::{Line, Respelling, Voice};
use sha2::{Digest, Sha256};

/// What is said for `line`: its `say`, or its `show` with the project's
/// readings (`voice.read_as`) applied.
pub fn spoken(line: &Line, voice: &Voice) -> String {
    match &line.say {
        Some(say) => say.clone(),
        None => replace_all(&line.show, &voice.read_as),
    }
}

/// What the synthetic voice is given: `say` or `show`, with the voice's
/// respellings applied in order. The readings are for people; the voice
/// reads symbols its own way unless a respelling says otherwise.
pub fn synthetic(line: &Line, voice: &Voice) -> String {
    replace_all(line.say.as_deref().unwrap_or(&line.show), &voice.respell)
}

fn replace_all(text: &str, pairs: &[Respelling]) -> String {
    pairs
        .iter()
        .fold(text.to_string(), |text, r| text.replace(&r.text, &r.say))
}

/// The words of `text`: lowercase, apostrophes joined (`it's` is `its`),
/// every other punctuation mark a space, and single spaces between words.
pub fn words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.to_lowercase().chars() {
        if c == '\'' || c == '\u{2019}' {
            continue;
        }
        out.push(if c.is_whitespace() || is_punctuation(c) {
            ' '
        } else {
            c
        });
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Punctuation that separates words. Letters, digits and combining marks in
/// any script are kept.
fn is_punctuation(c: char) -> bool {
    c.is_ascii_punctuation()
        || matches!(
            c,
            '\u{2018}'
                | '\u{201C}'
                | '\u{201D}'
                | '\u{2026}'
                | '\u{2013}'
                | '\u{2014}'
                | '\u{00B7}'
                | '\u{00AB}'
                | '\u{00BB}'
                | '\u{00A1}'
                | '\u{00BF}'
                | '\u{0964}'
                | '\u{0965}'
        )
}

/// The key of `line`'s audio: the first 16 hex digits of the SHA-256 of its spoken words.
pub fn spoken_hash(line: &Line, voice: &Voice) -> String {
    let digest = Sha256::digest(words(&spoken(line, voice)).as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(show: &str) -> Line {
        Line {
            id: "a-1".into(),
            section: "a".into(),
            show: show.into(),
            say: None,
            source: None,
            claim: None,
            scene: None,
            actions: Vec::new(),
            highlight: Vec::new(),
            camera: None,
            camera_vertical: None,
            overlay: None,
        }
    }

    fn hash(show: &str) -> String {
        spoken_hash(&line(show), &Voice::default())
    }

    #[test]
    fn words_ignore_punctuation_and_case() {
        assert_eq!(
            words("Hello, World! It's   nibli—a “reasoner”."),
            "hello world its nibli a reasoner"
        );
        assert_eq!(words("TRUE; FALSE: UNKNOWN…"), "true false unknown");
    }

    #[test]
    fn a_punctuation_edit_keeps_the_hash_and_a_word_edit_changes_it() {
        assert_eq!(hash("Adam is a dog."), hash("Adam is a dog!"));
        assert_eq!(hash("Adam is a dog."), hash("adam, is a dog"));
        assert_ne!(hash("Adam is a dog."), hash("Adam is a cat."));
        assert_eq!(hash("x").len(), 16);
    }

    #[test]
    fn say_and_readings_set_the_spoken_form() {
        let mut voice = Voice::default();
        voice.read_as.push(Respelling {
            text: "#1".into(),
            say: "number one".into(),
        });
        voice.respell.push(Respelling {
            text: "nibli".into(),
            say: "nib lee".into(),
        });
        let l = line("Source #1 is nibli.");
        assert_eq!(spoken(&l, &voice), "Source number one is nibli.");
        assert_eq!(synthetic(&l, &voice), "Source #1 is nib lee.");
        let said = Line {
            say: Some("Something else.".into()),
            ..l
        };
        assert_eq!(spoken(&said, &voice), "Something else.");
    }
}
