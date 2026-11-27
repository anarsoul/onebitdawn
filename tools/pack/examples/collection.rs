// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The pack maker and the clock's module player over a collection of
//! tracker modules kept outside the repository: every file is
//! converted, for the buzzer and for a speaker, and its clip played to
//! the end by `onebitdawn_core::xm::Tune`. What is asked is what
//! `tests/local.rs` asks of the files at hand, of thousands: a file
//! converts or is refused in words, never a panic, and its clip ends.
//!
//!     cargo collection [--firmware 0.5|0.6] [--raw DIR] [--list FILE] path…
//!
//! A path is a file or a folder, searched to its bottom for `.xm`,
//! `.mod` and `.s3m`; `--list` names a file of paths, one a line. The
//! report is a line for each file worth one (a panic, a tune that does
//! not end, a silent one), then the totals, why files were refused and
//! what the modules use that the player lacks, each with the number of
//! files. It exits with 1 if anything panicked or did not end.
//!
//! With `--raw DIR` the clips are made for a speaker with the limiter
//! off (`knee=64`), so that loudness is the tune's, and each one's
//! sound is written to `DIR/<n>.raw` (16 bits, mono, the clip's rate)
//! with a line in `DIR/index.tsv`: the number, the rate, the length in
//! milliseconds the clip's header gives, the module's path. That is
//! what `scripts/openmpt-compare.py` reads. `agents/collection.md` has
//! the rest.

use std::collections::BTreeMap;
use std::io::Write;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};

use onebitdawn_core::{clip, xm};
use onebitdawn_pack::{Firmware, Output, Source, Target, convert_for};

/// A tune longer than this is taken for one that does not end.
const MINUTES: usize = 30;

fn modules(path: &Path, into: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path).into_iter().flatten().filter_map(Result::ok).map(|e| e.path()).collect();
        entries.sort();
        for entry in entries {
            modules(&entry, into);
        }
    } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| ["xm", "mod", "s3m"].contains(&e.to_ascii_lowercase().as_str())) {
        into.push(path.to_path_buf());
    }
}

/// A message with its numbers taken out, so that the same refusal or
/// note of many files is one line: a word that is a number, with a
/// bracket or a colon by it or not. `E6` and `6x` are names.
fn kind(text: &str) -> String {
    let word = |w: &str| {
        let core = w.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if !core.is_empty() && core.bytes().all(|b| b.is_ascii_digit()) { w.replace(core, "#") } else { w.to_string() }
    };
    text.split(' ').map(word).collect::<Vec<_>>().join(" ")
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut firmware, mut raw, mut files) = (Firmware::default(), None::<PathBuf>, Vec::new());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--firmware" => firmware = args.next().as_deref().and_then(Firmware::parse).expect("--firmware 0.5 or 0.6"),
            "--raw" => raw = Some(args.next().expect("--raw DIR").into()),
            "--list" => {
                let list = std::fs::read_to_string(args.next().expect("--list FILE")).expect("the list");
                files.extend(list.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
            }
            path => modules(Path::new(path), &mut files),
        }
    }
    if files.is_empty() {
        eprintln!("usage: cargo collection [--firmware 0.5|0.6] [--raw DIR] [--list FILE] path…");
        std::process::exit(2);
    }
    let mut index = raw.as_ref().map(|dir| {
        std::fs::create_dir_all(dir).expect("the folder for --raw");
        std::fs::File::create(dir.join("index.tsv")).expect("index.tsv")
    });
    // The panics are counted here; their own words go to stderr.
    let (mut made, mut refused, mut bad, mut silent, mut seconds) = (0, 0, 0, 0, 0.0);
    let (mut why_refused, mut lacks) = (BTreeMap::<String, usize>::new(), BTreeMap::<String, usize>::new());
    for (n, path) in files.iter().enumerate() {
        let name = path.display();
        let Ok(data) = std::fs::read(path) else {
            println!("not read: {name}");
            bad += 1;
            continue;
        };
        let file = format!("m.{}", path.extension().and_then(|e| e.to_str()).unwrap_or("xm").to_ascii_lowercase());
        // With --raw the speaker's clip alone, its limiter off.
        let outputs: &[Output] = if raw.is_some() { &[Output::Speaker] } else { &[Output::Buzzer, Output::Speaker] };
        for (o, &output) in outputs.iter().enumerate() {
            let mut options = vec![("name".to_string(), "local".to_string())];
            if raw.is_some() {
                options.push(("knee".into(), "64".into()));
            }
            let source = Source { data: data.clone(), file: file.clone(), options, label: None };
            let clip = match catch_unwind(|| convert_for(&source, Target { output, firmware, ..Default::default() })) {
                Err(_) => {
                    println!("PANIC converting: {name}");
                    bad += 1;
                    break;
                }
                Ok(Err(why)) => {
                    *why_refused.entry(kind(why.trim_start_matches(&format!("{file}: ")))).or_default() += 1;
                    refused += 1;
                    break;
                }
                Ok(Ok(clip)) => clip,
            };
            // The speaker's is converted and not played again: its
            // patterns are the buzzer's.
            if o == 1 {
                continue;
            }
            for note in clip.notes.iter().filter(|n| n.starts_with("not played") || n.starts_with("not converted") || n.starts_with("the file ends")) {
                *lacks.entry(kind(note)).or_default() += 1;
            }
            if !xm::is_tune(&clip.data) {
                println!("not a module's clip: {name}");
                bad += 1;
                continue;
            }
            let keep = raw.is_some();
            let bytes = clip.data.clone();
            let played = catch_unwind(move || {
                let mut tune = xm::Tune::new(&bytes);
                let rate = tune.rate();
                let cap = rate as usize * 60 * MINUTES;
                let (mut count, mut peak, mut sound) = (0usize, 0i32, Vec::new());
                while let Some(s) = tune.next_sample() {
                    count += 1;
                    peak = peak.max((s as i32).abs());
                    if keep {
                        sound.extend(s.to_le_bytes());
                    }
                    if count > cap {
                        return Err(());
                    }
                }
                Ok((count, peak, sound, rate))
            });
            match played {
                Err(_) => {
                    println!("PANIC playing: {name}");
                    bad += 1;
                }
                Ok(Err(())) => {
                    println!("does not end in {MINUTES} minutes: {name}");
                    bad += 1;
                }
                Ok(Ok((count, peak, sound, rate))) => {
                    if count == 0 || peak == 0 {
                        println!("silent ({count} samples): {name}");
                        silent += 1;
                    }
                    let ms = clip::duration_ms(&clip.data);
                    seconds += ms as f64 / 1000.0;
                    made += 1;
                    if let (Some(dir), Some(index)) = (&raw, &mut index) {
                        std::fs::write(dir.join(format!("{n}.raw")), sound).expect("the sound");
                        writeln!(index, "{n}\t{rate}\t{ms}\t{name}").expect("index.tsv");
                    }
                }
            }
        }
    }
    println!(
        "{} files for {}: {made} converted and played to the end ({:.1} hours), {silent} of them silent, {refused} refused, {bad} bad",
        files.len(),
        firmware.text(),
        seconds / 3600.0
    );
    for (why, n) in &why_refused {
        println!("  refused {n}: {why}");
    }
    for (what, n) in &lacks {
        println!("  in {n}: {what}");
    }
    if bad != 0 {
        std::process::exit(1);
    }
}
