// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! A FastTracker 2 module (.xm) as a clip for the clock's own player
//! (`onebitdawn_core::xm`): the module rearranged for a player with no
//! memory to unpack into. Tables of where each pattern and each
//! instrument's sample is, the patterns as they are, the samples as
//! plain signed bytes (an XM stores differences), ping-pong loops
//! unrolled. One sample an instrument, the first. A sample of 16
//! bits is kept in 8 bits a sample and a sixteenth more, each sample
//! as what is left of it after the three before have predicted it
//! (`predicted`): some 25 dB nearer the 16 bits than 8 bits are, for
//! a piano, under which what 8 bits round off is heard as hiss.
//! `samples=16` keeps it as it is, two bytes a sample, and
//! `samples=8` cuts it to 8 bits. A sample of 16 bits whose low bytes
//! are all the same is one of 8 and is written so, that byte dropped,
//! whatever is asked. Each of the first two is a clip of
//! a magic of its own, which a firmware from before it refuses; a
//! module of 8-bit samples, or cut to them, is the clip there always
//! was. A sample too short for its table of predictors to pay for
//! itself is kept at 16 bits, which is then the smaller, and exact.
//! One effect is written as another: R, where it changes no volume,
//! as E9, which the player has.
//!
//! The player cuts the bass, which the buzzer does not have, and
//! limits hard; how is the clip's to say (`Shape`).

use std::collections::BTreeMap;

use onebitdawn_core::adpcm::BLOCK;
use onebitdawn_core::shape::{STEP, Shaper};
use onebitdawn_core::xm::{MAGIC, MAX_CHANNELS, PACKED_LOOP, PACKED_MAGIC, SEEK, WIDE_MAGIC, seek_points};

use crate::bytes::{Data, text};
use crate::shape::{rate_flag, section};
use crate::{Firmware, Limiter, Options, Output};

/// The effect R, by its number in a cell.
const R: u8 = 0x1B;
const ENVELOPE_POINTS: usize = 12;
/// What `xm.rs` plays: effects by number (E's by its first digit too)
/// and the volume column by its high nibble.
const EFFECTS: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x14];
const E_EFFECTS: [u8; 7] = [1, 2, 9, 0xA, 0xB, 0xC, 0xD];
const VOLUMES: [u8; 13] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xF];
/// Panning does nothing on one buzzer; not worth a note.
const SILENT: [u8; 3] = [8, 0x19, 0x1D];
const SILENT_VOLUMES: [u8; 3] = [0xC, 0xD, 0xE];

/// The options a module takes.
pub const OPTIONS: &[&str] = &["highpass", "passes", "drive", "release", "knee", "tops", "rate", "bits", "samples", "raise"];

/// The samples a minute `knee=auto` lets the limiter flatten: what
/// the two piano tunes' knees, chosen by ear on a speaker, leave
/// (`fit_knee`).
pub const TOPS: u32 = 60;

/// What becomes of a module's 16-bit samples.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Samples {
    /// Cut to 8 bits, which every firmware plays; and what a sample
    /// of 8 bits is.
    #[default]
    Cut,
    /// Kept, two bytes a sample.
    Wide,
    /// Kept in 8 bits a sample, predicted: what `Shape` asks for
    /// unless told.
    Packed,
}

/// How the player is to shape the mix.
#[derive(Clone, Debug)]
pub struct Shape {
    /// What the limiter aims a peak at, full scale being 127; 0 is the
    /// player's own.
    pub drive: u8,
    /// The limiter lets go by 1/2^N a sample at 8 kHz; 0 is the
    /// player's own, 7.
    pub release: u8,
    /// The level the limiter starts at, in 1024s of the mix where one
    /// voice at full level is 32; 0 is the player's own.
    pub knee: u8,
    /// `knee=auto`: the knee is the lowest that leaves no more than
    /// this many samples a minute flattened at full scale
    /// (`fit_knee`), and `knee` is not looked at.
    pub fit: Option<u32>,
    /// The high-pass's corner in Hz, and how many times it is applied,
    /// 12 dB an octave each.
    pub highpass: f64,
    pub passes: u8,
    /// Samples a second the player makes of it.
    pub rate: u32,
    /// The width of the samples the player hands the modulator.
    pub bits: u8,
    /// How the module's 16-bit samples are kept.
    pub samples: Samples,
    /// Instruments played octaves higher, by number from 1.
    pub raised: BTreeMap<usize, i32>,
}

impl Shape {
    /// The default pack's for the buzzer: 300 Hz once, which is what
    /// the user's ear chose for two chiptunes over the PCM clips'
    /// 900 Hz twice. For a speaker the bass is left in and the limiter
    /// lets go over 128 ms and not 16, so its gain follows the tune's
    /// level and not its waveform (`scripts/speaker-clips.sh`, heard
    /// 2026-11-15).
    pub fn new(output: Output) -> Self {
        let (highpass, release) = match output {
            Output::Buzzer => (300.0, 0),
            Output::Speaker => (100.0, 10),
        };
        Shape { drive: 0, release, knee: 0, fit: None, highpass, passes: 1, rate: 31250, bits: 16, samples: Samples::Packed, raised: BTreeMap::new() }
    }

    /// A firmware that plays no samples of 16 bits has them cut to 8,
    /// and an option that asks for them is refused.
    ///
    /// A speaker's limiter is as `limiter` says unless the options
    /// give the knee; a buzzer's is the player's own, whatever it says.
    pub fn options(output: Output, firmware: Firmware, limiter: Limiter, options: &mut Options) -> Result<Self, String> {
        let mut shape = Shape::new(output);
        shape.drive = options.number("drive", shape.drive)?;
        shape.release = options.number("release", shape.release)?;
        let tops = options.maybe("tops")?;
        if output == Output::Speaker {
            shape.fit = limiter.tops();
        }
        match options.text("knee")?.as_deref() {
            None => {}
            Some("auto") => shape.fit = Some(shape.fit.unwrap_or(TOPS)),
            Some(v) => (shape.knee, shape.fit) = (v.parse().map_err(|_| format!("knee={v}: a number to 255, or auto"))?, None),
        }
        match (tops, shape.fit) {
            (Some(tops), Some(_)) => shape.fit = Some(tops),
            (Some(_), None) => return Err("tops is knee=auto's".into()),
            (None, _) => {}
        }
        shape.highpass = options.number("highpass", shape.highpass)?;
        shape.passes = options.number("passes", shape.passes)?;
        shape.rate = options.number("rate", shape.rate)?;
        shape.bits = options.number("bits", shape.bits)?;
        shape.samples = match (options.text("samples")?.as_deref(), firmware.wide()) {
            (None, false) | (Some("8"), _) => Samples::Cut,
            (None | Some("packed"), true) => Samples::Packed,
            (Some("16"), true) => Samples::Wide,
            (Some(asked @ ("16" | "packed")), false) => return Err(firmware.refuses(&format!("samples={asked}"))),
            (Some(_), _) => return Err("samples is 8, 16 or packed".into()),
        };
        while let Some(item) = options.text("raise")? {
            let parsed = item.split_once(':').and_then(|(i, o)| Some((i.parse().ok()?, o.parse().ok()?)));
            let (instrument, octaves) = parsed.ok_or_else(|| format!("raise={item}: raise=INSTRUMENT:OCTAVES"))?;
            shape.raised.insert(instrument, octaves);
        }
        if shape.release > 15 {
            return Err("release is 1 to 15".into());
        }
        if shape.passes > 2 {
            return Err("passes is 0, 1 or 2".into());
        }
        if !matches!(shape.bits, 8 | 16) {
            return Err("bits is 8 or 16".into());
        }
        Ok(shape)
    }
}

/// The cells of a pattern's packed data, a row at a time: note,
/// instrument, volume, effect, parameter.
fn rows(data: &[u8], channels: usize) -> Vec<Vec<[u8; 5]>> {
    let (mut q, mut row, mut out) = (0, Vec::with_capacity(channels), Vec::new());
    let at = |q: usize| data.get(q).copied().unwrap_or(0);
    while q < data.len() {
        let b = data[q];
        q += 1;
        let mut cell = [0u8; 5];
        if b & 0x80 != 0 {
            for (i, c) in cell.iter_mut().enumerate() {
                if b & (1 << i) != 0 {
                    *c = at(q);
                    q += 1;
                }
            }
        } else {
            cell = [b, at(q), at(q + 1), at(q + 2), at(q + 3)];
            q += 4;
        }
        row.push(cell);
        if row.len() == channels {
            out.push(core::mem::replace(&mut row, Vec::with_capacity(channels)));
        }
    }
    out
}

/// FastTracker's R, the retrigger with a volume change, is E9, which
/// the player has, where the volume is left alone (x of 8, or of 0 on
/// a channel no other R gives a volume to remember) and the ticks are
/// said (y; none is the last R's). Written over the packed cells of a
/// pattern, which is no longer for it; `plain` is which channels have
/// no R with a volume.
fn retriggers(packed: &mut [u8], channels: usize, plain: &[bool]) {
    let (mut q, mut channel) = (0, 0);
    while q < packed.len() {
        let b = packed[q];
        q += 1;
        // Where the cell's effect and parameter are, if it has both.
        let at = if b & 0x80 != 0 {
            let before = |bit: u8| (b & (bit - 1)).count_ones() as usize;
            let at = (b & 0x18 == 0x18).then(|| (q + before(8), q + before(16)));
            q += (b & 0x1F).count_ones() as usize;
            at
        } else {
            q += 4;
            Some((q - 2, q - 1))
        };
        if let Some((effect, param)) = at.filter(|&(_, param)| param < packed.len())
            && packed[effect] == R
            && packed[param] & 0xF != 0
            && (packed[param] >> 4 == 8 || (packed[param] >> 4 == 0 && plain[channel]))
        {
            packed[effect] = 0xE;
            packed[param] = 0x90 | (packed[param] & 0xF);
        }
        channel = (channel + 1) % channels;
    }
}

fn effect_name(effect: u8, param: u8) -> String {
    let letter = if effect < 16 { format!("{effect:X}") } else { ((b'G' + effect - 16) as char).to_string() };
    if effect == 0xE { format!("{letter}{:X}", param >> 4) } else { letter }
}

/// A 16-bit sample as the player's predicted one (`adpcm::predicted`):
/// what the loop needs, the table of predictors fitted to the sample,
/// then its seek points and its blocks. A seek point is, for each
/// `SEEK` samples but the first, the three samples before that block
/// as the player decodes them, the newest first: a note a 9xx starts
/// far into the sample has nothing before it to decode, and a channel
/// that plays on with no level (its envelope at nothing, a fade run
/// out) is not decoded at all until it is heard again, which costs a
/// few blocks then. Every predicted sample has them, 2 % more; until
/// 2026-11-23 only one a 9xx played from an offset had.
pub fn predicted(sample: &[i16], looped: (usize, usize)) -> Vec<u8> {
    let sound = crate::adpcm::predicted(sample);
    // For the loop: the three samples before the block it starts in,
    // the newest first, and its first sample, as the player decodes
    // them.
    let mut head = [0i16; PACKED_LOOP / 2];
    if looped.1 != 0 && looped.0 < sound.decoded.len() {
        let first = looped.0 / BLOCK * BLOCK;
        for (n, h) in head[..3].iter_mut().enumerate() {
            *h = if first > n { sound.decoded[first - 1 - n] } else { 0 };
        }
        head[3] = sound.decoded[looped.0];
    }
    let mut out: Vec<u8> = head.iter().flat_map(|h| h.to_le_bytes()).collect();
    out.extend(sound.table);
    for k in 1..=seek_points(sample.len()) {
        out.extend((1..=3).flat_map(|n| sound.decoded[k * SEEK - n].to_le_bytes()));
    }
    out.extend(sound.blocks);
    out
}

#[derive(Default)]
struct Instrument {
    /// The sample at 16 bits, and how it is kept: one cut to 8 bits is
    /// written as its high bytes.
    sample: Vec<i16>,
    kept: Samples,
    /// The sample predicted, if that is how it is kept.
    packed: Vec<u8>,
    looped: (usize, usize),
    volume: u8,
    finetune: i8,
    relative: i8,
    flags: u8,
    points: Vec<(u16, u16)>,
    sustain: u8,
    env_loop: (u8, u8),
    fade: u16,
    auto: [u8; 3],
}

/// Where a clip's header has the limiter's drive, its flags and the
/// knee (`xm`'s header).
const DRIVE_AT: usize = 10;
const FLAGS_AT: usize = 17;
const KNEE_AT: usize = 29;

/// The knee for a clip made with any: the lowest that has the limiter
/// flatten no more than `tops` samples a minute, with the mix's peak
/// and how many it then flattens.
///
/// The limiter's attack is at once, so a sample over the envelope as
/// it has decayed comes out at full scale, and a run of them is a
/// flat top on a rising wave: a click where one instrument is heard
/// alone. Which samples those are is the envelope's and the
/// release's to say and not the knee's; the knee only lets the ones
/// under it by. So the tune is played once here with the limiter off
/// and its samples fine (a knee of 255 and a drive of 127 give the mix
/// after the high-pass, an eighth of it), the clip's own shaper is run
/// over that with the knee at its least, and the knee is put over all
/// but `tops` a minute of the levels it flattened.
///
/// Over them as the drive has them: under the knee a sample comes out
/// as its level times the drive over the knee, at full scale when
/// that is 127, so a drive over 127 has the samples within its reach
/// of the knee flat as well and the knee goes that much higher. Where
/// it would go over 255 it stays there, and the count, which is then
/// over what `allowed` (the last of the four) is, says so.
fn fit_knee(clip: &[u8], tops: u32) -> (u8, i32, usize, usize) {
    let mut plain = clip.to_vec();
    (plain[DRIVE_AT], plain[KNEE_AT], plain[FLAGS_AT]) = (127, 255, plain[FLAGS_AT] | 2);
    let mut tune = onebitdawn_core::xm::Tune::new(&plain);
    let rate = tune.rate();
    // 31250, 16000 or 8000: how many times twice the last, as the player counts.
    let octaves = match rate {
        r if r > 24000 => 2,
        r if r > 12000 => 1,
        _ => 0,
    };
    let full = 127 * STEP;
    let mut limiter = Shaper::new(clip[DRIVE_AT], clip[DRIVE_AT + 1], octaves, [0; 5], 0, true, 1);
    let (mut flat, mut peak, mut count) = (Vec::new(), 0, 0u64);
    while let Some(sample) = tune.next_sample() {
        let level = (sample as i64 * 255 * 1024 / full as i64).abs() as i32;
        peak = peak.max(level);
        count += 1;
        if limiter.run(level).abs() as i32 >= full {
            flat.push(level);
        }
    }
    flat.sort_unstable();
    let allowed = (count * tops as u64 / (rate as u64 * 60)) as usize;
    // The knee over the highest of the levels that may not be flat.
    // A sample the shaper flattened with the knee at its least is flat
    // at a knee too if its level times the drive is 127 knees still.
    let drive = match clip[DRIVE_AT] {
        0 => 127,
        d => d as i64,
    };
    let knee = match flat.len().checked_sub(allowed + 1) {
        Some(at) => (flat[at] as i64 * drive / (127 * 1024) + 1).clamp(1, 255),
        None => 1,
    };
    let flattened = flat.iter().filter(|&&level| level as i64 * drive >= 127 * 1024 * knee).count();
    (knee as u8, peak, flattened, allowed)
}

/// The clip of the module in `module`, and what there is to say of it.
pub fn convert(module: &[u8], shape: &Shape, notes: &mut Vec<String>) -> Result<Vec<u8>, String> {
    let d = Data(module);
    if !module.starts_with(b"Extended Module: ") {
        return Err("not an XM module".into());
    }
    let section = section(shape.highpass, shape.rate)?;
    let rate = rate_flag(shape.rate)?;
    let header = d.size(60)?;
    let orders_n = d.u16(64)? as usize;
    let restart = d.u16(66)? as usize;
    let channels = d.u16(68)? as usize;
    let patterns_n = d.u16(70)? as usize;
    let instruments_n = d.u16(72)? as usize;
    let flags = d.u16(74)?;
    let (speed, tempo) = (d.u16(76)?, d.u16(78)?);
    if channels > MAX_CHANNELS {
        return Err(format!("{channels} channels; the player has {MAX_CHANNELS}"));
    }
    if channels == 0 {
        return Err("no channels".into());
    }
    if flags & 1 == 0 {
        return Err("the Amiga frequency table; the player has the linear one only".into());
    }
    let orders = d.slice(80, 80 + orders_n);
    let mut warnings: BTreeMap<String, usize> = BTreeMap::new();
    let mut warn = |what: String| *warnings.entry(what).or_default() += 1;

    let mut at = 60usize.checked_add(header).ok_or(crate::bytes::SHORT)?;
    let mut patterns: Vec<(usize, Vec<u8>)> = Vec::new();
    for _ in 0..patterns_n {
        let length = d.size(at)?;
        let rows = d.u16(at + 5)? as usize;
        let size = d.u16(at + 7)? as usize;
        let start = at.saturating_add(length);
        // A pattern with no data is all empty rows.
        patterns.push((rows, if size > 0 { d.slice(start, start + size).to_vec() } else { vec![0x80; rows * channels] }));
        at = start + size;
    }
    // An order may name a pattern the file does not have: 64 empty rows.
    if let Some(&most) = orders.iter().max() {
        while most as usize >= patterns.len() {
            patterns.push((64, vec![0x80; 64 * channels]));
        }
    }
    let unpack = |patterns: &[(usize, Vec<u8>)]| -> Vec<Vec<Vec<[u8; 5]>>> { patterns.iter().map(|(_, packed)| rows(packed, channels)).collect() };
    // An R that leaves the volume alone becomes E9.
    let mut plain = vec![true; channels];
    for row in unpack(&patterns).iter().flatten() {
        for (c, &[.., effect, param]) in row.iter().enumerate() {
            plain[c] &= effect != R || matches!(param >> 4, 0 | 8);
        }
    }
    for (_, packed) in &mut patterns {
        retriggers(packed, channels, &plain);
    }
    let cells = unpack(&patterns);

    for pattern in &cells {
        for row in pattern {
            for &[_note, _ins, volume, effect, param] in row {
                if (effect != 0 || param != 0) && !SILENT.contains(&effect) && (!EFFECTS.contains(&effect) || (effect == 0xE && !E_EFFECTS.contains(&(param >> 4)))) {
                    warn(format!("effect {}", effect_name(effect, param)));
                }
                if !VOLUMES.contains(&(volume >> 4)) && !SILENT_VOLUMES.contains(&(volume >> 4)) {
                    warn(format!("volume column {:X}x", volume >> 4));
                }
            }
        }
    }

    let mut instruments = Vec::new();
    for number in 1..=instruments_n {
        let size = d.size(at)?;
        let samples = d.u16(at + 27)? as usize;
        let mut entry = Instrument::default();
        if samples > 0 {
            let sample_header = d.size(at + 29)?;
            let points = (d.u8(at + 225)? as usize).min(ENVELOPE_POINTS);
            for i in 0..points {
                entry.points.push((d.u16(at + 129 + 4 * i)?, d.u16(at + 131 + 4 * i)?));
            }
            entry.sustain = d.u8(at + 227)?;
            entry.env_loop = (d.u8(at + 228)?, d.u8(at + 229)?);
            entry.flags = if points > 0 { d.u8(at + 233)? & 7 } else { 0 };
            entry.fade = d.u16(at + 239)?;
            // Auto-vibrato: sweep, depth, rate. The player's is a sine.
            entry.auto = [d.u8(at + 236)?, d.u8(at + 237)?, d.u8(at + 238)?];
            let waveform = d.u8(at + 235)?;
            if entry.auto[1] != 0 && waveform != 0 {
                warn(format!("instrument {number}: auto-vibrato waveform {waveform}, played as a sine"));
            }
            if samples > 1 {
                warn(format!("instrument {number}: {samples} samples, the first is used"));
            }
            let head = at.saturating_add(size);
            let body = sample_header.checked_mul(samples).and_then(|n| n.checked_add(head)).ok_or(crate::bytes::SHORT)?;
            let length = d.size(head)?;
            let (mut loop_start, mut loop_len) = (d.size(head + 4)?, d.size(head + 8)?);
            let volume = d.u8(head + 12)?;
            let finetune = d.i8(head + 13)?;
            let kind = d.u8(head + 14)?;
            let relative = d.i8(head + 16)?;
            let raw = d.slice(body, body.saturating_add(length));
            let mut values: Vec<i16> = Vec::with_capacity(raw.len());
            if kind & 0x10 != 0 {
                let mut acc = 0u16;
                let wide: Vec<i16> = raw
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| {
                        acc = acc.wrapping_add(u16::from_le_bytes(*pair));
                        acc as i16
                    })
                    .collect();
                // A sample of 16 bits whose low bytes are all the same
                // is one of 8 in a 16-bit one's place, with an offset
                // of under a step if they are not nought: it is
                // written as 8 with that byte dropped, the same sound
                // in a clip every firmware plays.
                let low = wide.first().map_or(0, |s| s & 0xFF);
                let narrow = wide.iter().all(|s| s & 0xFF == low);
                entry.kept = if narrow { Samples::Cut } else { shape.samples };
                // Cut to 8 bits it is rounded, not its low byte
                // dropped; one that is of 8 bits has nothing to round.
                let cut = |s: i16| if narrow { s & !0xFF } else { (((s as i32 + 128) >> 8).min(127) << 8) as i16 };
                values.extend(wide.iter().map(|&s| if entry.kept == Samples::Cut { cut(s) } else { s }));
                loop_start /= 2;
                loop_len /= 2;
            } else {
                let mut acc = 0u8;
                for &delta in raw {
                    acc = acc.wrapping_add(delta);
                    values.push(((acc as u16) << 8) as i16);
                }
            }
            let looping = kind & 3;
            if looping == 0 || loop_len == 0 {
                loop_start = 0;
                loop_len = 0;
            } else {
                let start = loop_start.min(values.len());
                let end = loop_start.saturating_add(loop_len).min(values.len());
                if looping == 2 {
                    // Ping-pong: there and back is one forward loop.
                    let back: Vec<i16> = values[start..end].iter().rev().copied().collect();
                    values.truncate(end);
                    values.extend(back);
                    loop_len = loop_len.saturating_mul(2);
                } else {
                    values.truncate(end);
                }
            }
            let up = 12 * shape.raised.get(&number).copied().unwrap_or(0);
            entry.relative = (relative as i32 + up).clamp(-128, 127) as i8;
            entry.sample = values;
            entry.looped = (loop_start, loop_len);
            entry.volume = volume;
            entry.finetune = finetune;
            at = body;
            for i in 0..samples {
                at = at.saturating_add(d.size(head + sample_header * i)?);
            }
        } else {
            at = at.saturating_add(size);
        }
        instruments.push(entry);
    }

    // The length, played once as the player does: to the end of the
    // order list or to a jump back.
    let (mut ticks_ms, mut order, mut row, mut now_speed, mut now_tempo) = (0.0f64, 0usize, 0usize, speed as f64, tempo as f64);
    while order < orders.len() {
        let mut go: Option<(usize, usize)> = None;
        for cells_row in cells[orders[order] as usize].iter().skip(row) {
            for &[_note, _ins, _volume, effect, param] in cells_row {
                if effect == 0xF && param != 0 {
                    if param >= 0x20 {
                        now_tempo = param as f64;
                    } else {
                        now_speed = param as f64;
                    }
                } else if effect == 0xB {
                    go = Some((param as usize, 0));
                } else if effect == 0xD {
                    go = Some((go.map_or(order + 1, |g| g.0), (param >> 4) as usize * 10 + (param & 0xF) as usize));
                }
            }
            ticks_ms += now_speed * 2500.0 / now_tempo;
            if go.is_some() {
                break;
            }
        }
        if go.is_some_and(|g| g.0 <= order) {
            break;
        }
        (order, row) = go.unwrap_or((order + 1, 0));
    }

    let byte = |n: usize, what: &str| u8::try_from(n).map_err(|_| format!("{n} {what}; the clip has room for 255"));
    // A sample is predicted where that is the smaller: for a wave of
    // one period the table alone is more than the 16 bits it would
    // save, and those are kept as they are, exact.
    for i in instruments.iter_mut().filter(|i| i.kept == Samples::Packed) {
        i.packed = predicted(&i.sample, i.looped);
        if i.packed.len() >= 2 * i.sample.len() {
            (i.kept, i.packed) = (Samples::Wide, Vec::new());
        }
    }
    // The magic is the newest thing the clip has in it: a firmware
    // from before that refuses it.
    let has = |kept| instruments.iter().any(|i: &Instrument| i.kept == kept && !i.sample.is_empty());
    let mut out = if has(Samples::Packed) {
        PACKED_MAGIC.to_vec()
    } else if has(Samples::Wide) {
        WIDE_MAGIC.to_vec()
    } else {
        MAGIC.to_vec()
    };
    out.extend([
        channels as u8,
        byte(speed as usize, "ticks a row")?,
        byte(tempo as usize, "beats a minute")?,
        byte(orders.len(), "orders")?,
        byte(patterns.len(), "patterns")?,
        byte(instruments.len(), "instruments")?,
        shape.drive,
        shape.release,
    ]);
    out.extend((ticks_ms.round_ties_even() as u32).to_le_bytes());
    out.extend([shape.passes, rate | if shape.bits == 16 { 2 } else { 0 }]);
    for k in section {
        out.extend(k.to_le_bytes());
    }
    // The order to start over from, for the alarm, which plays the tune
    // round and round.
    out.extend([if restart < orders.len() { restart as u8 } else { 0 }, shape.knee, 0, 0]);
    out.extend(orders);
    let tables = out.len() + 6 * patterns.len() + 80 * instruments.len();
    let mut body: Vec<u8> = Vec::new();
    for (rows, packed) in &patterns {
        out.extend((*rows as u16).to_le_bytes());
        out.extend(((tables + body.len()) as u32).to_le_bytes());
        body.extend(packed);
    }
    for i in &instruments {
        // A sample of 16 bits at an even offset: the clock reads two
        // bytes at a load, from an even address (`xm::is_misplaced`).
        if !i.sample.is_empty() && i.kept == Samples::Wide && !(tables + body.len()).is_multiple_of(2) {
            body.push(0);
        }
        for n in [tables + body.len(), i.sample.len(), i.looped.0, i.looped.1] {
            out.extend(u32::try_from(n).map_err(|_| "a sample's loop is past what a clip can say")?.to_le_bytes());
        }
        out.extend([i.volume, i.finetune as u8, i.relative as u8, i.flags, i.points.len() as u8, i.sustain, i.env_loop.0, i.env_loop.1]);
        out.extend(i.fade.to_le_bytes());
        out.extend(i.auto);
        let kept = if i.sample.is_empty() { Samples::Cut } else { i.kept };
        // The sample's flags: how it is kept, and bit 2 for a
        // predicted one's seek points.
        out.extend([kept as u8 | ((kept == Samples::Packed) as u8) << 2, 0, 0]);
        for p in 0..ENVELOPE_POINTS {
            let (x, y) = i.points.get(p).copied().unwrap_or((0, 0));
            out.extend(x.to_le_bytes());
            out.extend(y.to_le_bytes());
        }
        match kept {
            Samples::Cut => body.extend(i.sample.iter().map(|s| (s >> 8) as u8)),
            Samples::Wide => body.extend(i.sample.iter().flat_map(|s| s.to_le_bytes())),
            Samples::Packed => body.extend(&i.packed),
        }
    }
    debug_assert_eq!(out.len(), tables);
    out.extend(body);
    if let Some(tops) = shape.fit {
        let (knee, peak, flattened, allowed) = fit_knee(&out, tops);
        out[KNEE_AT] = knee;
        // No knee the header can say keeps a drive over 127 to it.
        let over = if flattened > allowed { ", which no knee keeps to at this drive: a lower one would" } else { "" };
        notes.push(format!(
            "knee=auto: knee={knee}, the mix peaking at {:.1}; the limiter flattens {flattened} samples, {tops} a minute allowed{over}",
            peak as f64 / 1024.0
        ));
    }

    let name = text(d.slice(17, 37));
    notes.push(format!(
        "\"{}\", {channels} channels, {} orders, {:.1} s, {} bytes (the module is {})",
        name.trim(),
        orders.len(),
        ticks_ms / 1000.0,
        out.len(),
        module.len()
    ));
    for (what, count) in &warnings {
        notes.push(format!("not played: {what} ({count} times)"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_r_that_changes_no_volume_is_e9() {
        // Two channels, a row a line: packed cells with an effect and
        // a parameter, a whole cell, a cell with the effect alone.
        let rows: [&[u8]; 5] = [
            &[0x98, R, 0x02, 0x98, R, 0x02],
            &[0x9B, 49, 1, R, 0x83, 0x98, R, 0x83],
            &[49, 1, 0, R, 0x03, 0x80],
            &[0x88, R, 0x98, R, 0x00],
            &[0x98, R, 0x42, 0x98, 0xC, R],
        ];
        let mut packed = rows.concat();
        // The first channel has an R with a volume, in the last row.
        retriggers(&mut packed, 2, &[false, true]);
        let after: [&[u8]; 5] = [
            &[0x98, R, 0x02, 0x98, 0xE, 0x92],
            &[0x9B, 49, 1, 0xE, 0x93, 0x98, 0xE, 0x93],
            &[49, 1, 0, R, 0x03, 0x80],
            &[0x88, R, 0x98, R, 0x00],
            &[0x98, R, 0x42, 0x98, 0xC, R],
        ];
        assert_eq!(packed, after.concat());
    }
}
