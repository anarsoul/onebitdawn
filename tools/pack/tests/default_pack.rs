// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The default pack's clips made again from the modules in `mods/`:
//! byte for byte what is in `clips/`, which the Python converters this
//! crate was ported from made, and which the clocks have. The kinds
//! the default pack has none of are held to `tests/data/`, the same
//! converters' output for three small files.

use std::path::Path;

use onebitdawn_core::pack::Pack;
use onebitdawn_pack::{Clip, Output, Source, Spec, build, convert};

fn repo(path: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The clip of a file of the repository, by an argument of the command.
fn made(arg: &str, output: Output) -> Clip {
    let spec = Spec::parse(arg);
    let source = Source { data: repo(&spec.file), file: spec.file, options: spec.options, label: spec.label };
    convert(&source, output).unwrap()
}

#[test]
fn the_modules_come_out_as_the_clips_in_the_repository() {
    for (arg, clip) in [
        ("mods/Orion-Nero6.3.0.2.xm:name=keygen", "clips/keygen.xmc"),
        ("mods/nq-synchronization.pt3:restart,name=synchronization", "clips/synchronization.pt3c"),
        ("mods/nq-lera.xm:name=lera", "clips/lera.xmc"),
        ("mods/grieg-morning-bright.xm", "clips/morning_mood.xmc"),
        ("mods/grieg-morning-chip.xm", "clips/morning_mood_chip.xmc"),
        ("mods/grieg-mountain-king-bright.xm", "clips/mountain_king.xmc"),
        ("mods/suspended-seconds.xm:knee=12,release=10", "clips/suspended_seconds.xmc"),
        ("mods/serial-dawn.xm", "clips/serial_dawn.xmc"),
        ("tools/pack/tests/data/tiny.mod", "tools/pack/tests/data/tiny-mod.xmc"),
        ("tools/pack/tests/data/tiny.s3m", "tools/pack/tests/data/tiny-s3m.xmc"),
        ("tools/pack/tests/data/tiny.sid", "tools/pack/tests/data/tiny.sidc"),
    ] {
        let made = made(arg, Output::Buzzer);
        assert!(made.data == repo(clip), "{arg} is not {clip}");
        assert!(made.loops);
    }
}

#[test]
fn a_clip_made_already_goes_in_as_it_is() {
    for clip in [
        "clips/keygen.xmc",
        "clips/synchronization.pt3c",
        "clips/lera.xmc",
        "clips/morning_mood.xmc",
        "clips/morning_mood_chip.xmc",
        "clips/mountain_king.xmc",
        "clips/suspended_seconds.xmc",
        "clips/serial_dawn.xmc",
    ] {
        let made = made(clip, Output::Speaker);
        assert!(made.data == repo(clip) && made.loops && made.notes.is_empty(), "{clip}");
    }
}

#[test]
fn the_default_pack_from_its_sources() {
    let clips = [
        made("mods/Orion-Nero6.3.0.2.xm:name=keygen=Keygen", Output::Buzzer),
        made("mods/nq-synchronization.pt3:restart,name=synchronization=Synchronization", Output::Buzzer),
        made("mods/nq-lera.xm:name=lera=Lera", Output::Buzzer),
        // The modules made here have their names and labels in their titles.
        made("mods/grieg-morning-bright.xm", Output::Buzzer),
        made("mods/grieg-morning-chip.xm", Output::Buzzer),
        made("mods/grieg-mountain-king-bright.xm", Output::Buzzer),
        made("mods/suspended-seconds.xm:knee=12,release=10", Output::Buzzer),
        made("mods/serial-dawn.xm", Output::Buzzer),
    ];
    let pack = build(&clips).unwrap();
    let pack = Pack::parse(&pack).unwrap();
    let names: Vec<&str> = (0..pack.count()).map(|k| pack.clip(k).unwrap().name).collect();
    assert_eq!(names, ["keygen", "synchronization", "lera", "morning_mood", "morning_mood_chip", "mountain_king", "suspended_seconds", "serial_dawn"]);
    let labels: Vec<&str> = (0..pack.count()).map(|k| pack.clip(k).unwrap().label).collect();
    assert_eq!(labels, ["Keygen", "Synchronization", "Lera", "Morning Mood", "Morning Mood Chip", "Mountain King", "Suspended Seconds", "Serial Dawn"]);
    // A speaker's clips are others, of the same lengths: the shaping
    // is the header's.
    // And the speaker's pack has a ninth, which the buzzer's has not
    // (scripts/speaker-clips.sh): its name and label are its title's.
    let ninth = made("mods/starlight-aurora.xm:knee=64,release=13", Output::Speaker);
    assert_eq!((ninth.name.as_str(), ninth.label.as_str(), ninth.loops), ("starlight_and_aurora", "Starlight and Aurora", true));
    // One note, the module's line: nothing in it that the player lacks.
    assert_eq!(ninth.notes.len(), 1, "{:?}", ninth.notes);
    // Its samples are of 16 bits. Kept so they are a clip a firmware
    // from before them refuses; cut to 8 it is a clip of the kind
    // there was, smaller by a byte a sample. The pack's has them
    // predicted, as a module's 16-bit samples are unless told, in 8
    // bits a sample and a sixteenth more, a third kind of clip.
    let wide = made("mods/starlight-aurora.xm:knee=64,release=13,samples=16", Output::Speaker);
    let cut = made("mods/starlight-aurora.xm:knee=64,release=13,samples=8", Output::Speaker);
    assert_eq!(made("mods/starlight-aurora.xm:knee=64,release=13,samples=packed", Output::Speaker).data, ninth.data);
    assert_eq!((&ninth.data[..4], &wide.data[..4], &cut.data[..4]), (&b"XMC4"[..], &b"XMC3"[..], &b"XMC2"[..]));
    assert!(wide.data.len() > cut.data.len() + 100_000 && wide.data.len() < 2 * cut.data.len());
    assert!(ninth.data.len() > cut.data.len() && ninth.data.len() < cut.data.len() + (wide.data.len() - cut.data.len()) / 8);
    for clip in [&ninth, &wide] {
        assert!(!onebitdawn_core::clip::is_unsupported(&clip.data));
        assert_eq!(onebitdawn_core::clip::duration_ms(&clip.data), onebitdawn_core::clip::duration_ms(&cut.data));
    }
    // What the player makes of each, against the 16 bits', limiter
    // and all (the mixes before it are 70 dB and 50 apart): the
    // predicted samples are 10 dB nearer than the 8 bits and more.
    let play = |clip: &Clip| -> Vec<f64> {
        let mut tune = onebitdawn_core::xm::Tune::new(&clip.data);
        core::iter::from_fn(|| tune.next_sample()).map(f64::from).collect()
    };
    let (best, packed, narrow) = (play(&wide), play(&ninth), play(&cut));
    let under = |other: &[f64]| {
        assert_eq!(other.len(), best.len());
        let signal: f64 = best.iter().map(|s| s * s).sum();
        let noise: f64 = best.iter().zip(other).map(|(a, b)| (a - b) * (a - b)).sum();
        10.0 * (signal / noise).log10()
    };
    let (packed, narrow) = (under(&packed), under(&narrow));
    println!("starlight-aurora against its 16 bits: packed {packed:.1} dB, 8 bits {narrow:.1} dB");
    assert!(packed > 45.0 && packed > narrow + 10.0, "packed {packed:.1} dB, 8 bits {narrow:.1} dB");
    // With the eight for a speaker it fits the partition.
    let mut speaker: Vec<Clip> = Vec::new();
    for arg in [
        "mods/Orion-Nero6.3.0.2.xm:name=keygen",
        "mods/nq-synchronization.pt3:restart,name=synchronization",
        "mods/nq-lera.xm:name=lera",
        "mods/grieg-morning-bright.xm",
        "mods/grieg-morning-chip.xm",
        "mods/grieg-mountain-king-bright.xm",
        "mods/suspended-seconds.xm:knee=12,release=13",
        "mods/serial-dawn.xm",
    ] {
        speaker.push(made(arg, Output::Speaker));
    }
    speaker.push(ninth);
    assert_eq!(Pack::parse(&build(&speaker).unwrap()).unwrap().count(), 9);
    // Lera's rolls are R02, which the converter writes as E92: one
    // note, the module's line, and nothing the player lacks.
    assert_eq!(clips[2].notes.len(), 1, "{:?}", clips[2].notes);
    for (arg, k) in [("mods/Orion-Nero6.3.0.2.xm:name=keygen", 0), ("mods/nq-synchronization.pt3:restart,name=synchronization", 1)] {
        let speaker = made(arg, Output::Speaker);
        assert!(speaker.data.len() == clips[k].data.len() && speaker.data != clips[k].data, "{arg}");
    }
}

/// A module of one channel and one instrument, its sample of 16 bits
/// with a loop, and a pattern of `cells` (note, instrument, volume,
/// effect, parameter), a row each.
fn module_of_16_bits(sample: &[i16], looped: (u32, u32), cells: &[[u8; 5]]) -> Vec<u8> {
    // The title and the tracker's name, twenty bytes each.
    let mut d = b"Extended Module: ".to_vec();
    d.extend(format!("{:<20}\x1a{:<20}", "a test", "onebitdawn").bytes());
    assert_eq!(d.len(), 58);
    d.extend([4, 1]);
    d.extend(276u32.to_le_bytes());
    for n in [1u16, 0, 1, 1, 1, 1, 6, 125] {
        d.extend(n.to_le_bytes());
    }
    d.extend([0u8; 256]);
    // The pattern: every cell whole, and rows enough for the second
    // of sound a clip must be.
    let mut cells = cells.to_vec();
    cells.resize(cells.len().max(16), [0; 5]);
    d.extend(9u32.to_le_bytes());
    d.push(0);
    d.extend((cells.len() as u16).to_le_bytes());
    d.extend((cells.len() as u16 * 5).to_le_bytes());
    for cell in &cells {
        assert!(cell[0] & 0x80 == 0);
        d.extend(cell);
    }
    // The instrument: no envelope, one sample.
    let mut ins = vec![0u8; 263];
    ins[..4].copy_from_slice(&263u32.to_le_bytes());
    ins[27] = 1;
    ins[29..33].copy_from_slice(&40u32.to_le_bytes());
    d.extend(ins);
    let mut head = vec![0u8; 40];
    head[..4].copy_from_slice(&(2 * sample.len() as u32).to_le_bytes());
    head[4..8].copy_from_slice(&(2 * looped.0).to_le_bytes());
    head[8..12].copy_from_slice(&(2 * looped.1).to_le_bytes());
    head[12] = 64;
    head[14] = 0x10 | (looped.1 != 0) as u8;
    d.extend(head);
    let mut last = 0i16;
    for &s in sample {
        d.extend(s.wrapping_sub(last).to_le_bytes());
        last = s;
    }
    d
}

#[test]
fn a_16_bit_sample_is_predicted_unless_it_is_played_from_an_offset() {
    use onebitdawn_core::adpcm::{BLOCK, BLOCK_BYTES};
    use onebitdawn_core::xm::{PACKED_HEAD, Tune};
    // A struck string of a kind: partials, a fall, a loop off a
    // block's edge.
    let wave: Vec<i16> = (0..3000)
        .map(|i| {
            let t = i as f64;
            (30000.0 * (-t / 900.0).exp() * (0.6 * (t * 0.11).sin() + 0.3 * (t * 0.31).sin() + 0.1 * (t * 0.8).sin())) as i16
        })
        .collect();
    let looped = (2501, 499);
    let note = [49, 1, 0, 0, 0];
    let rest = [0u8; 5];
    let made = |cells: &[[u8; 5]], options: &str| {
        let source = Source { data: module_of_16_bits(&wave, looped, cells), file: "test.xm".into(), options: Spec::parse(&format!("test.xm:{options}")).options, label: None };
        convert(&source, Output::Speaker).unwrap()
    };
    let cells = [note, rest, rest, rest, rest, rest, rest, rest];
    let (cut, wide, packed) = (made(&cells, "samples=8"), made(&cells, "samples=16"), made(&cells, "samples=packed"));
    assert_eq!((&cut.data[..4], &wide.data[..4], &packed.data[..4]), (&b"XMC2"[..], &b"XMC3"[..], &b"XMC4"[..]));
    // Predicted is what a 16-bit sample is unless told.
    assert_eq!(made(&cells, "").data, packed.data);
    assert_eq!(wide.data.len(), cut.data.len() + wave.len());
    assert_eq!(packed.data.len(), cut.data.len() - wave.len() + PACKED_HEAD + wave.len().div_ceil(BLOCK) * BLOCK_BYTES);
    assert_eq!(packed.notes.len(), 1, "{:?}", packed.notes);
    // Played, limiter and all, the predicted one is the 16 bits' to
    // 45 dB, through the loop, where the wave has fallen to a few of
    // the 8 bits' steps and the cut one is not to 25.
    let play = |clip: &Clip| -> Vec<f64> {
        let mut tune = Tune::new(&clip.data);
        core::iter::from_fn(|| tune.next_sample()).map(f64::from).collect()
    };
    let best = play(&wide);
    let under = |clip: &Clip| {
        let other = play(clip);
        assert_eq!(other.len(), best.len());
        let signal: f64 = best.iter().map(|s| s * s).sum();
        let noise: f64 = best.iter().zip(&other).map(|(a, b)| (a - b) * (a - b)).sum();
        10.0 * (signal / noise).log10()
    };
    let (near, far) = (under(&packed), under(&cut));
    println!("a struck string against its 16 bits: packed {near:.1} dB, 8 bits {far:.1} dB");
    assert!(near > 45.0 && far < 25.0, "packed {near:.1} dB, 8 bits {far:.1} dB");
    // A 9xx on the instrument, or on a cell that names none: the
    // sample stays at 16 bits, and the notes say so.
    for offset in [[49, 1, 0, 9, 2], [49, 0, 0, 9, 2]] {
        let kept = made(&[note, rest, offset, rest], "samples=packed");
        assert_eq!(&kept.data[..4], b"XMC3");
        assert!(kept.notes.iter().any(|n| n.contains("9xx")), "{:?}", kept.notes);
    }
    // A sample of 16 bits whose low bytes are all the same is one of
    // 8: it is written so whatever is asked, the clip there always
    // was, and plays as the 16 bits would.
    let coarse: Vec<i16> = wave.iter().map(|&s| s & !0xFF).collect();
    let of = |options: &str| {
        let source = Source { data: module_of_16_bits(&coarse, looped, &cells), file: "test.xm".into(), options: Spec::parse(&format!("test.xm:{options}")).options, label: None };
        convert(&source, Output::Speaker).unwrap()
    };
    let plain = of("samples=8");
    assert_eq!(&plain.data[..4], b"XMC2");
    for options in ["", "samples=packed", "samples=16"] {
        assert_eq!(of(options).data, plain.data, "{options}");
    }
    assert_eq!(plain.data.len(), cut.data.len());
    // The same with every low byte one value that is not nought, which
    // is an offset of under a step and is dropped: the same clip, and
    // not one rounded up a step.
    for low in [0x01, 0x7F, 0x80, 0xFF] {
        let offset: Vec<i16> = coarse.iter().map(|&s| s | low).collect();
        for options in ["", "samples=8", "samples=16"] {
            let source = Source { data: module_of_16_bits(&offset, looped, &cells), file: "test.xm".into(), options: Spec::parse(&format!("test.xm:{options}")).options, label: None };
            assert_eq!(convert(&source, Output::Speaker).unwrap().data, plain.data, "{low:#x} {options}");
        }
    }
    // One sample off a multiple of 256 and it is of 16 bits again.
    let mut fine = coarse.clone();
    fine[1000] |= 1;
    let source = Source { data: module_of_16_bits(&fine, looped, &cells), file: "test.xm".into(), options: vec![], label: None };
    assert_eq!(&convert(&source, Output::Speaker).unwrap().data[..4], b"XMC4");
    // Asked for anything else, it says what there is.
    let source = Source { data: module_of_16_bits(&wave, looped, &cells), file: "test.xm".into(), options: vec![("samples".into(), "12".into())], label: None };
    assert_eq!(convert(&source, Output::Speaker).err().as_deref(), Some("test.xm: samples is 8, 16 or packed"));
}

#[test]
fn what_a_module_has_that_the_player_lacks_is_said() {
    let module = made("tools/pack/tests/data/tiny.mod", Output::Buzzer);
    assert!(module.notes.iter().any(|n| n == "not converted: the effect E0, the Amiga's filter"), "{:?}", module.notes);
    let module = made("tools/pack/tests/data/tiny.s3m", Output::Buzzer);
    assert!(module.notes.iter().any(|n| n == "not converted: the effects E, H, S"), "{:?}", module.notes);
    let module = made("tools/pack/tests/data/tiny.s3m:raise=1:1,rate=16000", Output::Speaker);
    assert_eq!(module.data.len(), repo("tools/pack/tests/data/tiny-s3m.xmc").len());
    let tune = made("tools/pack/tests/data/tiny.sid:seconds=3,plain", Output::Buzzer);
    assert!(tune.data.starts_with(b"SID1") && tune.notes.iter().any(|n| n.contains("cut, starts over from the top")), "{:?}", tune.notes);
    let refused = convert(&Source { file: "tune.xm".into(), data: b"Extended Module: cut".to_vec(), ..Default::default() }, Output::Buzzer);
    assert_eq!(refused.unwrap_err(), "tune.xm: the file is cut short");
}

/// A second and a half of a tone that dies away, as a WAV file of 16
/// bits and two channels.
fn wav(rate: u32) -> Vec<u8> {
    let frames = rate * 3 / 2;
    let mut out = b"RIFF\0\0\0\0WAVEfmt \x10\0\0\0\x01\0\x02\0".to_vec();
    out.extend(rate.to_le_bytes());
    out.extend((rate * 4).to_le_bytes());
    out.extend([4, 0, 16, 0]);
    out.extend(b"LIST\x03\0\0\0abc\0data");
    out.extend((frames * 4).to_le_bytes());
    for k in 0..frames {
        let t = k as f64 / rate as f64;
        let s = (12000.0 * (2.0 * core::f64::consts::PI * 1500.0 * t).sin() * (1.0 - t / 1.5)) as i16;
        out.extend(s.to_le_bytes());
        out.extend((s / 2).to_le_bytes());
    }
    out
}

#[test]
fn a_sound_file_is_a_clip_at_the_clips_rate() {
    use onebitdawn_core::adpcm;
    // 16 bits predicted unless told: the magic, the rate, the samples'
    // number, the table, a block of 17 bytes for every 16 samples.
    let packed = 8 + 4 + adpcm::TABLE_BYTES + 24000 / 16 * 17;
    for (rate, options, magic, bytes) in [
        (44100, "", &b"ADP2\x80\x3e\0\0"[..], packed),
        (44100, "format=packed", b"ADP2\x80\x3e\0\0", packed),
        (44100, "format=pcm16", b"PCM2\x80\x3e\0\0", 8 + 48000),
        (44100, "format=adpcm", b"ADP1\x80\x3e\0\0", 8 + 12000),
        (22050, "format=pcm,rate=31250", b"PCM1\x12\x7a\0\0", 8 + 46875),
        (22050, "format=pcm16,rate=31250", b"PCM2\x12\x7a\0\0", 8 + 93750),
        (16000, "format=pcm,rate=8000,seconds=1.25", b"", 10000),
    ] {
        let source = Source { file: "ring.wav".into(), data: wav(rate), options: onebitdawn_pack::spec::parse_options(options).unwrap(), label: None };
        let clip = convert(&source, Output::Speaker).unwrap();
        assert!(clip.data.starts_with(magic) && !clip.loops, "{options}");
        assert_eq!(clip.data.len(), bytes, "{options}");
        assert!(!onebitdawn_core::clip::is_unsupported(&clip.data), "{options}");
        // The tone is there, as loud as the clip gets, and it is
        // 1500 Hz still: the samples go through nought that often.
        let (samples, full, out_rate): (Vec<i32>, i32, usize) = if adpcm::is_clip(&clip.data) {
            let mut decoder = adpcm::Decoder::new(&clip.data);
            (core::iter::from_fn(|| decoder.next_sample()).map(i32::from).collect(), 32768, adpcm::rate(&clip.data) as usize)
        } else {
            (clip.data[magic.len()..].iter().map(|&b| b as i32 - 128).collect(), 128, if magic.is_empty() { 8000 } else { 31250 })
        };
        assert_eq!(onebitdawn_core::clip::duration_ms(&clip.data) as usize, samples.len() * 1000 / out_rate, "{options}");
        let loudest = samples.iter().map(|s| s.abs()).max().unwrap();
        assert!((full * 110 / 128..=full).contains(&loudest), "{options}: {loudest}");
        let part = &samples[out_rate / 10..out_rate / 2];
        let crossings = part.windows(2).filter(|w| (w[0] < 0) != (w[1] < 0)).count();
        let hz = crossings as f64 / 2.0 / (part.len() as f64 / out_rate as f64);
        assert!((hz - 1500.0).abs() < 40.0, "{options}: {hz} Hz");
    }
    // Predicted, the tone (pushed into the limiter as a speaker's clip
    // is, so not a pure one) is the 16 bits' to 50 dB; ADPCM's is not
    // to 30.
    let made = |options: &str| {
        let source = Source { file: "ring.wav".into(), data: wav(44100), options: onebitdawn_pack::spec::parse_options(options).unwrap(), label: None };
        let clip = convert(&source, Output::Speaker).unwrap();
        let mut decoder = adpcm::Decoder::new(&clip.data);
        core::iter::from_fn(|| decoder.next_sample()).map(f64::from).collect::<Vec<f64>>()
    };
    let wide = made("format=pcm16");
    let under = |other: Vec<f64>| {
        assert_eq!(other.len(), wide.len());
        let signal: f64 = wide.iter().map(|s| s * s).sum();
        let noise: f64 = wide.iter().zip(&other).map(|(a, b)| (a - b) * (a - b)).sum();
        10.0 * (signal / noise).log10()
    };
    let (packed, four) = (under(made("")), under(made("format=adpcm")));
    println!("a tone against its 16 bits: predicted {packed:.1} dB, ADPCM {four:.1} dB");
    assert!(packed > 50.0 && four < 30.0, "predicted {packed:.1} dB, ADPCM {four:.1} dB");
    let bad = Source { file: "ring.wav".into(), data: wav(44100), options: vec![("format".into(), "flac".into())], label: None };
    assert_eq!(convert(&bad, Output::Speaker).err().as_deref(), Some("ring.wav: format=flac: packed, pcm16, adpcm or pcm"));
    let cut = Source { file: "ring.wav".into(), data: wav(8000)[..60].to_vec(), ..Default::default() };
    assert!(convert(&cut, Output::Buzzer).is_err());
}
