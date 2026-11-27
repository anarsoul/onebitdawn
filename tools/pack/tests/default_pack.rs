// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The default pack's clips made again from the modules in `mods/`:
//! byte for byte what is in `clips/`, which the Python converters this
//! crate was ported from made, and which the clocks have. Lera is the
//! clip for v0.5, its R written as E9: the same sound on a player
//! with R, and its rolls on one without. The kinds
//! the default pack has none of are held to `tests/data/`, the same
//! converters' output for three small files.

use std::path::Path;

use onebitdawn_core::pack::Pack;
use onebitdawn_pack::{Clip, Firmware, Limiter, Output, Source, Spec, Target, build, convert_for};

/// A clock at v0.5 with a speaker.
const OLD: Target = Target { output: Output::Speaker, firmware: Firmware::V0_5, limiter: Limiter::Loudest };

fn repo(path: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A clip with the limiter as the release's speaker pack has it, the
/// loudest, which is the knee as the options give it: what the clips
/// here are held to. The maker's own default fits the knee.
fn convert(source: &Source, output: Output) -> Result<Clip, String> {
    convert_for(source, Target { output, limiter: Limiter::Loudest, ..Default::default() })
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
        if clip == "clips/lera.xmc" {
            // The module's R02 as it is, for a player that has R: the
            // same bytes but for the 26 cells, and the same ticks.
            assert!(made.data != repo(clip) && made.data.len() == repo(clip).len());
            assert_eq!(play(&made.data), play(&repo(clip)));
            let spec = Spec::parse(arg);
            let source = Source { data: repo(&spec.file), file: spec.file, options: spec.options, label: spec.label };
            let old = convert_for(&source, Target { firmware: Firmware::V0_5, limiter: Limiter::Loudest, ..Default::default() }).unwrap();
            assert!(old.data == repo(clip), "{arg} for v0.5 is not {clip}");
            continue;
        }
        assert!(made.data == repo(clip), "{arg} is not {clip}");
        assert!(made.loops);
    }
}

/// What the player makes of a module's clip.
fn play(clip: &[u8]) -> Vec<i16> {
    let mut tune = onebitdawn_core::xm::Tune::new(clip);
    core::iter::from_fn(|| tune.next_sample()).collect()
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
    // Lera's rolls are R02, which the player has (and the converter
    // writes as E92 for one that has not): one note, the module's
    // line, and nothing the player lacks.
    assert_eq!(clips[2].notes.len(), 1, "{:?}", clips[2].notes);
    for (arg, k) in [("mods/Orion-Nero6.3.0.2.xm:name=keygen", 0), ("mods/nq-synchronization.pt3:restart,name=synchronization", 1)] {
        let speaker = made(arg, Output::Speaker);
        assert!(speaker.data.len() == clips[k].data.len() && speaker.data != clips[k].data, "{arg}");
    }
}

#[test]
fn an_r_is_the_modules_for_a_player_that_has_it() {
    let wave: Vec<i16> = (0..2000).map(|i| ((i % 64) * 256 - 8000) as i16).collect();
    let made = |cells: &[[u8; 5]], target: Target| {
        let source = Source { data: module_of_16_bits(&wave, (0, 2000), cells), file: "test.xm".into(), options: vec![("samples".into(), "8".into())], label: None };
        convert_for(&source, target).unwrap()
    };
    let has = |clip: &Clip, cell: [u8; 4]| clip.data.windows(4).any(|w| w == cell);
    let said = |clip: &Clip| clip.notes.iter().any(|n| n.contains("effect R"));
    // One that changes no volume: R as it is, or E9 for v0.5; no
    // note of either.
    let plain = [[49, 1, 0, 0x1B, 0x83]];
    let (new, old) = (made(&plain, Target::default()), made(&plain, OLD));
    assert!(has(&new, [1, 0, 0x1B, 0x83]) && !has(&new, [1, 0, 0xE, 0x93]) && !said(&new));
    assert!(has(&old, [1, 0, 0xE, 0x93]) && !has(&old, [1, 0, 0x1B, 0x83]) && !said(&old));
    // One that does: the same cell in both, played by one and named
    // as not played for the other.
    let loud = [[49, 1, 0, 0x1B, 0x43]];
    let (new, old) = (made(&loud, Target::default()), made(&loud, OLD));
    assert!(has(&new, [1, 0, 0x1B, 0x43]) && has(&old, [1, 0, 0x1B, 0x43]));
    assert!(!said(&new) && said(&old), "{:?} {:?}", new.notes, old.notes);
    // The global volume slide, H, came with it: named for v0.5 alone.
    let fade = [[49, 1, 0, 0x10, 0], [0, 0, 0, 0x11, 0x40]];
    let (new, old) = (made(&fade, Target::default()), made(&fade, OLD));
    let said = |clip: &Clip| clip.notes.iter().any(|n| n.contains("effect H"));
    assert!(!said(&new) && said(&old), "{:?} {:?}", new.notes, old.notes);
    assert!(play(&new.data).iter().any(|&s| s != 0));
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
fn a_module_that_ends_before_its_instruments_do_is_converted() {
    let wave: Vec<i16> = (0..3000).map(|i| (20000.0 * (i as f64 * 0.11).sin()) as i16).collect();
    let whole = module_of_16_bits(&wave, (2000, 1000), &[[49, 1, 0, 0, 0], [0, 0, 0, 0, 0], [61, 1, 0, 0, 0]]);
    let convert = |data: &[u8]| convert(&Source { data: data.to_vec(), file: "test.xm".into(), options: vec![("samples".into(), "16".into())], label: None }, Output::Speaker);
    let clip = convert(&whole).unwrap();
    let sound = play(&clip.data);
    // The header counts instruments the file has not: they have no
    // sample, the tune is the same, and it is said.
    let mut counted = whole.clone();
    counted[72] = 4;
    let more = convert(&counted).unwrap();
    assert_eq!(play(&more.data), sound);
    assert!(more.notes.iter().any(|n| n == "the file ends before instrument 2 of 4: it and those after have no sample"), "{:?}", more.notes);
    assert!(!clip.notes.iter().any(|n| n.contains("the file ends")), "{:?}", clip.notes);
    let instrument = whole.len() - 2 * wave.len() - 40 - 263;
    // What the instrument says of a sample's header is not asked.
    let mut says = whole.clone();
    says[instrument + 29] = 18;
    assert_eq!(convert(&says).unwrap().data, clip.data);
    // Cut anywhere from its first instrument on, it is a module with
    // less in it. The loop's end gone, the sample is as long as its
    // bytes.
    for cut in (instrument..whole.len()).step_by(7) {
        let less = convert(&whole[..cut]).unwrap_or_else(|why| panic!("cut at {cut}: {why}"));
        play(&less.data);
    }
    assert_ne!(play(&convert(&whole[..whole.len() - 300]).unwrap().data), sound);
    // Cut in a pattern's header it is refused, as before.
    assert_eq!(convert(&whole[..340]).unwrap_err(), "test.xm: the file is cut short");
}

#[test]
fn a_16_bit_sample_is_predicted_with_seek_points() {
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
    // A byte more a sample, and one before them for an even offset.
    assert_eq!(wide.data.len(), cut.data.len() + wave.len() + (cut.data.len() - wave.len()) % 2);
    // Its seek points with it, six bytes for each 256 samples but the
    // first, whatever the module does with the instrument.
    let points = onebitdawn_core::xm::seek_points(wave.len()) * onebitdawn_core::xm::SEEK_BYTES;
    assert_eq!(packed.data.len(), cut.data.len() - wave.len() + PACKED_HEAD + points + wave.len().div_ceil(BLOCK) * BLOCK_BYTES);
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
    // sample is predicted all the same, and the note that starts 512
    // samples in plays as the 16 bits' does.
    for offset in [[49, 1, 0, 9, 2], [49, 0, 0, 9, 2]] {
        let cells = [note, rest, offset, rest];
        let (sought, as_it_is) = (made(&cells, ""), made(&cells, "samples=16"));
        assert_eq!((&sought.data[..4], &as_it_is.data[..4]), (&b"XMC4"[..], &b"XMC3"[..]));
        assert_eq!(sought.data.len(), packed.data.len());
        assert_eq!(sought.notes.len(), 1, "{:?}", sought.notes);
        let (a, b) = (play(&as_it_is), play(&sought));
        assert_eq!(a.len(), b.len());
        let signal: f64 = a.iter().map(|s| s * s).sum();
        let noise: f64 = a.iter().zip(&b).map(|(a, b)| (a - b) * (a - b)).sum();
        let db = 10.0 * (signal / noise).log10();
        // Limiter and all, as above; that the seek points give the
        // decoder's own samples is the player's test.
        assert!(db > 40.0, "a note from an offset: {db:.1} dB");
    }
    // A sample too short to pay for its table of predictors is kept
    // at 16 bits, the smaller and exact: 64 samples are 128 bytes so,
    // and 8 + 90 + 4 * 17 predicted.
    let short = (30, 34);
    let tiny = |options: &str| {
        let source = Source { data: module_of_16_bits(&wave[..64], short, &cells), file: "test.xm".into(), options: Spec::parse(&format!("test.xm:{options}")).options, label: None };
        convert(&source, Output::Speaker).unwrap()
    };
    assert_eq!(&tiny("").data[..4], b"XMC3");
    assert_eq!(tiny("").data, tiny("samples=16").data);
    assert_eq!(&tiny("samples=8").data[..4], b"XMC2");
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
    // For v0.5 it is cut to 8 bits, as `samples=8` cuts it, and an
    // option that keeps the 16 is refused.
    let cut = Source { options: vec![("samples".into(), "8".into())], ..source.clone() };
    let old = convert_for(&source, OLD).unwrap();
    assert!(old.data.starts_with(b"XMC2") && old.data == convert(&cut, Output::Speaker).unwrap().data);
    assert_eq!(convert_for(&cut, OLD).unwrap().data, old.data);
    for asked in ["16", "packed"] {
        let keep = Source { options: vec![("samples".into(), asked.into())], ..source.clone() };
        assert_eq!(
            convert_for(&keep, OLD).err(),
            Some(format!("test.xm: samples={asked} makes a clip for a firmware after v0.5, and the pack is for v0.5"))
        );
    }
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
    // For v0.5 a sound file is ADPCM unless it is asked for in 8
    // bits, and never 16.
    let ring = |options: &str| Source { file: "ring.wav".into(), data: wav(44100), options: Spec::parse(&format!("ring.wav:{options}")).options, label: None };
    let old = convert_for(&ring("seconds=2"), OLD).unwrap();
    assert!(old.data.starts_with(b"ADP1") && old.data == convert(&ring("seconds=2,format=adpcm"), Output::Speaker).unwrap().data);
    assert!(convert_for(&ring("seconds=2,format=pcm"), OLD).unwrap().data.starts_with(b"PCM1"));
    for asked in ["packed", "pcm16"] {
        let why = convert_for(&ring(&format!("format={asked}")), OLD).unwrap_err();
        assert_eq!(why, format!("ring.wav: format={asked} makes a clip for a firmware after v0.5, and the pack is for v0.5"));
    }
    let cut = Source { file: "ring.wav".into(), data: wav(8000)[..60].to_vec(), ..Default::default() };
    assert!(convert(&cut, Output::Buzzer).is_err());
}

/// A drive over 127 has samples under the knee at full scale too, and
/// the knee fitted is over those as well: what the note says the
/// limiter flattens is what the clock's player flattens, and within
/// what was allowed wherever a knee can keep to it.
#[test]
fn the_knee_fitted_counts_the_drive() {
    let flat = |clip: &onebitdawn_pack::Clip| {
        let mut tune = onebitdawn_core::xm::Tune::new(&clip.data);
        std::iter::from_fn(|| tune.next_sample()).filter(|s| s.unsigned_abs() >= 127 * 256).count()
    };
    let said = |clip: &onebitdawn_pack::Clip| -> usize {
        let note = &clip.notes[0];
        let from = note.find("flattens ").unwrap() + 9;
        note[from..].split(' ').next().unwrap().parse().unwrap()
    };
    let mut knees = Vec::new();
    for drive in [100, 127, 128, 160, 255] {
        let clip = made(&format!("mods/Orion-Nero6.3.0.2.xm:name=keygen,knee=auto,tops=0,drive={drive}"), Output::Speaker);
        assert_eq!((flat(&clip), said(&clip)), (0, 0), "drive {drive}: {}", clip.notes[0]);
        assert!(!clip.notes[0].contains("no knee"), "{}", clip.notes[0]);
        knees.push(clip.data[29]);
    }
    // Under 127 nothing is flat at any knee; over it the knee goes up by the drive.
    assert_eq!(knees, [1, 98, 99, 124, 197]);
    // A budget, with the drive at its most: the count is the player's.
    let clip = made("mods/Orion-Nero6.3.0.2.xm:name=keygen,knee=auto,drive=255", Output::Speaker);
    assert_eq!(flat(&clip), said(&clip), "{}", clip.notes[0]);
    // 57.6 s of it at 60 a minute.
    assert!(said(&clip) <= 57, "{}", clip.notes[0]);
    // Where the knee would be over 255 the note says the budget is not kept.
    // (the chip Mountain King's mix peaks at 143, which that drive
    // has at 288).
    let clip = made("mods/grieg-mountain-king-chip.xm:knee=auto,tops=0,drive=255", Output::Speaker);
    assert_eq!((clip.data[29], flat(&clip), said(&clip)), (255, 9, 9), "{}", clip.notes[0]);
    assert!(clip.notes[0].contains("which no knee keeps to at this drive"), "{}", clip.notes[0]);
}

/// `knee=auto` finds the knees the two piano tunes were given by ear
/// on a speaker, or next to them: 48 was "good" for Starlight & Aurora
/// and 12 for Suspended Seconds, both with the limiter letting go over
/// a second.
#[test]
fn the_knee_fitted_is_what_the_ear_chose() {
    for (file, knee) in [("mods/starlight-aurora.xm", 46), ("mods/suspended-seconds.xm", 14)] {
        let fitted = made(&format!("{file}:knee=auto,release=13"), Output::Speaker);
        assert_eq!(fitted.data[29], knee, "{file}");
        assert!(fitted.notes[0].starts_with(&format!("knee=auto: knee={knee}, ")), "{}", fitted.notes[0]);
        // The clip is the one that knee asked for by its number makes.
        assert!(fitted.data == made(&format!("{file}:knee={knee},release=13"), Output::Speaker).data);
        // With nothing allowed the limiter is off: the knee is over the peak.
        let off = made(&format!("{file}:knee=auto,tops=0,release=13"), Output::Speaker);
        assert!(off.notes[0].contains("flattens 0 samples") && off.data[29] > knee, "{}", off.notes[0]);
    }
    // The limiter's choice is a `tops` for a speaker's module that
    // gives no knee of its own, and nothing to a buzzer's.
    let chosen = |arg: &str, output, limiter| {
        let spec = Spec::parse(arg);
        let source = Source { data: repo(&spec.file), file: spec.file, options: spec.options, label: spec.label };
        convert_for(&source, Target { output, limiter, ..Default::default() }).unwrap().data
    };
    let keygen = "mods/Orion-Nero6.3.0.2.xm:name=keygen";
    let knees: Vec<u8> = Limiter::ALL.iter().map(|&(limiter, _)| chosen(keygen, Output::Speaker, limiter)[29]).collect();
    assert_eq!(knees, [0, 50, 60, 76]);
    // The maker's own default is the cleanest.
    assert_eq!(chosen(keygen, Output::Speaker, Limiter::default())[29], 76);
    assert!(chosen(keygen, Output::Speaker, Limiter::Cleaner) == made(&format!("{keygen},knee=auto,tops=600"), Output::Speaker).data);
    assert!(chosen(keygen, Output::Buzzer, Limiter::Cleanest) == made(keygen, Output::Buzzer).data);
    assert_eq!(chosen(&format!("{keygen},knee=12"), Output::Speaker, Limiter::Cleanest)[29], 12);
    assert_eq!(chosen(&format!("{keygen},tops=3000"), Output::Speaker, Limiter::Cleanest)[29], 41);
    // A PT3 module's knee is one of fifteen steps, and fitted it comes
    // with the player's own drive unless the options give one.
    let sync = "mods/nq-synchronization.pt3:restart,name=synchronization";
    let steps: Vec<[u8; 2]> = Limiter::ALL.iter().map(|&(limiter, _)| chosen(sync, Output::Speaker, limiter)).map(|clip| [clip[4] >> 4, clip[5]]).collect();
    // 37 and 150 as they were; then 58, 74 and 93.
    assert_eq!(steps, [[11, 150], [13, 0], [14, 0], [15, 0]]);
    assert!(chosen(sync, Output::Speaker, Limiter::Loudest) == made(sync, Output::Speaker).data);
    assert!(chosen(sync, Output::Speaker, Limiter::Cleaner) == made(&format!("{sync},knee=auto,tops=600"), Output::Speaker).data);
    assert!(chosen(sync, Output::Buzzer, Limiter::Cleanest) == made(sync, Output::Buzzer).data);
    let kept = chosen(&format!("{sync},knee=23,drive=140"), Output::Speaker, Limiter::Cleanest);
    assert_eq!([kept[4] >> 4, kept[5]], [9, 140]);
    assert_eq!(chosen(&format!("{sync},flat"), Output::Speaker, Limiter::Cleanest)[4] >> 4, 0);
    let refused = |options: &str| {
        let spec = Spec::parse(&format!("mods/suspended-seconds.xm:{options}"));
        convert(&Source { data: repo(&spec.file), file: spec.file, options: spec.options, label: None }, Output::Speaker).unwrap_err()
    };
    assert_eq!(refused("tops=60"), "mods/suspended-seconds.xm: tops is knee=auto's");
    assert_eq!(refused("knee=low"), "mods/suspended-seconds.xm: knee=low: a number to 255, or auto");
    let source = Source { file: "sync.pt3".into(), data: repo("mods/nq-synchronization.pt3"), options: Spec::parse("a.pt3:flat,knee=auto").options, label: None };
    assert_eq!(convert(&source, Output::Speaker).unwrap_err(), "sync.pt3: knee is the limiter's: not with flat");
}
