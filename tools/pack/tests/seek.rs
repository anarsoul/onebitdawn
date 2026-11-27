// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The page's seek (`web::Session::seek`) over a clip of every kind:
//! what is heard after it against the clip played to the same place.
//! `core` holds each player's `skip` to its state; this is the sound,
//! with the filters and the limiter a skip leaves behind.

use std::path::Path;
use std::time::Instant;

use onebitdawn_core::{adpcm, clip, pt3, sid, xm};
use onebitdawn_pack::web::Session;
use onebitdawn_pack::{Firmware, Source, Spec, Target, convert_for};

fn repo(path: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Three seconds of a note that fades, with a click in every second.
fn wav() -> Vec<u8> {
    let rate = 16000u32;
    let frames = rate * 3;
    let mut out = b"RIFF\0\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0".to_vec();
    out.extend(rate.to_le_bytes());
    out.extend((rate * 2).to_le_bytes());
    out.extend([2, 0, 16, 0]);
    out.extend(b"data");
    out.extend((frames * 2).to_le_bytes());
    for k in 0..frames {
        let t = k as f64 / rate as f64;
        let click = if k % rate < 40 { 9000.0 } else { 0.0 };
        out.extend(((12000.0 * (2.0 * core::f64::consts::PI * 700.0 * t).sin() * (1.0 - t / 3.0) + click) as i16).to_le_bytes());
    }
    out
}

fn rms(samples: &[f32]) -> f64 {
    (samples.iter().map(|&s| s as f64 * s as f64).sum::<f64>() / samples.len().max(1) as f64).sqrt()
}

/// The next `n` samples of the session's clip, or what there is.
fn hear(session: &mut Session, n: usize) -> Vec<f32> {
    let mut out = Vec::new();
    let mut buf = [0f32; 4096];
    while out.len() < n {
        let got = session.samples(&mut buf[..(n - out.len()).next_multiple_of(64).min(4096)]);
        if got == 0 {
            break;
        }
        out.extend(&buf[..got]);
    }
    // (Asked for in 64s, as the players like it.)
    out.truncate(n);
    out
}

/// A clip heard after seeks all over it, forward and back, against
/// itself played through.
fn seeks_as_played(name: &str, data: &[u8]) {
    let mut session = Session::default();
    let id = session.adopt(data);
    let rate = session.preview(id);
    assert!(rate != 0, "{name}");
    let started = Instant::now();
    let all = hear(&mut session, usize::MAX >> 1);
    let played = started.elapsed();
    let len = all.len() as u32;
    assert!(len > rate / 2, "{name}: {len} samples");
    let tune = xm::is_tune(data) || pt3::is_tune(data) || sid::is_tune(data);
    // A SID's noise and its synced voices are not the same samples
    // after a seek, only the same sound.
    let alike = !sid::is_tune(data);
    let window = (rate / 2) as usize;
    let mut worst = 0f64;
    // Back and forth.
    for tenths in [5u32, 9, 2, 7, 0, 3] {
        let to = (len as u64 * tenths as u64 / 10) as u32;
        assert_eq!(session.seek(to), to, "{name}: seek to {to} of {len}");
        let got = hear(&mut session, window);
        let want = &all[to as usize..(to as usize + window).min(all.len())];
        assert_eq!(got.len(), want.len(), "{name}: after a seek to {to} of {len}");
        if !tune {
            assert!(got == want, "{name}: after a seek to {to} of {len}");
            continue;
        }
        let level = rms(want).max(1e-3);
        let off = if alike {
            let diff: Vec<f32> = got.iter().zip(want).map(|(a, b)| a - b).collect();
            rms(&diff) / level
        } else {
            (rms(&got) - rms(want)).abs() / level
        };
        worst = worst.max(off);
    }
    // Past the end is the end. (A beeper clip's is where the buffer
    // it ended in did: a line's silence after it is no sample fewer.)
    let end = session.seek(u32::MAX);
    assert!(end <= len && len - end < if clip::is_beeper(data) { 4096 } else { 1 }, "{name}: {end} of {len}");
    assert!(hear(&mut session, 64).is_empty());
    let started = Instant::now();
    session.seek(0);
    session.seek(len - 1);
    let sought = started.elapsed();
    println!(
        "{name}: {:.1} s, played in {:.1} ms, a seek to its end {:.2} ms; after a seek {:.2} % off",
        len as f64 / rate as f64,
        played.as_secs_f64() * 1e3,
        sought.as_secs_f64() * 1e3,
        worst * 100.0
    );
    assert!(worst < if alike { 0.02 } else { 0.15 }, "{name}: {worst}");
}

#[test]
fn a_seek_is_heard_as_the_clip_played_to_there() {
    for file in ["clips/keygen.xmc", "clips/lera.xmc", "clips/morning_mood.xmc", "clips/suspended_seconds.xmc", "clips/serial_dawn.xmc", "clips/synchronization.pt3c", "tools/pack/tests/data/tiny-mod.xmc", "tools/pack/tests/data/tiny.sidc"] {
        seeks_as_played(file, &repo(file));
    }
    // Sampled sound in each of its codings, and a beeper clip.
    for format in ["", "format=adpcm", "format=pcm", "format=pcm16"] {
        let spec = Spec::parse(&format!("ring.wav:{format}"));
        let made = convert_for(&Source { file: spec.file, data: wav(), options: spec.options, label: None }, Target { firmware: Firmware::V0_6, ..Default::default() }).unwrap();
        assert!(format == "format=pcm" || adpcm::is_clip(&made.data));
        seeks_as_played(&format!("a sound file, {format}"), &made.data);
    }
    let mut beeper = clip::BEEPER_MAGIC.to_vec();
    let runs: Vec<u8> = (0..40_000u32).flat_map(|k| [60 + (k / 400 % 50) as u8]).collect();
    beeper.extend(runs.iter().map(|&r| r as u32).sum::<u32>().to_le_bytes());
    beeper.extend(runs);
    assert!(clip::is_beeper(&beeper));
    seeks_as_played("a beeper clip", &beeper);
    // And whatever else is named, to see what a seek costs.
    for file in std::env::var("SEEK_CLIPS").unwrap_or_default().split(',').filter(|f| !f.is_empty()) {
        seeks_as_played(file, &std::fs::read(file).unwrap());
    }
}
