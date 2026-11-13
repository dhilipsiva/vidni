//! The timeline and caption writers, through the public API. Parity with the
//! reference's golden timelines is checked by `just parity timeline`.

use vidni_core::TEMPLATE;
use vidni_core::timeline::{Clip, build, srt, vtt};
use vidni_core::validate::check;

fn clip(seconds: f64) -> Clip {
    Clip {
        wav: "x.wav".into(),
        frames: (seconds * 48000.0) as u64,
        rate: 48000,
    }
}

#[test]
fn lines_follow_the_lead_in_and_the_gaps_and_the_tail_ends_it() {
    let project = check(TEMPLATE, None).unwrap();
    let t = build(&project, &[clip(2.0), clip(3.5)]).unwrap();
    let spans: Vec<(f64, f64)> = t.sentences.iter().map(|e| (e.start, e.end)).collect();
    // lead_in 0.6, line_gap 0.28 within the section, tail 2.4.
    assert_eq!(spans, [(0.6, 2.6), (2.88, 6.38)]);
    assert_eq!(t.total, 8.78);
    assert_eq!(t.rate, 48000);
    assert_eq!(
        (t.sentences[1].segment.as_str(), t.sentences[1].index),
        ("intro", 1)
    );
}

#[test]
fn a_new_section_waits_the_section_gap() {
    let text = TEMPLATE
        .replace(
            "[sections.intro]\nlabel = \"Introduction\"\n",
            "[sections.intro]\nlabel = \"Introduction\"\n\n[sections.more]\nlabel = \"More\"\n",
        )
        .replace(
            "section = \"intro\"\nshow = \"This video",
            "section = \"more\"\nshow = \"This video",
        );
    let project = check(&text, None).unwrap();
    let t = build(&project, &[clip(2.0), clip(1.0)]).unwrap();
    assert_eq!(t.sentences[1].start, 3.35); // 2.6 + section_gap 0.75
    assert_eq!(
        (t.sentences[1].index, t.sentences[1].label.as_str()),
        (0, "More")
    );
}

#[test]
fn clips_must_match_the_lines_and_share_a_rate() {
    let project = check(TEMPLATE, None).unwrap();
    assert!(build(&project, &[clip(1.0)]).is_err());
    let odd = Clip {
        rate: 22050,
        ..clip(1.0)
    };
    assert!(
        build(&project, &[clip(1.0), odd])
            .unwrap_err()
            .contains("sample rate 22050")
    );
}

#[test]
fn captions_as_srt_and_vtt() {
    let project = check(TEMPLATE, None).unwrap();
    let t = build(&project, &[clip(2.0), clip(3.5)]).unwrap();
    assert_eq!(
        srt(&t),
        "1\n00:00:00,600 --> 00:00:02,600\nHello, and welcome.\n\n\
         2\n00:00:02,880 --> 00:00:06,380\nThis video was made with vidni.\n\n"
    );
    assert_eq!(
        vtt(&t),
        "WEBVTT\n\n00:00:00.600 --> 00:00:02.600\nHello, and welcome.\n\n\
         00:00:02.880 --> 00:00:06.380\nThis video was made with vidni.\n\n"
    );
}
