// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The clock's PT3 player against zxtune123, the reference, over a
//! collection of modules kept outside the repository: every file is
//! converted as the pack maker does, its clip played by
//! `onebitdawn_core::pt3::Tune` for the registers it gives the chip, a
//! frame at a time, and those are held to the registers zxtune123
//! writes for the same file (`--convert mode=psg`).
//!
//!     cargo pt3-check [--show N] [--zxtune PATH] [--list FILE] path…
//!
//! A path is a file or a folder, searched to its bottom for `.pt3`;
//! `--list` names a file of paths, one a line. The report is a line
//! for each file that differs: the frames of each side, how many
//! frames differ, the first of them, and the registers that differ
//! with the number of frames; then the totals. `--show N` adds the
//! first N differing frames of each such file, the clock's registers
//! over zxtune's, with the position and the frame within it. It exits
//! with 1 if a file differs.
//!
//! What is compared is what the chip would do with the registers, so
//! the bits it has no use for are left out (a tone's top four, the
//! mixer's ports), and the shape register counts as written or not:
//! a write of the same shape starts the envelope over. zxtune writes
//! a PSG of the changes alone, but the shape whenever the player
//! writes it. `agents/collection.md` has the rest.

use std::collections::BTreeMap;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::Command;

use onebitdawn_core::pt3;
use onebitdawn_pack::{Output, Source, Target, convert_for};

/// A tune longer than this is taken for one that does not end.
const FRAMES: usize = 50 * 60 * 30;

/// The bits of each register the chip reads.
const MASK: [u8; 14] = [0xFF, 0x0F, 0xFF, 0x0F, 0xFF, 0x0F, 0x1F, 0x3F, 0x1F, 0x1F, 0x1F, 0xFF, 0xFF, 0xFF];

fn modules(path: &Path, into: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path).into_iter().flatten().filter_map(Result::ok).map(|e| e.path()).collect();
        entries.sort();
        for entry in entries {
            modules(&entry, into);
        }
    } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("pt3")) {
        into.push(path.to_path_buf());
    }
}

/// A PSG record as registers frame by frame: `FF` is the end of a
/// frame (the record begins with one, before the first), `FE n` the
/// ends of four times n frames, `FD` the record's end, and anything
/// else a register with its value, for the frame that follows the
/// last end. The shape is 255 in a frame that does not write it.
fn psg(d: &[u8]) -> Option<Vec<[u8; 14]>> {
    if !d.starts_with(b"PSG\x1a") {
        return None;
    }
    let (mut out, mut regs, mut begun) = (Vec::new(), [0u8; 14], false);
    regs[13] = 0xFF;
    let mut at = 16;
    while at < d.len() {
        let ends = match d[at] {
            0xFF => 1,
            0xFE => 4 * *d.get(at + 1)? as usize,
            0xFD => break,
            register => {
                if let Some(slot) = regs.get_mut(register as usize) {
                    *slot = *d.get(at + 1)?;
                }
                at += 2;
                continue;
            }
        };
        at += if d[at] == 0xFE { 2 } else { 1 };
        // The first end has no frame before it.
        for _ in 0..ends - (!begun) as usize {
            out.push(regs);
            regs[13] = 0xFF;
        }
        begun = true;
    }
    out.push(regs);
    Some(out)
}

/// zxtune's registers for a module, through a copy under a plain name:
/// the file's own may have what the option's map takes for its own.
fn reference(zxtune: &str, dir: &Path, data: &[u8]) -> Result<Vec<[u8; 14]>, String> {
    let (module, record) = (dir.join("m.pt3"), dir.join("m.psg"));
    let _ = std::fs::remove_file(&record);
    std::fs::write(&module, data).map_err(|e| e.to_string())?;
    let run = Command::new(zxtune).current_dir(dir).args(["--convert", "mode=psg,filename=m.psg", "m.pt3"]).output().map_err(|e| format!("{zxtune}: {e}"))?;
    let record = std::fs::read(&record).map_err(|_| format!("zxtune made no record: {}", String::from_utf8_lossy(&run.stderr).trim()))?;
    psg(&record).ok_or_else(|| "zxtune's record is not a PSG".into())
}

fn masked(frame: &[u8; 14]) -> [u8; 14] {
    let mut out: [u8; 14] = core::array::from_fn(|r| frame[r] & MASK[r]);
    if frame[13] != 0xFF {
        out[13] &= 0x0F;
    }
    out
}

fn hex(frame: &[u8; 14]) -> String {
    frame.iter().enumerate().map(|(r, v)| if r == 13 && *v == 0xFF { "--".into() } else { format!("{v:02X}") }).collect::<Vec<_>>().join(" ")
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut show, mut zxtune, mut files) = (0usize, "zxtune123".to_string(), Vec::new());
    let mut from = None::<usize>;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--show" => show = args.next().and_then(|n| n.parse().ok()).expect("--show N"),
            "--from" => from = Some(args.next().and_then(|n| n.parse().ok()).expect("--from FRAME")),
            "--zxtune" => zxtune = args.next().expect("--zxtune PATH"),
            "--list" => {
                let list = std::fs::read_to_string(args.next().expect("--list FILE")).expect("the list");
                files.extend(list.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
            }
            path => modules(Path::new(path), &mut files),
        }
    }
    if files.is_empty() {
        eprintln!("usage: cargo pt3-check [--show N] [--zxtune PATH] [--list FILE] path…");
        std::process::exit(2);
    }
    let dir = std::env::temp_dir().join(format!("pt3check-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder for zxtune's records");
    let (mut same, mut differ, mut refused, mut bad, mut frames, mut short) = (0, 0, 0, 0, 0usize, 0usize);
    let (mut why_refused, mut by_register) = (BTreeMap::<String, usize>::new(), [0usize; 14]);
    for path in &files {
        let name = path.display();
        let Ok(data) = std::fs::read(path) else {
            println!("not read: {name}");
            bad += 1;
            continue;
        };
        let source = Source { data: data.clone(), file: "m.pt3".into(), options: vec![("name".into(), "local".into())], label: None };
        let clip = match catch_unwind(|| convert_for(&source, Target { output: Output::Speaker, ..Default::default() })) {
            Err(_) => {
                println!("PANIC converting: {name}");
                bad += 1;
                continue;
            }
            Ok(Err(why)) => {
                *why_refused.entry(why.trim_start_matches("m.pt3: ").to_string()).or_default() += 1;
                refused += 1;
                continue;
            }
            Ok(Ok(clip)) => clip,
        };
        let bytes = clip.data.clone();
        let ours = catch_unwind(move || {
            let mut tune = pt3::Tune::new(&bytes);
            let (mut out, mut places) = (Vec::new(), Vec::new());
            while let Some(frame) = tune.frame_registers() {
                out.push(frame);
                places.push(tune.place());
                if out.len() > FRAMES {
                    return None;
                }
            }
            Some((out, places))
        });
        let (ours, places) = match ours {
            Err(_) => {
                println!("PANIC playing: {name}");
                bad += 1;
                continue;
            }
            Ok(None) => {
                println!("does not end in half an hour: {name}");
                bad += 1;
                continue;
            }
            Ok(Some(both)) => both,
        };
        let theirs = match reference(&zxtune, &dir, &data) {
            Ok(theirs) => theirs,
            Err(why) => {
                println!("{why}: {name}");
                bad += 1;
                continue;
            }
        };
        frames += ours.len();
        // zxtune's record is short: it never has the last frame, loses
        // one before it now and then, and a run of frames with nothing
        // written longer than 1020 by 1024 or a multiple (the count's
        // byte goes round). So frames the record lacks are looked for
        // where the two first differ, and are the clock's alone.
        let whole = ours.len();
        let lost = whole.saturating_sub(theirs.len());
        let at = ours.iter().zip(&theirs).position(|(a, b)| masked(a) != masked(b)).unwrap_or(theirs.len());
        let rejoined = at + lost <= ours.len() && ours[at + lost..].iter().map(masked).eq(theirs[at..].iter().map(masked));
        let ours: Vec<[u8; 14]> = if rejoined && lost != 0 { ours[..at].iter().chain(&ours[at + lost..]).copied().collect() } else { ours };
        let places: Vec<_> = if rejoined && lost != 0 { places[..at].iter().chain(&places[at + lost..]).copied().collect() } else { places };
        short += (lost > 1) as usize;
        let (mut wrong, mut first, mut registers) = (0, None, [0usize; 14]);
        for (n, (a, b)) in ours.iter().zip(&theirs).enumerate() {
            let (a, b) = (masked(a), masked(b));
            if a != b {
                wrong += 1;
                first.get_or_insert(n);
                for r in 0..14 {
                    registers[r] += (a[r] != b[r]) as usize;
                }
            }
        }
        if wrong == 0 && ours.len() == theirs.len() {
            same += 1;
            continue;
        }
        differ += 1;
        let which: Vec<_> = registers.iter().enumerate().filter(|(_, n)| **n != 0).map(|(r, n)| format!("R{r}:{n}")).collect();
        for r in 0..14 {
            by_register[r] += (registers[r] != 0) as usize;
        }
        println!("{} / {} frames, {wrong} differ from {}: {} : {name}", whole, theirs.len(), first.map_or("-".into(), |n| n.to_string()), which.join(" "));
        let mut left = show;
        for (n, (a, b)) in ours.iter().zip(&theirs).enumerate() {
            if left == 0 {
                break;
            }
            let differs = masked(a) != masked(b);
            if from.map_or(differs, |from| n >= from) {
                let mark = if differs { '*' } else { ' ' };
                let (position, at) = places[n];
                println!("{mark} {n:6} clock  {}  position {position}, the patterns at {:04X} {:04X} {:04X}\n         zxtune {}", hex(a), at[0], at[1], at[2], hex(b));
                left -= 1;
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("{} files: {same} the same as zxtune over {:.1} hours of frames, {differ} differ, {refused} refused, {bad} bad", files.len(), frames as f64 / 50.0 / 3600.0);
    if short != 0 {
        println!("  zxtune's record lacks more than the last frame in {short}");
    }
    for (r, n) in by_register.iter().enumerate().filter(|(_, n)| **n != 0) {
        println!("  R{r} differs in {n}");
    }
    for (why, n) in &why_refused {
        println!("  refused {n}: {why}");
    }
    if differ != 0 || bad != 0 {
        std::process::exit(1);
    }
}
