// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! A tracker module as a clip: a player for the part of FastTracker 2's
//! XM format that chiptunes use, giving the samples the clip's
//! modulator takes, `clip::PCM_HZ` a second.
//!
//! A module is a few short recorded waveforms and patterns of notes
//! that say when to play which at what pitch and volume, so a tune of
//! a minute is ten or twenty kilobytes where its PCM is half a
//! megabyte. The price is this player. It is written for the clock
//! and not after any other: no heap, the module read in place from
//! flash, a fixed number of channels.
//!
//! The file is not the XM itself but what `tools/pack/src/xmc.rs` makes
//! of one (`MAGIC`): the same patterns, still packed, behind tables of
//! where each pattern and each instrument's sample is, with the samples
//! as plain signed bytes (an XM stores them as differences, which
//! cannot be read from the middle) and ping-pong loops unrolled. An
//! instrument of several samples is an entry a sample (`Instrument::keyed`). A module that keeps samples of 16 bits is
//! `WIDE_MAGIC`: the same layout, with an instrument's entry saying
//! which of them are, and a firmware from before it refuses the clip
//! and does not play their bytes as 8-bit ones; each such sample is at
//! an even offset, and the clip at an even one of its pack (`words`). One that keeps a
//! 16-bit sample in 8 bits a sample, as what is left of each after
//! the samples before it have predicted it (`adpcm::decode_block`,
//! with a table of predictors fitted to the sample), is
//! `PACKED_MAGIC`, refused the same way.
//!
//! What plays: notes with the instrument's relative note and finetune
//! on the linear frequency table or the Amiga one (`AMIGA`), forward loops, the volume envelope
//! with its sustain and loop points and the fade after a key-off, the
//! volume column's set, slides, vibrato and tone portamento, and the
//! effects 0 arpeggio (in FastTracker's tick order), 1 2 3 portamento,
//! 4 vibrato, 5 6 with volume slide, 9 offset, A volume slide, B jump,
//! C volume, D break, E1 E2 fine portamento, E5 finetune, 7 tremolo (a sine), L envelope position, T tremor, X1 X2 extra fine portamento, E9 retrigger, EA EB fine
//! volume, E6 pattern loop, EC cut, ED delay, EE pattern delay, F speed and tempo, G global volume, K key
//! off, H global volume slide, R retrigger with a volume change (`Tune::retrigger`); and an instrument's auto-vibrato, as a sine. Panning is nothing on one buzzer. The converter says what a
//! module uses beyond that.
//!
//! The channels are mixed at four times the output rate with linear
//! interpolation and averaged down. A level moves to where a tick put
//! it over a couple of milliseconds, and a note cut by the next on its
//! channel dies away over a few, as FastTracker's do: a step in the
//! mix is a click. Then the tune gets what the PCM
//! clips get on the host in `tools/pack/src/wav.rs`, since the buzzer is
//! the same: the bass cut by a high-pass and the level pushed hard into
//! a limiter. How much bass and how hard are the clip's to say, in its
//! header; a tune whose melody sits under 900 Hz wants a lower cut than
//! the PCM clips' 900 Hz twice over.
//!
//! The tune is played through once: the order list to its end, or to
//! a jump back.

use crate::clip::{FAST_HZ, PCM_HZ};
use crate::adpcm::{self, BLOCK, BLOCK_BYTES};
use crate::shape::Shaper;

/// The first bytes of a converted module.
pub const MAGIC: &[u8; 4] = b"XMC2";
/// And of one with a sample of 16 bits in it (`Instrument::wide`).
pub const WIDE_MAGIC: &[u8; 4] = b"XMC3";
/// And of one with a predicted sample in it (`Kind::Packed`).
pub const PACKED_MAGIC: &[u8; 4] = b"XMC4";
/// What a predicted sample starts with: for its loop, the three
/// samples before the block the loop starts in, the newest first, and
/// the sample at the loop's start, each an `i16` (nothing for a sample
/// with no loop); then its table of predictors. The blocks follow.
pub const PACKED_LOOP: usize = 8;
pub const PACKED_HEAD: usize = PACKED_LOOP + adpcm::TABLE_BYTES;
/// Samples between a predicted sample's seek points, which is what a
/// 9xx's offset counts in: a sample has, after its table and before
/// its blocks, for each `SEEK` samples but the first the three samples
/// before that block, the newest first, each an `i16` (`SEEK_BYTES`),
/// so that a note starting there has nothing before it to decode, and
/// neither has a channel heard again after a silence (`pass`). The
/// converter gives them to every predicted sample; a clip from before
/// 2026-11-23 has them only where a 9xx plays its sample from an
/// offset, and the instrument's flags say which.
pub const SEEK: usize = 256;
pub const SEEK_BYTES: usize = 6;
const _: () = assert!(SEEK.is_multiple_of(BLOCK));

/// The seek points of a sample of `len` samples that has them.
pub const fn seek_points(len: usize) -> usize {
    len.saturating_sub(1) / SEEK
}
/// The header: magic, channels, speed, tempo, orders, patterns,
/// instruments, the limiter's drive and release (0 for `DRIVE` and
/// `RELEASE_SHIFT`), the length in ms as `u32`; then the high-pass:
/// how many times it is applied (0 to 2), the output rate (0 for
/// `PCM_HZ`, 1 for twice that; bit 1 asks for samples finer than the
/// 8-bit step, `Tune::next_sample`), and its
/// second-order section as five `i16` in Q14, b0 b1 b2 a1 a2, which
/// the converter works out since the chip has no tangent; the order to
/// start over from when the tune is played round and round; the
/// limiter's knee (0 for the shaper's own); two spare.
const HEADER: usize = 32;

/// Whether `data` is a converted module of the first layout, which
/// this firmware would take for PCM: convert it again.
pub const fn is_old_tune(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == b'X' && data[1] == b'M' && data[2] == b'C' && data[3] == b'1'
}
/// A pattern's entry: rows as `u16`, its data's offset as `u32`.
const PATTERN_ENTRY: usize = 6;
/// An instrument's entry; see `Instrument`.
const INSTRUMENT_ENTRY: usize = 80;
/// The notes of an instrument's table of samples (`Instrument::keyed`).
pub const KEYS: usize = 96;
/// Points of a volume envelope.
const ENVELOPE_POINTS: usize = 12;
/// A level the tick changed is reached over a 512th of a second, about
/// 2 ms, and not within one mixed sample: FastTracker ramps its
/// volumes too, and a step in level is a click.
const RAMP_PER_SECOND: u32 = 512;
/// What a cut note left in the mix loses a 64th of each mixed sample:
/// gone in about 5 ms.
const CUT_DECAY: i32 = 64;
/// Channels played; the converter refuses a module with more. A
/// channel that is not sounding costs the mixer nothing, one that is
/// about as much as any other, so a module's cost follows its voices.
pub const MAX_CHANNELS: usize = 20;

/// The mixing rate: four times the PCM clips' 8 kHz, and twice the
/// 16 kHz a clip may ask for instead.
const MIX_HZ: u32 = PCM_HZ * 4;
/// And for a clip at `FAST_HZ`: twice that.
const FAST_MIX_HZ: u32 = FAST_HZ * 2;
/// The header's byte 17: the samples at twice `PCM_HZ`, finer than the
/// 8-bit step, at `FAST_HZ`.
const TWICE: u8 = 1;
const WIDE: u8 = 2;
const FAST: u8 = 4;
/// And bit 3: the module is on the Amiga frequency table (`AMIGA`).
/// A firmware up to v0.5 does not look and plays it on the linear
/// one: the notes where they are, the slides and vibratos not.
const AMIGA_TABLE: u8 = 8;

/// Period units a semitone, and the period of the lowest note.
const SEMITONE: i32 = 64;
const PERIOD_MAX: i32 = 10 * 12 * SEMITONE;
/// The period at which a sample plays at `C4_HZ`.
const PERIOD_C4: i32 = PERIOD_MAX - 4 * 12 * SEMITONE;
const C4_HZ: u32 = 8363;
/// FastTracker's other table, the Amiga's, as OpenMPT has it
/// (`XMPeriodTable`): the periods of an octave and a third of a tone
/// either side of it, eight to a semitone, C at `AMIGA[8]`. A note's
/// period is between two of them by its finetune, twice that for the
/// lowest octave and halved for each above; C-4 is 1712 and plays at
/// `C4_HZ`, and a period is as many Hz as `AMIGA_HZ` over it.
const AMIGA: [u16; 104] = [
    907, 900, 894, 887, 881, 875, 868, 862, 856, 850, 844, 838, 832, 826, 820, 814, 808, 802, 796, 791, 785, 779, 774, 768, 762, 757, 752, 746, 741, 736, 730, 725, 720, 715, 709,
    704, 699, 694, 689, 684, 678, 675, 670, 665, 660, 655, 651, 646, 640, 636, 632, 628, 623, 619, 614, 610, 604, 601, 597, 592, 588, 584, 580, 575, 570, 567, 563, 559, 555, 551,
    547, 543, 538, 535, 532, 528, 524, 520, 516, 513, 508, 505, 502, 498, 494, 491, 487, 484, 480, 477, 474, 470, 467, 463, 460, 457, 453, 450, 447, 443, 440, 437, 434, 431,
];
const AMIGA_HZ: u32 = C4_HZ * 1712;
/// The step of a period of 1 on that table, in 16.16 at a mixing
/// rate: a period's step is this over it, one division of 32 bits a
/// channel a tick, which the chip has. It fits at either rate.
const fn amiga_step(mix_hz: u32) -> u32 {
    let step = ((AMIGA_HZ as u64) << 16) / mix_hz as u64;
    assert!(step <= u32::MAX as u64);
    step as u32
}
const AMIGA_STEP: u32 = amiga_step(MIX_HZ);
const AMIGA_STEP_FAST: u32 = amiga_step(FAST_MIX_HZ);
/// The longest period on that table, FastTracker's limit.
const AMIGA_MAX: i32 = 31999;
/// The note an arpeggio does not go above, C-8.
const ARPEGGIO_TOP: i32 = 96;
/// Steps of the pitch table an octave.
const OCTAVE: i32 = 12 * SEMITONE;

/// 2^(i / 768) for one octave, as Q15 from 32768 up.
const POW: [u16; OCTAVE as usize] = {
    // 2^(1 / 768) in Q32.
    const STEP: u64 = 4_298_845_406;
    let mut table = [0u16; OCTAVE as usize];
    let mut x: u64 = 1 << 32;
    let mut i = 0;
    while i < table.len() {
        table[i] = ((x + (1 << 16)) >> 17) as u16;
        x = ((x as u128 * STEP as u128) >> 32) as u64;
        i += 1;
    }
    table
};

/// Half a sine wave, 32 steps, peak 127: the tremolo's, which is
/// FastTracker's table as OpenMPT has it (`ModSinusTable`) and not
/// the vibrato's halved.
const TREMOLO: [u8; 32] = [0, 12, 25, 37, 49, 60, 71, 81, 90, 98, 106, 112, 117, 122, 125, 126, 127, 126, 125, 122, 117, 112, 106, 98, 90, 81, 71, 60, 49, 37, 25, 12];
/// Half a sine wave, 32 steps, peak 255: the vibrato's.
const SINE: [u8; 32] =
    [0, 24, 49, 74, 97, 120, 141, 161, 180, 197, 212, 224, 235, 244, 250, 253, 255, 253, 250, 244, 235, 224, 212, 197, 180, 161, 141, 120, 97, 74, 49, 24];

/// The key-off note.
const KEY_OFF: u8 = 97;
/// The effect R, by its number in a cell.
pub const RETRIGGER: u8 = 0x1B;
/// The effect H, the global volume slide.
pub const GLOBAL_SLIDE: u8 = 0x11;
/// The effects L, the envelope's position, T, the tremor, and X, the
/// extra fine portamento.
pub const ENVELOPE_AT: u8 = 0x15;
pub const TREMOR: u8 = 0x1D;
pub const EXTRA_FINE: u8 = 0x21;
/// The fade's full level.
const FADE_FULL: u32 = 32768;

/// What one 8-bit step is in the samples the tune gives.
pub use crate::shape::STEP;

/// Whether `data` is a converted module.
pub const fn is_tune(data: &[u8]) -> bool {
    data.len() >= HEADER
        && data[0] == MAGIC[0]
        && data[1] == MAGIC[1]
        && data[2] == MAGIC[2]
        && (data[3] == MAGIC[3] || data[3] == WIDE_MAGIC[3] || data[3] == PACKED_MAGIC[3])
}

/// How long the tune plays, as the converter worked it out.
pub const fn duration_ms(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[12], data[13], data[14], data[15]])
}

fn byte(d: &[u8], at: usize) -> u8 {
    d.get(at).copied().unwrap_or(0)
}

fn word(d: &[u8], at: usize) -> u16 {
    byte(d, at) as u16 | (byte(d, at + 1) as u16) << 8
}

fn long(d: &[u8], at: usize) -> u32 {
    word(d, at) as u32 | (word(d, at + 2) as u32) << 16
}

/// How an instrument's sample is kept.
#[derive(Clone, Copy, Default, PartialEq)]
enum Kind {
    /// A byte a sample.
    #[default]
    Narrow,
    /// Two bytes a sample, 16 bits.
    Wide,
    /// 16 bits predicted, in blocks (`adpcm::decode_block`).
    Packed,
}

/// The bytes of a sample that is not predicted: `len` samples from
/// `at` in the clip, a byte each or two (`Kind::Wide`), or as many of
/// them as the clip has. The mixer reads a sample from its own bytes
/// and not from the clip: one bounds check, against the sample's own
/// end, so a sample past it is 0 with nothing more to ask.
fn held(d: &[u8], at: usize, len: usize, kind: Kind) -> &[u8] {
    let bytes = match kind {
        Kind::Narrow => len,
        Kind::Wide => len.saturating_mul(2),
        Kind::Packed => 0,
    };
    let rest = d.get(at..).unwrap_or(&[]);
    &rest[..rest.len().min(bytes)]
}

/// A sample of 16 bits as the `i16`s it is, in the clip's byte order
/// (`i16::from_le` reads one): what `held` gives, for the mixer to
/// read a sample with one load of 16 bits and not two of a byte, which
/// on twenty channels was twelve points of the processor.
///
/// Empty, so the sample is silence, if it does not start on an even
/// address. The pack maker puts a clip that has such samples at an
/// even offset in the pack and the converter each of them at an even
/// one in the clip (`is_misplaced`, which a pack is checked with);
/// the partition is mapped at an even address and a clip built into
/// the firmware is aligned (`core/build.rs`). Half a sample at the
/// end, where a clip was cut, is no sample.
fn words(bytes: &[u8]) -> &[i16] {
    // SAFETY: `align_to` asks only that the middle slice's type may be
    // made of the bytes of the first's, as with a transmute. `i16` is
    // two bytes with no padding and no value that is not one, so any
    // two bytes of a `[u8]` are an `i16`; neither type has anything to
    // drop or a cell in it, and the borrow is shared going in and
    // coming out, for the same lifetime. That the middle starts at an
    // address `i16` may be read from and lies within `bytes` is
    // `align_to`'s own doing, not this call's. How it splits the bytes
    // is not relied on for more than that: bytes left before the
    // middle mean the sample is at an odd address, and it is then not
    // played at all.
    let (before, words, _) = unsafe { bytes.align_to::<i16>() };
    if before.is_empty() { words } else { &[] }
}

/// Whether a clip with the pack's offset `at` (0 for one that is
/// aligned by itself) has a 16-bit sample at an odd address, which
/// `words` does not play. A pack skips the clip: it is not this
/// project's converter's.
pub const fn is_misplaced(data: &[u8], at: usize) -> bool {
    const fn get(d: &[u8], at: usize) -> usize {
        if at < d.len() { d[at] as usize } else { 0 }
    }
    if !is_tune(data) {
        return false;
    }
    let instruments = HEADER + get(data, 7) + PATTERN_ENTRY * get(data, 8);
    let mut i = 0;
    while i < get(data, 9) {
        let entry = instruments + INSTRUMENT_ENTRY * i;
        // The sample's flags (`Instrument::kind`) and its offset's low byte.
        if get(data, entry + 29) & 3 == 1 && !(at + get(data, entry)).is_multiple_of(2) {
            return true;
        }
        i += 1;
    }
    false
}

/// Sample `pos` of a sample of 8 bits (`held`), at 16: its byte times
/// 256. 0 past its end.
#[inline(always)]
fn narrow_at(held: &[u8], pos: u32) -> i32 {
    held.get(pos as usize).map_or(0, |&s| (s as i8 as i32) << 8)
}

/// Sample `pos` of a sample of 16 bits (`words`). 0 past its end.
#[inline(always)]
fn wide_at(words: &[i16], pos: u32) -> i32 {
    words.get(pos as usize).map_or(0, |&s| i16::from_le(s) as i32)
}

/// Mixed samples a `Tune::render`: a channel is mixed over a run of
/// them at a time, its state in registers. A whole number of output
/// samples at either oversampling, so no group of mixed samples is
/// split between two renders.
const RUN: usize = 16;
const _: () = assert!(RUN.is_multiple_of(4));

/// A channel's part in a run of mixed samples, added to `out`: the
/// two samples it is between (`pair`, which is all that differs from
/// kind to kind, so this is built once a kind), the level's ramp, the
/// step, the loop's wrap and the sample's end, each where the mixer
/// had it when it went a sample at a time over the channels
/// (`Tune::mix_one`, kept for the tests). The position, the fraction
/// and the gain are locals for the run and written back after it.
#[inline(always)]
fn run<'a>(ch: &mut Channel<'a>, out: &mut [i32], mut pair: impl FnMut(&mut Channel<'a>, u32) -> (i32, i32)) {
    let (mut pos, mut frac, step) = (ch.pos, ch.frac, ch.step);
    let (mut gain, mut ramp, gain_step, amp) = (ch.gain, ch.ramp, ch.gain_step, ch.amp);
    let (loop_start, loop_len, len) = (ch.loop_start, ch.loop_len, ch.len);
    let end = loop_start.wrapping_add(loop_len);
    for sum in out.iter_mut() {
        let (a, b) = pair(ch, pos);
        let sample = a + (((b - a) * (frac >> 8) as i32) >> 8);
        if ramp != 0 {
            ramp -= 1;
            gain = if ramp == 0 { amp << 8 } else { gain + gain_step };
        }
        *sum += (sample * (gain >> 8)) >> 12;
        let moved = frac + (step & 0xFFFF);
        frac = moved & 0xFFFF;
        pos = pos.saturating_add((step >> 16) + (moved >> 16));
        if loop_len != 0 {
            if pos >= end {
                pos = loop_start + (pos - end) % loop_len;
            }
        } else if pos >= len {
            ch.playing = false;
            ch.mixing = false;
            break;
        }
    }
    (ch.pos, ch.frac, ch.gain, ch.ramp) = (pos, frac, gain, ramp);
}

/// No block of a channel's is decoded.
const NO_BLOCK: u32 = u32::MAX - 2;

/// Block `b` of the channel's predicted sample into a half of its
/// buffer, after the block before it; silence past the sample's end.
#[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"), inline(never))]
fn fill(d: &[u8], ch: &mut Channel<'_>, b: u32, half: usize) {
    let out: &mut [i16; BLOCK] = if half == 0 { ch.blocks.first_chunk_mut().unwrap() } else { ch.blocks.last_chunk_mut().unwrap() };
    let first = b as usize * BLOCK;
    #[cfg(test)]
    tests::FILLS.with(|n| n.set(n.get() + 1));
    let at = ch.data as usize + PACKED_HEAD + ch.seeks as usize * SEEK_BYTES + b as usize * BLOCK_BYTES;
    let table = d.get(ch.data as usize + PACKED_LOOP..).unwrap_or(&[]);
    match d.get(at..).and_then(|rest| rest.first_chunk::<BLOCK_BYTES>()) {
        Some(block) if first < ch.len as usize => {
            adpcm::decode_block(&table[..table.len().min(adpcm::TABLE_BYTES)], &mut ch.before, block, out);
            let left = ch.len as usize - first;
            if left < BLOCK {
                out[left..].fill(0);
            }
        }
        _ => out.fill(0),
    }
}

/// The latest block at or before `block` that decoding the channel's
/// predicted sample can start from, with the three samples before it:
/// the sample's start, its loop's block and a seek point.
#[inline(always)]
fn nearest(d: &[u8], ch: &Channel<'_>, block: u32) -> (u32, [i32; 3]) {
    let head = ch.data as usize;
    let three = |at: usize| [word(d, at) as i16 as i32, word(d, at + 2) as i16 as i32, word(d, at + 4) as i16 as i32];
    let mut start = (0, [0; 3]);
    let looped = ch.loop_start / BLOCK as u32;
    if ch.loop_len != 0 && looped <= block && looped > start.0 {
        start = (looped, three(head));
    }
    let per = (SEEK / BLOCK) as u32;
    let seek = (block / per).min(ch.seeks);
    if seek * per > start.0 {
        start = (seek * per, three(head + PACKED_HEAD + (seek as usize - 1) * SEEK_BYTES));
    }
    start
}

/// Blocks a silent channel's predicted sample may be past a place
/// decoding can start from (`nearest`) with nothing decoded for it:
/// what being heard again may cost at once, and two. The blocks
/// between a sample's seek points, four times over.
const REACH: u32 = 4 * (SEEK / BLOCK) as u32;

/// A silent channel moved on by the `n` mixed samples a tick lasted,
/// in one step where the mixer takes one a sample (`Tune::mix`, whose
/// arithmetic this is): the position, its fraction, the loop's wrap
/// and the sample's end. Once a tick, so in the flash.
///
/// A predicted sample too far past anywhere decoding can start from
/// is decoded up to where it now is, on from where it was: a tick's
/// blocks at once, and none of them to do when it is heard again.
#[inline(never)]
fn pass(d: &[u8], ch: &mut Channel<'_>, n: u32) {
    let at = ((ch.pos as u64) << 16 | ch.frac as u64) + ch.step as u64 * n as u64;
    ch.frac = (at & 0xFFFF) as u32;
    let pos = at >> 16;
    if ch.loop_len != 0 {
        let end = ch.loop_start as u64 + ch.loop_len as u64;
        ch.pos = if pos >= end { ch.loop_start + ((pos - end) % ch.loop_len as u64) as u32 } else { pos as u32 };
    } else {
        ch.pos = pos.min(u32::MAX as u64) as u32;
        if ch.pos >= ch.len {
            ch.playing = false;
            return;
        }
    }
    if ch.kind == Kind::Packed {
        let block = ch.pos / BLOCK as u32;
        if block != ch.block && block - nearest(d, ch, block).0 > REACH {
            refill(d, ch, block);
        }
    }
}

/// Makes `block` and the one after it the two the channel has
/// decoded. The next block is the common case and costs a block.
/// Anything else decodes forward from the nearest place there is a
/// state to start from: where the channel is, the loop's block (what
/// the sample's head keeps, for the loop's wrap), a seek point (for
/// a 9xx, which lands on one), and the sample's start. A sample with
/// no seek points hit far in is decoded from its start: an older
/// clip's, the converter giving every sample its seek points.
#[inline(never)]
#[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"))]
fn refill(d: &[u8], ch: &mut Channel<'_>, block: u32) {
    if block == ch.block.wrapping_add(1) {
        ch.blocks.copy_within(BLOCK.., 0);
    } else {
        let start = nearest(d, ch, block);
        // Or on from where the channel is, if that is nearer.
        let on = ch.block.wrapping_add(2);
        let mut from = if ch.block != NO_BLOCK && on <= block && on >= start.0 {
            on
        } else {
            ch.before = start.1;
            start.0
        };
        while from < block {
            fill(d, ch, from, 0);
            from += 1;
        }
        fill(d, ch, block, 0);
    }
    fill(d, ch, block + 1, 1);
    ch.block = block;
}

/// An instrument's entry in the file: where its sample is (offset,
/// length, loop start and loop length as `u32`, the three in samples, a loop length of 0 for
/// none), its volume, finetune and relative note, the envelope's flags
/// (1 on, 2 sustain, 4 loop), number of points, sustain point and loop
/// points, the fade as `u16`, the auto-vibrato's sweep, depth and rate,
/// the sample's flags (1 for samples of 16 bits, little-endian, two
/// bytes each; `WIDE_MAGIC`; 2 for a predicted sample, `PACKED_HEAD`
/// and then its blocks; `PACKED_MAGIC`; with 4 as well, its seek
/// points between the two, `SEEK`; 8 for an instrument of several
/// samples, `Instrument::keyed`), two spare bytes, then twelve
/// points of frame and level as `u16` each.
#[derive(Clone, Copy)]
struct Instrument<'a> {
    d: &'a [u8],
    at: usize,
}

impl Instrument<'_> {
    fn sample(&self) -> (u32, u32, u32, u32) {
        (long(self.d, self.at), long(self.d, self.at + 4), long(self.d, self.at + 8), long(self.d, self.at + 12))
    }

    fn volume(&self) -> u8 {
        byte(self.d, self.at + 16).min(64)
    }

    fn finetune(&self) -> i32 {
        byte(self.d, self.at + 17) as i8 as i32
    }

    fn relative(&self) -> i32 {
        byte(self.d, self.at + 18) as i8 as i32
    }

    fn envelope_flags(&self) -> u8 {
        byte(self.d, self.at + 19)
    }

    fn points(&self) -> usize {
        (byte(self.d, self.at + 20) as usize).min(ENVELOPE_POINTS)
    }

    fn sustain(&self) -> usize {
        byte(self.d, self.at + 21) as usize
    }

    fn envelope_loop(&self) -> (usize, usize) {
        (byte(self.d, self.at + 22) as usize, byte(self.d, self.at + 23) as usize)
    }

    fn fade(&self) -> u32 {
        word(self.d, self.at + 24) as u32
    }

    /// The auto-vibrato: ticks to full depth, the depth in period
    /// units, and the rate, 256 to a cycle.
    fn auto_vibrato(&self) -> (u32, u32, u8) {
        (byte(self.d, self.at + 26) as u32, byte(self.d, self.at + 27) as u32, byte(self.d, self.at + 28))
    }

    /// How the sample is kept.
    fn kind(&self) -> Kind {
        match byte(self.d, self.at + 29) & 3 {
            0 => Kind::Narrow,
            1 => Kind::Wide,
            _ => Kind::Packed,
        }
    }

    /// The seek points a predicted sample has: none unless its flags
    /// say so.
    fn seeks(&self) -> u32 {
        if byte(self.d, self.at + 29) & 6 == 6 { seek_points(long(self.d, self.at + 4) as usize) as u32 } else { 0 }
    }

    /// Whether the instrument has several samples: the `KEYS` bytes
    /// before its sample are then, for each note, the number of the
    /// entry whose sample the note plays, this one or one of those
    /// after the module's own, which have the instrument's envelope
    /// and a sample each.
    fn keyed(&self) -> bool {
        byte(self.d, self.at + 29) & 8 != 0
    }

    /// Point `i` of the envelope: its frame and its level, 0 to 64.
    fn point(&self, i: usize) -> (u32, u32) {
        let at = self.at + 32 + 4 * i.min(ENVELOPE_POINTS - 1);
        (word(self.d, at) as u32, (word(self.d, at + 2) as u32).min(64))
    }
}

/// One cell of a pattern.
#[derive(Clone, Copy, Default)]
struct Cell {
    note: u8,
    instrument: u8,
    volume: u8,
    effect: u8,
    param: u8,
}

impl Cell {
    /// Tone portamento, by effect or by the volume column: the note is
    /// where to slide to, not one to start.
    fn slides_to_note(&self) -> bool {
        self.effect == 3 || self.effect == 5 || self.volume >> 4 == 0xF
    }

    fn has_vibrato(&self) -> bool {
        self.effect == 4 || self.effect == 6 || self.volume >> 4 == 0xB
    }

    /// `EDx` with a tick to wait for.
    fn delay(&self) -> u8 {
        if self.effect == 0xE && self.param >> 4 == 0xD { self.param & 0xF } else { 0 }
    }
}

#[derive(Clone, Copy, Default)]
struct Channel<'a> {
    cell: Cell,
    /// The instrument that is sounding, from 1; 0 for none yet.
    instrument: u8,
    /// The instrument last named, which the next note to begin takes.
    named: u8,
    /// The sample being played: where, how long, and its loop.
    data: u32,
    len: u32,
    loop_start: u32,
    loop_len: u32,
    kind: Kind,
    /// Its bytes, if its samples are a byte each (`held`), and its
    /// samples, if they are 16 bits (`words`); empty otherwise.
    held: &'a [u8],
    words: &'a [i16],
    /// Of a predicted sample: two blocks as decoded, the one `pos` is
    /// in and the next, which block the first is (`NO_BLOCK` for
    /// none), the three samples before the next to be decoded, the
    /// newest first, and the sample at the loop's start.
    blocks: [i16; 2 * BLOCK],
    block: u32,
    /// The seek points before its blocks, 0 for a sample with none.
    seeks: u32,
    before: [i32; 3],
    loop_first: i32,
    playing: bool,
    /// Playing with no gain and no ramp to one, as the last tick left
    /// it: the mixer passes it by and the next tick moves it on
    /// (`pass`). And `mixing`, playing and not quiet, which is all
    /// the mixer asks of a channel: one test, as before there was a
    /// quiet one.
    quiet: bool,
    mixing: bool,
    /// Position in the sample, with 16 bits of fraction, and the step a
    /// mixed sample.
    pos: u32,
    frac: u32,
    step: u32,
    /// The note's period, where a tone portamento is going, and what
    /// vibrato adds for now.
    period: i32,
    target: i32,
    vibrato: i32,
    /// One more than the semitones an arpeggio's tick is above the
    /// note; 0 on a tick with none.
    arpeggio: u8,
    /// The period being sounded, all of those together.
    sounding: i32,
    /// 0 to 64.
    volume: u8,
    /// What the mixer is to multiply by, 0 to 4096, and what it does
    /// multiply by for now, in 256ths of that: the level gets to where
    /// a tick put it over `ramp` mixed samples, `gain_step` each, and a
    /// note that has just begun (`fresh`) starts at its level.
    amp: i32,
    gain: i32,
    gain_step: i32,
    ramp: u32,
    fresh: bool,
    key_on: bool,
    envelope_frame: u32,
    fade: u32,
    /// Effect memories; FastTracker keeps one for each direction of
    /// the portamento.
    porta: u8,
    porta_down: u8,
    tone_speed: u8,
    slide: u8,
    global_slide: u8,
    /// E6: the row its loop goes back to, and the times still to go.
    loop_row: u8,
    loop_count: u8,
    vibrato_speed: u8,
    vibrato_depth: u8,
    vibrato_pos: u8,
    /// The period of the note last begun, for R to begin it again, and
    /// R's own: its parameter as remembered, the ticks counted, and
    /// the quarters of a volume step its last change left over.
    struck: i32,
    retrig_param: u8,
    retrig_count: u8,
    retrig_fine: u8,
    /// What E1 and E2 last slid by, the high nibble E1's, and the same
    /// of X1 and X2.
    fine: u8,
    extra_fine: u8,
    /// The tremolo: where in its cycle (64 a cycle), its speed and
    /// depth, and what it adds to the volume on this tick, in 256ths
    /// of full volume.
    tremolo_pos: u8,
    tremolo_speed: u8,
    tremolo_depth: u8,
    tremolo: i16,
    /// T: its parameter, and FastTracker's count as OpenMPT has it
    /// (`ProcessTremor`): bit 7 a tremor begun, bit 6 sounding, bit 5
    /// an instrument named since, the rest the ticks left.
    tremor_param: u8,
    tremor: u8,
    /// The volume heard, in 256ths, for `dump_the_ticks`.
    #[cfg(test)]
    heard: i32,
    /// The instrument's own vibrato: where in its cycle, and its depth
    /// so far in 256ths of a period unit.
    auto_pos: u8,
    auto_depth: u32,
}

/// A module being played.
pub struct Tune<'a> {
    d: &'a [u8],
    channels: usize,
    order_len: usize,
    patterns: usize,
    instruments: usize,
    patterns_at: usize,
    instruments_at: usize,
    speed: u8,
    tempo: u8,
    global: u8,
    /// Where in the song: the order, the row about to be read, the rows
    /// of this pattern and the next cell's place in the file.
    order: usize,
    row: u16,
    rows: u16,
    cursor: usize,
    /// The tick of the row, counted through its repeats (EE).
    tick: u16,
    /// The rows the row is held for besides its own: EEx.
    row_delay: u8,
    /// A jump or a break read on this row: the order and row to go to.
    go: Option<(usize, u16)>,
    /// The row a break or a pattern loop last named, FastTracker's
    /// one word for both: a loop goes back to it, and a pattern that
    /// ends after one begins the next there.
    break_row: u16,
    /// A pattern loop goes back at this row's end.
    loop_jump: bool,
    pool: i32,
    /// Mixed samples a second. A tick is 2.5 / tempo seconds: each
    /// mixed sample takes `tempo` out of a pool of 2.5 times this.
    mix_hz: u32,
    /// On the Amiga frequency table, not the linear one.
    amiga: bool,
    /// `amiga_step` of the mixing rate.
    amiga_step: u32,
    /// Whether the tune goes round and round: it follows a jump back,
    /// and from the end of the order list it starts over.
    looping: bool,
    /// The tick being played is the tune's last.
    last: bool,
    ended: bool,
    /// What the notes cut short were adding to the mix when they were
    /// cut, on its way to nothing (`sounding_now`).
    cut: i32,
    /// What the pool held when the tick began (`wind`): the mixed
    /// samples since are what it has lost, `tempo` each, for the
    /// channels the tick left quiet.
    wound: i32,
    channel: [Channel<'a>; MAX_CHANNELS],
    /// The high-pass and the limiter the mix ends in.
    shaper: Shaper,
    /// Mixed samples an output sample: 4 at `PCM_HZ`, 2 at twice it.
    oversample: u32,
    /// The output samples of the last `render`, how many there are
    /// and how many `next_sample` has given out.
    ready: [i16; RUN / 2],
    have: u8,
    given: u8,
}

impl<'a> Tune<'a> {
    /// `data` must be a converted module (`is_tune`).
    pub fn new(data: &'a [u8]) -> Self {
        let order_len = byte(data, 7) as usize;
        let patterns = byte(data, 8) as usize;
        let patterns_at = HEADER + order_len;
        let flags = byte(data, 17);
        // How many times the rate is twice the PCM clips', for the limiter.
        let octaves = if flags & FAST != 0 { 2 } else { (flags & TWICE) as u32 };
        let mut tune = Self {
            d: data,
            channels: (byte(data, 4) as usize).min(MAX_CHANNELS),
            order_len,
            patterns,
            instruments: byte(data, 9) as usize,
            patterns_at,
            instruments_at: patterns_at + PATTERN_ENTRY * patterns,
            speed: byte(data, 5).max(1),
            tempo: byte(data, 6).max(32),
            global: 64,
            order: 0,
            row: 0,
            rows: 0,
            cursor: 0,
            tick: 0,
            row_delay: 0,
            go: None,
            break_row: 0,
            loop_jump: false,
            pool: 0,
            looping: false,
            last: false,
            ended: false,
            cut: 0,
            wound: 0,
            channel: [Channel::default(); MAX_CHANNELS],
            mix_hz: if flags & FAST != 0 { FAST_MIX_HZ } else { MIX_HZ },
            amiga: flags & AMIGA_TABLE != 0,
            amiga_step: if flags & FAST != 0 { AMIGA_STEP_FAST } else { AMIGA_STEP },
            shaper: Shaper::new(byte(data, 10), byte(data, 11), octaves, core::array::from_fn(|i| word(data, 18 + 2 * i) as i16), byte(data, 16), flags & WIDE != 0, byte(data, 29)),
            oversample: if flags & (TWICE | FAST) == 0 { 4 } else { 2 },
            ready: [0; RUN / 2],
            have: 0,
            given: 0,
        };
        tune.enter(0, 0);
        tune
    }

    /// Play the tune round and round, as a tracker does, or once.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// The pool filled for the tick that `step_tick` has just begun.
    fn wind(&mut self) {
        self.pool += self.tick_pool();
        self.wound = self.pool;
    }

    /// What a tick's pool holds; see `mix_hz`.
    fn tick_pool(&self) -> i32 {
        (self.mix_hz * 5 / 2) as i32
    }

    /// Samples a second `next_sample` gives: `PCM_HZ`, or twice it or
    /// `FAST_HZ` for a clip that asks.
    pub fn rate(&self) -> u32 {
        self.mix_hz / self.oversample
    }

    /// The next sample, signed, in `STEP`ths of the 8-bit step a PCM
    /// clip has, so full scale is 127 of those; `None` when the tune
    /// has ended. Whole steps, unless the clip asks for the finer ones.
    ///
    /// In RAM in the firmware (the `ram` feature), the mixer with it,
    /// as is what else runs for every sample: `Player::level` and the
    /// two `fill`s under it, a predicted sample's `refill`, `fill` and
    /// `adpcm::decode_block`, and `Shaper::run`. From the flash this
    /// code is read through the 16 KB cache it shares with the
    /// samples, which push it out.
    #[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"), inline(never))]
    pub fn next_sample(&mut self) -> Option<i16> {
        if self.given == self.have {
            self.render();
            if self.have == 0 {
                return None;
            }
        }
        self.given += 1;
        Some(self.ready[self.given as usize - 1])
    }

    /// The output samples rendered and not yet given out, a render's
    /// worth made when there are none: empty when the tune has ended.
    /// `taken` says how many of them were used. For a caller that
    /// writes a run of them where they go (`clip::Player`), with no
    /// call a sample.
    #[inline(always)]
    pub fn block(&mut self) -> &[i16] {
        if self.given == self.have {
            self.render();
        }
        &self.ready[self.given as usize..self.have as usize]
    }

    /// The first `n` samples of `block` have been used.
    #[inline(always)]
    pub fn taken(&mut self, n: usize) {
        self.given += n as u8;
    }

    /// The next `RUN` mixed samples, or what there is of them before
    /// the tune ends, into `acc`: how many. In runs, each to the end
    /// of `acc` or of the tick, whichever is nearer, so a tick is at
    /// the mixed sample it was at when the mixer took one at a time.
    #[inline(always)]
    fn runs(&mut self, acc: &mut [i32; RUN]) -> usize {
        let mut filled = 0;
        while filled < RUN {
            if self.pool <= 0 {
                self.step_tick();
                self.wind();
            }
            if self.ended {
                break;
            }
            // A mixed sample takes `tempo` out of the pool, and the
            // last of a tick is the one that empties it.
            let (tempo, room) = (self.tempo as i32, RUN - filled);
            let n = if self.pool >= tempo * room as i32 { room } else { ((self.pool + tempo - 1) / tempo) as usize };
            self.pool -= tempo * n as i32;
            self.mix(&mut acc[filled..filled + n]);
            filled += n;
        }
        filled
    }

    /// The output samples of the next `RUN` mixed ones into `ready`:
    /// each the mean of its group, shaped, in order. None when the
    /// tune has ended, and a group the end falls in is dropped. The
    /// sums are here and nowhere else, 64 bytes of the stack.
    #[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"), inline(never))]
    fn render(&mut self) {
        let mut acc = [0; RUN];
        (self.have, self.given) = (0, 0);
        if self.ended {
            return;
        }
        let filled = self.runs(&mut acc);
        let four = self.oversample == 4;
        for group in acc[..filled].chunks_exact(self.oversample as usize) {
            let sum: i32 = group.iter().sum();
            self.ready[self.have as usize] = self.shaper.run(if four { sum / 4 } else { sum / 2 });
            self.have += 1;
        }
    }

    fn instrument(&self, number: u8) -> Option<Instrument<'a>> {
        let index = (number as usize).checked_sub(1).filter(|&i| i < self.instruments)?;
        Some(Instrument { d: self.d, at: self.instruments_at + INSTRUMENT_ENTRY * index })
    }

    /// The entry whose sample a note of instrument `number` plays: its
    /// own, or for an instrument of several samples what its table
    /// says (`Instrument::keyed`).
    fn keyed(&self, number: u8, note: u8) -> u8 {
        match self.instrument(number) {
            Some(i) if i.keyed() => (i.sample().0 as usize)
                .checked_sub(KEYS)
                .map_or(number, |keys| byte(self.d, keys + (note as usize).clamp(1, KEYS) - 1)),
            _ => number,
        }
    }

    /// Go to `row` of the pattern at `order`; the end of the list is
    /// the end of the tune, once the tick being played is over.
    fn enter(&mut self, order: usize, row: u16) {
        let mut order = order;
        if order >= self.order_len && self.looping {
            // Round again, from where the module says, if that is anywhere.
            order = Some(byte(self.d, 28) as usize).filter(|&o| o < self.order_len).unwrap_or(0);
        }
        let pattern = byte(self.d, HEADER + order) as usize;
        if order >= self.order_len || pattern >= self.patterns {
            self.last = true;
            return;
        }
        let entry = self.patterns_at + PATTERN_ENTRY * pattern;
        self.order = order;
        self.rows = word(self.d, entry);
        self.cursor = long(self.d, entry + 2) as usize;
        self.row = row.min(self.rows.saturating_sub(1));
        for _ in 0..self.row as usize * self.channels {
            self.read_cell();
        }
    }

    /// Unpack the cell at the cursor: a byte with its top bit set says
    /// which of the five fields follow, any other byte is the note and
    /// all four follow.
    fn read_cell(&mut self) -> Cell {
        let mut next = || {
            let b = byte(self.d, self.cursor);
            self.cursor += 1;
            b
        };
        let first = next();
        let mut cell = Cell::default();
        if first & 0x80 == 0 {
            cell.note = first;
            cell.instrument = next();
            cell.volume = next();
            cell.effect = next();
            cell.param = next();
            return cell;
        }
        if first & 1 != 0 {
            cell.note = next();
        }
        if first & 2 != 0 {
            cell.instrument = next();
        }
        if first & 4 != 0 {
            cell.volume = next();
        }
        if first & 8 != 0 {
            cell.effect = next();
        }
        if first & 16 != 0 {
            cell.param = next();
        }
        cell
    }

    /// One tick: a new row on the first of a row's ticks, the running
    /// effects on the others, then every channel's pitch and level.
    ///
    /// Not inlined: `next_sample` is in RAM in the firmware (the
    /// crate's `ram` feature), and this, once a tick, is 3 KB.
    #[inline(never)]
    fn step_tick(&mut self) {
        if self.last || self.ended {
            self.ended = true;
            return;
        }
        // The channels the mixer passed by, to where it would have
        // left them, before anything here asks where they are.
        let mixed = ((self.wound - self.pool) / self.tempo as i32) as u32;
        self.wound = self.pool;
        for ch in &mut self.channel[..self.channels] {
            if ch.quiet && ch.playing {
                pass(self.d, ch, mixed);
            }
        }
        if self.tick == 0 {
            if self.row >= self.rows {
                self.ended = true;
                return;
            }
            self.row_delay = 0;
            for c in 0..self.channels {
                let cell = self.read_cell();
                self.channel[c].cell = cell;
                self.row_start(c);
            }
            self.row += 1;
        } else {
            for c in 0..self.channels {
                self.row_tick(c);
            }
        }
        for c in 0..self.channels {
            self.settle(c);
        }
        self.tick += 1;
        // A row held by EE is its ticks so many times over, none of the
        // further ones a first tick.
        if self.tick >= self.speed as u16 * (1 + self.row_delay as u16) {
            self.tick = 0;
            let (looped, row) = (core::mem::take(&mut self.loop_jump), self.break_row);
            match self.go.take() {
                // A jump to where the tune has been is its loop: the end.
                Some((order, _)) if order <= self.order && !self.looping => self.last = true,
                Some((order, row)) => {
                    self.break_row = 0;
                    self.enter(order, row);
                }
                // A pattern loop, its row kept; held by EE it lands a
                // row on.
                None if looped => self.enter(self.order, row + (self.row_delay != 0) as u16),
                None if self.row >= self.rows => {
                    self.break_row = 0;
                    self.enter(self.order + 1, row);
                }
                None => {}
            }
        }
    }

    /// The note, instrument and volume of the channel's cell.
    fn trigger(&mut self, c: usize) {
        let cell = self.channel[c].cell;
        // An instrument's number is for the next note that begins: the
        // one sounding stays the channel's until then, and it is its
        // volume and envelope a number alone gives back. A note slid
        // to does not even remember the number (openmpt123,
        // 2026-11-26).
        let ch = &mut self.channel[c];
        let slides = cell.note != 0 && cell.note != KEY_OFF && cell.slides_to_note() && ch.playing;
        if cell.instrument != 0 && !slides {
            ch.named = cell.instrument;
        }
        if cell.note != 0 && cell.note != KEY_OFF && !slides {
            // Of the samples of the one named, the note's.
            let named = ch.named;
            self.channel[c].instrument = self.keyed(named, cell.note);
        }
        let instrument = self.instrument(self.channel[c].instrument);
        let amiga = self.amiga;
        let ch = &mut self.channel[c];
        if cell.instrument != 0 {
            ch.volume = instrument.map_or(0, |i| i.volume());
            ch.key_on = true;
            ch.envelope_frame = 0;
            ch.fade = FADE_FULL;
            ch.retrig_fine = 0;
            // And a tremor is over.
            ch.tremor = 0x20;
        }
        if cell.note == KEY_OFF {
            // A key-off beside a number leaves the volume the number gave.
            let volume = ch.volume;
            Self::key_off(ch, instrument);
            if cell.instrument != 0 {
                ch.volume = volume;
            }
        } else if cell.note != 0 {
            if slides {
                ch.target = instrument.map_or(0, |i| Self::period_of(amiga, cell.note, i));
            } else {
                // 9xx: start 256 samples in for each of xx.
                self.strike(c, cell.note, if cell.effect == 9 { cell.param as u32 * 256 } else { 0 });
                if let Some(i) = instrument {
                    let ch = &mut self.channel[c];
                    ch.vibrato_pos = 0;
                    ch.tremolo_pos = 0;
                    ch.auto_pos = 0;
                    ch.auto_depth = if i.auto_vibrato().0 == 0 { i.auto_vibrato().1 << 8 } else { 0 };
                }
            }
        }
        let ch = &mut self.channel[c];
        match cell.volume >> 4 {
            1..=5 => (ch.volume, ch.retrig_fine) = ((cell.volume - 0x10).min(64), 0),
            8 => ch.volume = ch.volume.saturating_sub(cell.volume & 0xF),
            9 => ch.volume = (ch.volume + (cell.volume & 0xF)).min(64),
            _ => {}
        }
    }

    fn period_of(amiga: bool, note: u8, i: Instrument) -> i32 {
        Self::period_at(amiga, note as i32 - 1 + i.relative(), i.finetune())
    }

    /// The period of a note, C-0 being 0, at a finetune of -128 to
    /// 127 (and beyond, for `arpeggio_period`). On the Amiga table as
    /// OpenMPT's `GetPeriodFromNote` has FastTracker's: the finetune
    /// in steps of eight, between two entries of the table. (On the
    /// linear table FastTracker has those steps too, and this has the
    /// finetune as it is: `PLAN.md`'s backlog.)
    #[inline(never)]
    fn period_at(amiga: bool, note: i32, finetune: i32) -> i32 {
        let note = note.clamp(0, 119);
        if !amiga {
            return PERIOD_MAX - note * SEMITONE - finetune / 2;
        }
        let finetune = finetune & !7;
        let at = (note % 12) * 8 + 8 + finetune / 16;
        let entry = |i: i32| AMIGA[i.clamp(0, 103) as usize] as i32;
        let (next, part) = if finetune < 0 { (at - 1, -finetune & 15) } else { (at + 1, finetune & 15) };
        ((entry(at) * (16 - part) + entry(next) * part) << 1) >> (note / 12)
    }

    /// The period an arpeggio's tick plays: FastTracker looks for the
    /// note the period is nearest to, slid or not, and plays the one
    /// `up` semitones above it, C-8 at most (OpenMPT's reading of it,
    /// `kFT2Arpeggio`). On both tables.
    #[inline(never)]
    fn arpeggio_period(amiga: bool, period: i32, finetune: i32, up: i32) -> i32 {
        // The first note whose period, half a semitone down, is not over this one.
        let (mut low, mut count) = (0, 120);
        while count > 0 {
            let half = count / 2;
            if Self::period_at(amiga, low + half, finetune + 64) > period {
                low += half + 1;
                count -= half + 1;
            } else {
                count = half;
            }
        }
        let note = low + up;
        let period = Self::period_at(amiga, note, finetune);
        if note >= ARPEGGIO_TOP { period.max(Self::period_at(amiga, ARPEGGIO_TOP, 0)) } else { period }
    }

    /// A note begun: the sample of the channel's instrument from
    /// `offset`, at the note's period. The volume, the envelope and
    /// the vibratos are the caller's.
    fn strike(&mut self, c: usize, note: u8, offset: u32) {
        let instrument = self.instrument(self.channel[c].instrument);
        self.cut += self.sounding_now(c);
        let amiga = self.amiga;
        let ch = &mut self.channel[c];
        let Some(i) = instrument else {
            ch.playing = false;
            return;
        };
        ch.fresh = true;
        (ch.data, ch.len, ch.loop_start, ch.loop_len) = i.sample();
        ch.kind = i.kind();
        let bytes = held(self.d, ch.data as usize, ch.len as usize, ch.kind);
        (ch.held, ch.words) = if ch.kind == Kind::Wide { (&[][..], words(bytes)) } else { (bytes, &[][..]) };
        ch.block = NO_BLOCK;
        ch.seeks = i.seeks();
        ch.loop_first = word(self.d, ch.data as usize + 6) as i16 as i32;
        ch.period = Self::period_of(amiga, note, i);
        ch.target = ch.period;
        ch.struck = ch.period;
        ch.frac = 0;
        ch.pos = offset;
        ch.playing = ch.pos < ch.len;
    }

    /// R on a tick of its row, as OpenMPT plays a FastTracker module's
    /// (`CSoundFile::RetrigNote`, 1.32): FastTracker's own replayer
    /// differs in corners, and OpenMPT is what a module is heard in
    /// today. The ticks are counted across rows and looked at before
    /// they are counted: at y of them the note last begun begins
    /// again, at its own period, and the volume changes as x says, in
    /// quarters of a step. On a row's first tick a cell that names an
    /// instrument sets the count to 1, a volume in the column (not 0)
    /// ends the tick there, and a note in the cell is not begun
    /// twice. Under a volume in the column x changes nothing.
    fn retrigger(&mut self, c: usize, first: bool) {
        let ch = &mut self.channel[c];
        let cell = ch.cell;
        let (x, y) = (ch.retrig_param >> 4, ch.retrig_param & 0xF);
        let sets = matches!(cell.volume, 0x10..=0x50);
        let mut count = ch.retrig_count;
        if first {
            if cell.instrument != 0 && cell.note != KEY_OFF {
                count = 1;
            }
            if sets && cell.volume != 0x10 {
                ch.retrig_count = count;
                return;
            }
        }
        let begins = count >= y && (!first || cell.note == 0 || cell.note == KEY_OFF);
        ch.retrig_count = if begins { 1 } else { count.wrapping_add(1) };
        if !begins {
            return;
        }
        if !sets {
            let v = ch.volume as i32 * 4 + ch.retrig_fine as i32;
            let v = match x {
                1..=5 => v - (4 << (x - 1)),
                6 => v * 10 / 16,
                7 => v / 2,
                9..=0xD => v + (4 << (x - 9)),
                0xE => v * 24 / 16,
                0xF => 2 * v,
                _ => v,
            }
            .clamp(0, 256);
            (ch.volume, ch.retrig_fine) = ((v / 4) as u8, (v % 4) as u8);
        }
        if ch.struck != 0 {
            self.cut += self.sounding_now(c);
            let ch = &mut self.channel[c];
            ch.fresh = true;
            ch.pos = 0;
            ch.frac = 0;
            ch.playing = ch.len > 0;
            ch.period = ch.struck;
            ch.target = ch.struck;
        }
    }

    /// What the channel adds to the mix at this moment. A note that
    /// starts where another is sounding takes its place within a
    /// sample, which is a step in the mix and a click; the step is
    /// carried on in `cut` and dies away there.
    fn sounding_now(&mut self, c: usize) -> i32 {
        let ch = &mut self.channel[c];
        // Nothing of a channel with no gain, and nothing decoded for it.
        if !ch.playing || ch.pos >= ch.len || ch.gain >> 8 == 0 {
            return 0;
        }
        let sample = if ch.kind == Kind::Packed {
            if ch.pos / BLOCK as u32 != ch.block {
                refill(self.d, ch, ch.pos / BLOCK as u32);
            }
            ch.blocks[ch.pos as usize % BLOCK] as i32
        } else {
            if ch.kind == Kind::Wide { wide_at(ch.words, ch.pos) } else { narrow_at(ch.held, ch.pos) }
        };
        (sample * (ch.gain >> 8)) >> 12
    }

    fn key_off(ch: &mut Channel<'_>, instrument: Option<Instrument>) {
        ch.key_on = false;
        // With no envelope there is nothing to fade along: silence.
        if instrument.is_none_or(|i| i.envelope_flags() & 1 == 0) {
            ch.volume = 0;
        }
    }

    /// The first tick of a row.
    fn row_start(&mut self, c: usize) {
        let cell = self.channel[c].cell;
        if cell.delay() == 0 {
            self.trigger(c);
        }
        let (hi, lo) = (cell.param >> 4, cell.param & 0xF);
        let order = self.order;
        let instrument = self.instrument(self.channel[c].instrument);
        let (longest, amiga) = (if self.amiga { AMIGA_MAX } else { PERIOD_MAX }, self.amiga);
        let ch = &mut self.channel[c];
        ch.arpeggio = 0;
        ch.tremolo = 0;
        if !cell.has_vibrato() {
            ch.vibrato = 0;
        }
        if cell.effect != RETRIGGER {
            ch.retrig_fine = 0;
        }
        match cell.volume >> 4 {
            0xA => ch.vibrato_speed = cell.volume & 0xF,
            0xB if cell.volume & 0xF != 0 => ch.vibrato_depth = cell.volume & 0xF,
            0xF if cell.volume & 0xF != 0 => ch.tone_speed = (cell.volume & 0xF) << 4,
            _ => {}
        }
        match cell.effect {
            1 if cell.param != 0 => ch.porta = cell.param,
            2 if cell.param != 0 => ch.porta_down = cell.param,
            3 if cell.param != 0 => ch.tone_speed = cell.param,
            4 => {
                if hi != 0 {
                    ch.vibrato_speed = hi;
                }
                if lo != 0 {
                    ch.vibrato_depth = lo;
                }
            }
            5 | 6 | 0xA if cell.param != 0 => ch.slide = cell.param,
            // The tremolo is on the row's first tick too, from where
            // it was.
            7 => {
                if hi != 0 {
                    ch.tremolo_speed = hi;
                }
                if lo != 0 {
                    ch.tremolo_depth = lo;
                }
                Self::tremolo(ch, false);
            }
            0xB => {
                self.go = Some((cell.param as usize, 0));
                self.break_row = 0;
            }
            0xC => ch.volume = cell.param.min(64),
            0xD => {
                self.break_row = (hi * 10 + lo) as u16;
                self.go = Some((self.go.map_or(order + 1, |g| g.0), self.break_row));
            }
            0xE => match hi {
                // E9 counts on R's counter, from the row's start.
                9 => ch.retrig_count = 0,
                // Each remembers what it last slid by.
                1 | 2 => {
                    ch.fine = Self::remembered(ch.fine, hi == 1, lo);
                    let by = Self::remembered_by(ch.fine, hi == 1) * 4;
                    ch.period = if hi == 1 { (ch.period - by).max(1) } else { (ch.period + by).min(longest) };
                }
                // The finetune of the note begun on the row, and of
                // that note alone (openmpt123, 2026-11-26).
                5 if cell.note != 0 && cell.note != KEY_OFF && ch.playing => {
                    if let Some(i) = instrument {
                        ch.period = Self::period_at(amiga, cell.note as i32 - 1 + i.relative(), (lo as i32 - 8) * 16);
                        (ch.target, ch.struck) = (ch.period, ch.period);
                    }
                }
                0xA => ch.volume = (ch.volume + lo).min(64),
                0xB => ch.volume = ch.volume.saturating_sub(lo),
                // E60 marks the row, E6x goes back to the mark x times:
                // the row is the break's, so a break before it in the
                // row goes to that row of its pattern.
                6 if lo == 0 => ch.loop_row = self.row as u8,
                6 => {
                    ch.loop_count = if ch.loop_count == 0 { lo } else { ch.loop_count - 1 };
                    if ch.loop_count != 0 {
                        self.break_row = ch.loop_row as u16;
                        self.loop_jump = true;
                        if let Some(go) = &mut self.go {
                            go.1 = self.break_row;
                        }
                    }
                }
                // The last channel's is the row's, nought too.
                0xE => self.row_delay = lo,
                _ => {}
            },
            0xF if cell.param >= 0x20 => self.tempo = cell.param,
            0xF if cell.param != 0 => self.speed = cell.param,
            // G.
            0x10 => self.global = cell.param.min(64),
            // H: the channel's own memory.
            GLOBAL_SLIDE if cell.param != 0 => ch.global_slide = cell.param,
            // K.
            0x14 if cell.param == 0 => Self::key_off(ch, instrument),
            // L: the envelope goes on from there.
            ENVELOPE_AT => ch.envelope_frame = cell.param as _,
            TREMOR => {
                if cell.param != 0 {
                    ch.tremor_param = cell.param;
                }
                ch.tremor |= 0x80;
            }
            // X1 and X2: E1 and E2 by a quarter of their step.
            EXTRA_FINE if hi == 1 || hi == 2 => {
                ch.extra_fine = Self::remembered(ch.extra_fine, hi == 1, lo);
                let by = Self::remembered_by(ch.extra_fine, hi == 1);
                ch.period = if hi == 1 { (ch.period - by).max(1) } else { (ch.period + by).min(longest) };
            }
            // x and y are remembered each by itself.
            RETRIGGER => {
                ch.retrig_param = (if hi != 0 { hi } else { ch.retrig_param >> 4 }) << 4 | if lo != 0 { lo } else { ch.retrig_param & 0xF };
                self.retrigger(c, true);
            }
            _ => {}
        }
    }

    /// The other ticks of a row.
    fn row_tick(&mut self, c: usize) {
        let cell = self.channel[c].cell;
        let (tick, speed) = (self.tick, self.speed);
        // A delayed note begins at its tick, and of a row held by EE
        // at that tick of every repeat.
        if cell.delay() != 0 && tick % speed.max(1) as u16 == cell.delay() as u16 {
            self.trigger(c);
        }
        let instrument = self.instrument(self.channel[c].instrument);
        let cut = self.sounding_now(c);
        let longest = if self.amiga { AMIGA_MAX } else { PERIOD_MAX };
        let ch = &mut self.channel[c];
        let lo = cell.param & 0xF;
        match cell.volume >> 4 {
            6 => ch.volume = ch.volume.saturating_sub(cell.volume & 0xF),
            7 => ch.volume = (ch.volume + (cell.volume & 0xF)).min(64),
            0xB => Self::vibrato(ch),
            0xF => Self::tone_portamento(ch),
            _ => {}
        }
        match cell.effect {
            0 if cell.param != 0 => ch.arpeggio = Self::arpeggio(cell.param, speed, tick),
            1 => ch.period = (ch.period - ch.porta as i32 * 4).max(1),
            2 => ch.period = (ch.period + ch.porta_down as i32 * 4).min(longest),
            3 => Self::tone_portamento(ch),
            4 => Self::vibrato(ch),
            5 => {
                Self::tone_portamento(ch);
                Self::volume_slide(ch);
            }
            6 => {
                Self::vibrato(ch);
                Self::volume_slide(ch);
            }
            7 => Self::tremolo(ch, true),
            0xA => Self::volume_slide(ch),
            0xE => match cell.param >> 4 {
                9 if lo != 0 && tick % lo as u16 == 0 => {
                    self.cut += cut;
                    ch.fresh = true;
                    ch.pos = 0;
                    ch.frac = 0;
                    ch.playing = ch.len > 0;
                    ch.retrig_count = 0;
                }
                9 if lo != 0 => ch.retrig_count = (tick % lo as u16) as u8,
                0xC if tick == lo as u16 => ch.volume = 0,
                _ => {}
            },
            // Up if there is an x, as the volume slide.
            GLOBAL_SLIDE => {
                let (up, down) = (ch.global_slide >> 4, ch.global_slide & 0xF);
                self.global = if up != 0 { (self.global + up).min(64) } else { self.global.saturating_sub(down) };
            }
            0x14 if tick == cell.param as u16 => Self::key_off(ch, instrument),
            // The count runs out and the other state begins, for one
            // tick more than its nibble says.
            TREMOR if ch.tremor & 0x80 != 0 => {
                ch.tremor = match ch.tremor & !0x20 {
                    0x80 => ch.tremor_param >> 4 | 0xC0,
                    0xC0 => ch.tremor_param & 0xF | 0x80,
                    count => count - 1,
                }
            }
            RETRIGGER => self.retrigger(c, false),
            _ => {}
        }
    }

    /// FastTracker's arpeggio: which of the three notes a tick plays
    /// is from the ticks left of the speed, the last of them being
    /// base, y, x. A row held by EE counts each repeat's ticks anew,
    /// and a repeat's first is no first tick: it has a step as well.
    /// One more than the step's semitones, 0 for the base's tick: a
    /// step of no semitones is still the nearest note's
    /// (`arpeggio_period`).
    fn arpeggio(param: u8, speed: u8, tick: u16) -> u8 {
        let speed = speed.max(1) as u16;
        match (speed - tick % speed) % 3 {
            0 => 0,
            1 => 1 + (param >> 4),
            _ => 1 + (param & 0xF),
        }
    }

    /// A fine slide's memory with `nibble` put in for its direction
    /// if it is not nought, and what the direction then slides by.
    fn remembered(memory: u8, up: bool, nibble: u8) -> u8 {
        match (nibble, up) {
            (0, _) => memory,
            (_, true) => memory & 0x0F | nibble << 4,
            (_, false) => memory & 0xF0 | nibble,
        }
    }

    fn remembered_by(memory: u8, up: bool) -> i32 {
        (if up { memory >> 4 } else { memory & 0xF }) as i32
    }

    /// The tremolo's part of this tick's volume: a sine of at most
    /// `depth * 127 / 8` 256ths either way, up first, and on the
    /// ticks but a row's first the cycle moved on after it.
    fn tremolo(ch: &mut Channel<'_>, on: bool) {
        let wave = TREMOLO[(ch.tremolo_pos & 31) as usize] as i16 * ch.tremolo_depth as i16 / 8;
        ch.tremolo = if ch.tremolo_pos & 32 == 0 { wave } else { -wave };
        if on {
            ch.tremolo_pos = ch.tremolo_pos.wrapping_add(ch.tremolo_speed);
        }
    }

    fn volume_slide(ch: &mut Channel<'_>) {
        let (up, down) = (ch.slide >> 4, ch.slide & 0xF);
        ch.volume = if up != 0 { (ch.volume + up).min(64) } else { ch.volume.saturating_sub(down) };
    }

    fn tone_portamento(ch: &mut Channel<'_>) {
        let speed = ch.tone_speed as i32 * 4;
        ch.period = if ch.period < ch.target { (ch.period + speed).min(ch.target) } else { (ch.period - speed).max(ch.target) };
    }

    /// A sine of at most `depth * 255 / 32` period units either way,
    /// down in pitch first, as FastTracker's goes (the instrument's
    /// own goes up first).
    fn vibrato(ch: &mut Channel<'_>) {
        let wave = (SINE[(ch.vibrato_pos >> 2 & 31) as usize] as i32 * ch.vibrato_depth as i32) >> 5;
        ch.vibrato = if ch.vibrato_pos & 0x80 == 0 { wave } else { -wave };
        ch.vibrato_pos = ch.vibrato_pos.wrapping_add(ch.vibrato_speed << 2);
    }

    /// The level of the instrument's envelope at the channel's frame,
    /// 0 to 64, and the frame moved on.
    fn envelope(ch: &mut Channel<'_>, instrument: Instrument) -> u32 {
        let flags = instrument.envelope_flags();
        let points = instrument.points();
        if flags & 1 == 0 || points == 0 {
            return 64;
        }
        let frame = ch.envelope_frame;
        let mut level = instrument.point(points - 1).1;
        for i in 1..points {
            let (x1, y1) = instrument.point(i);
            if frame < x1 {
                let (x0, y0) = instrument.point(i - 1);
                level = if x1 > x0 { (y0 * (x1 - frame) + y1 * (frame.max(x0) - x0)) / (x1 - x0) } else { y1 };
                break;
            }
        }
        let held = flags & 2 != 0 && ch.key_on && frame == instrument.point(instrument.sustain()).0;
        if !held {
            ch.envelope_frame = frame.saturating_add(1);
            let (from, to) = instrument.envelope_loop();
            if flags & 4 != 0 && ch.envelope_frame == instrument.point(to).0 {
                ch.envelope_frame = instrument.point(from).0;
            }
        }
        if !ch.key_on {
            ch.fade = ch.fade.saturating_sub(instrument.fade());
        }
        level
    }

    /// The channel's step through its sample and its level, from what
    /// the tick left.
    fn settle(&mut self, c: usize) {
        let instrument = self.instrument(self.channel[c].instrument);
        let global = self.global as u32;
        let (mix_hz, amiga, amiga_step) = (self.mix_hz, self.amiga, self.amiga_step);
        let ch = &mut self.channel[c];
        let level = match instrument {
            Some(i) => Self::envelope(ch, i),
            None => 64,
        };
        // The volume in 256ths: a tremolo's swing on it unless it is
        // nought, and nothing while a tremor is at its silence, which
        // it stays at after its last row.
        let mut volume = ch.volume as i32 * 4;
        if volume != 0 {
            volume = (volume + ch.tremolo as i32).clamp(0, 256);
        }
        if ch.tremor & 0xE0 == 0x80 {
            volume = 0;
        }
        #[cfg(test)]
        {
            ch.heard = volume;
        }
        ch.amp = ((volume as u32 * level * global / 256) * ch.fade / FADE_FULL) as i32;
        let gain = ch.amp << 8;
        if ch.fresh || !ch.playing {
            (ch.gain, ch.ramp, ch.fresh) = (gain, 0, false);
        } else if ch.gain != gain {
            ch.ramp = (mix_hz / RAMP_PER_SECOND).max(1);
            ch.gain_step = (gain - ch.gain) / ch.ramp as i32;
        }
        ch.quiet = ch.playing && ch.ramp == 0 && ch.gain >> 8 == 0;
        #[cfg(test)]
        if tests::EAGER.with(|e| e.get()) {
            ch.quiet = false;
        }
        ch.mixing = ch.playing && !ch.quiet;
        // The instrument's vibrato: a sine that grows to its depth over
        // the sweep, whatever the pattern says.
        let mut auto = 0;
        if let Some((sweep, depth, rate)) = instrument.map(|i| i.auto_vibrato())
            && depth != 0
        {
            let wave = ((SINE[(ch.auto_pos >> 2 & 31) as usize] as u32 * ch.auto_depth) >> 16) as i32;
            auto = if ch.auto_pos & 0x80 == 0 { -wave } else { wave };
            ch.auto_pos = ch.auto_pos.wrapping_add(rate);
            ch.auto_depth = (ch.auto_depth + (depth << 8) / sweep.max(1)).min(depth << 8);
        }
        let base = match ch.arpeggio {
            0 => ch.period,
            up => Self::arpeggio_period(amiga, ch.period, instrument.map_or(0, |i| i.finetune()), up as i32 - 1),
        };
        if amiga {
            let period = (base + ch.vibrato + auto).clamp(1, AMIGA_MAX);
            ch.sounding = period;
            ch.step = amiga_step / period as u32;
            return;
        }
        let period = (base + ch.vibrato + auto).clamp(1, PERIOD_MAX);
        ch.sounding = period;
        // The frequency is C4_HZ * 2^((PERIOD_C4 - period) / 768).
        let exponent = PERIOD_C4 - period;
        let (octave, rest) = (exponent.div_euclid(OCTAVE), exponent.rem_euclid(OCTAVE));
        // Hz in Q15, then a step in 16.16 at the mixing rate.
        // (Twice the Hz is under 2^31, so the division is one of 32 bits.)
        let hz = C4_HZ * POW[rest as usize] as u32;
        let step = ((hz << 1) / mix_hz) as u64;
        ch.step = (if octave >= 0 { step << octave.min(16) } else { step >> (-octave).min(16) }).min(u32::MAX as u64) as u32;
    }

    /// A run of mixed samples into `out`: the channels' sum, 32768
    /// for one channel at full level. A channel at a time over the
    /// run (`run`), the kind chosen once for it.
    fn mix(&mut self, out: &mut [i32]) {
        // What the notes that were cut short left behind, dying away.
        for sum in out.iter_mut() {
            *sum = self.cut;
            if self.cut != 0 {
                self.cut -= self.cut / CUT_DECAY + self.cut.signum();
            }
        }
        let d = self.d;
        for ch in &mut self.channel[..self.channels] {
            // A quiet channel adds nothing, and the tick moves it on.
            if !ch.mixing {
                continue;
            }
            // The sample after `pos`, which past a loop's end is its first.
            let (loop_start, loop_len) = (ch.loop_start, ch.loop_len);
            let after = move |pos: u32| if loop_len != 0 && pos + 1 >= loop_start + loop_len { pos + 1 - loop_len } else { pos + 1 };
            match ch.kind {
                Kind::Packed => run(ch, out, |ch, pos| {
                    // From the two blocks decoded: the flash has no sample
                    // to read. A new block once in `BLOCK` samples.
                    if pos / BLOCK as u32 != ch.block {
                        refill(d, ch, pos / BLOCK as u32);
                    }
                    let i = pos as usize % BLOCK;
                    let b = if loop_len != 0 && pos + 1 >= loop_start + loop_len { ch.loop_first } else { ch.blocks[i + 1] as i32 };
                    (ch.blocks[i] as i32, b)
                }),
                Kind::Wide => {
                    let words = ch.words;
                    run(ch, out, |_, pos| (wide_at(words, pos), wide_at(words, after(pos))))
                }
                Kind::Narrow => {
                    let held = ch.held;
                    run(ch, out, |_, pos| (narrow_at(held, pos), narrow_at(held, after(pos))))
                }
            }
        }
    }

    /// One mixed sample, as the mixer made it before it went by runs:
    /// every channel in turn, its state in the channel. What `mix`
    /// must give, sample for sample.
    #[cfg(test)]
    fn mix_one(&mut self) -> i32 {
        // What the notes that were cut short left behind, dying away.
        let mut sum = self.cut;
        if self.cut != 0 {
            self.cut -= self.cut / CUT_DECAY + self.cut.signum();
        }
        for ch in &mut self.channel[..self.channels] {
            // A quiet channel adds nothing, and the tick moves it on.
            if !ch.mixing {
                continue;
            }
            let (a, b) = if ch.kind == Kind::Packed {
                // From the two blocks decoded: the flash has no sample
                // to read. A new block once in `BLOCK` samples.
                if ch.pos / BLOCK as u32 != ch.block {
                    refill(self.d, ch, ch.pos / BLOCK as u32);
                }
                let i = ch.pos as usize % BLOCK;
                let b = if ch.loop_len != 0 && ch.pos + 1 >= ch.loop_start + ch.loop_len { ch.loop_first } else { ch.blocks[i + 1] as i32 };
                (ch.blocks[i] as i32, b)
            } else {
                let mut after = ch.pos + 1;
                if ch.loop_len != 0 && after >= ch.loop_start + ch.loop_len {
                    after -= ch.loop_len;
                }
                if ch.kind == Kind::Wide { (wide_at(ch.words, ch.pos), wide_at(ch.words, after)) } else { (narrow_at(ch.held, ch.pos), narrow_at(ch.held, after)) }
            };
            let sample = a + (((b - a) * (ch.frac >> 8) as i32) >> 8);
            if ch.ramp != 0 {
                ch.ramp -= 1;
                ch.gain = if ch.ramp == 0 { ch.amp << 8 } else { ch.gain + ch.gain_step };
            }
            sum += (sample * (ch.gain >> 8)) >> 12;
            let frac = ch.frac + (ch.step & 0xFFFF);
            ch.frac = frac & 0xFFFF;
            ch.pos = ch.pos.saturating_add((ch.step >> 16) + (frac >> 16));
            if ch.loop_len != 0 {
                let end = ch.loop_start + ch.loop_len;
                if ch.pos >= end {
                    ch.pos = ch.loop_start + (ch.pos - end) % ch.loop_len;
                }
            } else if ch.pos >= ch.len {
                ch.playing = false;
                ch.mixing = false;
            }
        }
        sum
    }
}

/// The whole tune's mix before the shaping, a sample an output sample,
/// scaled to fit `i16` with four channels at full level: for comparing
/// the player with another, from a test.
#[cfg(test)]
pub fn raw_mix(data: &[u8]) -> Vec<i16> {
    let mut t = Tune::new(data);
    let mut out = Vec::new();
    loop {
        let mut acc = [0; RUN];
        let filled = t.runs(&mut acc);
        for group in acc[..filled].chunks_exact(t.oversample as usize) {
            out.push((group.iter().sum::<i32>() / (t.oversample as i32 * 4)).clamp(-32768, 32767) as i16);
        }
        if filled < RUN {
            return out;
        }
    }
}

/// A short tune that ends, for another module's tests.
#[cfg(test)]
pub(crate) fn short_tune() -> Vec<u8> {
    tests::short_tune()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two notes of a square wave, eight rows.
    pub(super) fn short_tune() -> Vec<u8> {
        module(1, 6, 125, &[0], &[pattern(8, &[(0, (C4, 1, 0, 0, 0)), (4, (C4 + 7, 1, 0, 0, 0))])], &[square(32)])
    }

    /// One instrument of a test module.
    #[derive(Clone)]
    struct Ins {
        sample: Vec<i8>,
        loop_start: u32,
        loop_len: u32,
        volume: u8,
        finetune: i8,
        relative: i8,
        envelope: Vec<(u16, u16)>,
        flags: u8,
        sustain: u8,
        fade: u16,
        /// The auto-vibrato: sweep, depth, rate.
        auto: (u8, u8, u8),
        /// The sample at 16 bits, in place of `sample`, if it has any.
        wide: Vec<i16>,
        /// And whether that is kept predicted, and with seek points.
        packed: bool,
        seeks: bool,
    }

    std::thread_local! {
        /// Blocks decoded on this thread, for a test to count.
        pub(super) static FILLS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
        /// No channel is left quiet: the mixer plays the silent ones
        /// too, a sample at a time, for a test to compare with.
        pub(super) static EAGER: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    }

    use crate::adpcm::tests::{rung, table};

    /// A predicted sample as the converter writes one, with every
    /// predictor of `table` and every shift tried for each block, and
    /// the samples it decodes to.
    fn pack(wave: &[i16], loop_start: usize, looped: bool, seeks: bool) -> (Vec<u8>, Vec<i16>) {
        let (blocks, decoded) = crate::adpcm::tests::blocks(&table(), wave);
        let mut head = [0i16; 4];
        if looped {
            let first = loop_start / BLOCK * BLOCK;
            for (n, h) in head[..3].iter_mut().enumerate() {
                *h = if first > n { decoded[first - 1 - n] } else { 0 };
            }
            head[3] = decoded[loop_start];
        }
        let mut data: Vec<u8> = head.iter().flat_map(|h| h.to_le_bytes()).collect();
        data.extend(table());
        if seeks {
            for k in 1..=seek_points(wave.len()) {
                data.extend((1..=3).flat_map(|n| decoded[k * SEEK - n].to_le_bytes()));
            }
        }
        data.extend(blocks);
        (data, decoded)
    }

    /// A square wave of `period` samples, looped.
    fn square(period: usize) -> Ins {
        let sample = (0..period).map(|i| if i < period / 2 { 127 } else { -127 }).collect();
        Ins { sample, loop_start: 0, loop_len: period as u32, volume: 64, finetune: 0, relative: 0, envelope: vec![], flags: 0, sustain: 0, fade: 0, auto: (0, 0, 0), wide: vec![], packed: false, seeks: false }
    }

    /// A cell: note, instrument, volume column, effect, parameter.
    type Cell5 = (u8, u8, u8, u8, u8);
    type Row = Vec<Cell5>;
    /// A cell and the row it goes on.
    type At = (usize, Cell5);

    /// A converted module, as `tools/pack/src/xmc.rs` lays one out.
    fn module(channels: u8, speed: u8, tempo: u8, orders: &[u8], patterns: &[Vec<Row>], instruments: &[Ins]) -> Vec<u8> {
        let mut d = match () {
            _ if instruments.iter().any(|i| i.packed) => PACKED_MAGIC.to_vec(),
            _ if instruments.iter().any(|i| !i.wide.is_empty()) => WIDE_MAGIC.to_vec(),
            _ => MAGIC.to_vec(),
        };
        d.extend([channels, speed, tempo, orders.len() as u8, patterns.len() as u8, instruments.len() as u8, 0, 0]);
        d.extend(0u32.to_le_bytes());
        // 900 Hz at 8 kHz, second order, twice: `sox highpass 900` twice.
        d.extend([2, 0]);
        for k in [9883i16, -19766, 9883, -17075, 6072] {
            d.extend(k.to_le_bytes());
        }
        d.extend([0; 4]);
        assert_eq!(d.len(), HEADER);
        d.extend(orders);
        let tables = d.len() + PATTERN_ENTRY * patterns.len() + INSTRUMENT_ENTRY * instruments.len();
        let mut body = Vec::new();
        for rows in patterns {
            d.extend((rows.len() as u16).to_le_bytes());
            d.extend(((tables + body.len()) as u32).to_le_bytes());
            for row in rows {
                assert_eq!(row.len(), channels as usize);
                for &(note, instrument, volume, effect, param) in row {
                    // Packed where it is all nothing, plain otherwise, to try both.
                    if (note, instrument, volume, effect, param) == (0, 0, 0, 0, 0) {
                        body.push(0x80);
                    } else if instrument == 0 && volume == 0 && param == 0 && effect == 0 {
                        body.extend([0x81, note]);
                    } else {
                        body.extend([note, instrument, volume, effect, param]);
                    }
                }
            }
        }
        for i in instruments {
            // A sample of 16 bits at an even offset, as the converter
            // puts one.
            if !i.wide.is_empty() && !i.packed && !(tables + body.len()).is_multiple_of(2) {
                body.push(0);
            }
            d.extend(((tables + body.len()) as u32).to_le_bytes());
            d.extend((if i.wide.is_empty() { i.sample.len() } else { i.wide.len() } as u32).to_le_bytes());
            d.extend(i.loop_start.to_le_bytes());
            d.extend(i.loop_len.to_le_bytes());
            d.extend([i.volume, i.finetune as u8, i.relative as u8, i.flags, i.envelope.len() as u8, i.sustain, 0, 0]);
            d.extend(i.fade.to_le_bytes());
            d.extend([i.auto.0, i.auto.1, i.auto.2, if i.packed { 2 | (i.seeks as u8) << 2 } else { !i.wide.is_empty() as u8 }, 0, 0]);
            for p in 0..ENVELOPE_POINTS {
                let (x, y) = i.envelope.get(p).copied().unwrap_or((0, 0));
                d.extend(x.to_le_bytes());
                d.extend(y.to_le_bytes());
            }
            if i.wide.is_empty() {
                body.extend(i.sample.iter().map(|&s| s as u8));
            } else if i.packed {
                body.extend(pack(&i.wide, i.loop_start as usize, i.loop_len != 0, i.seeks).0);
            } else {
                body.extend(i.wide.iter().flat_map(|s| s.to_le_bytes()));
            }
        }
        assert_eq!(d.len(), tables);
        d.extend(body);
        d
    }

    const EMPTY: Cell5 = (0, 0, 0, 0, 0);

    /// `rows` rows of one channel, the given cells at the given rows.
    fn pattern(rows: usize, cells: &[At]) -> Vec<Row> {
        let mut p = vec![vec![EMPTY]; rows];
        for &(row, cell) in cells {
            p[row] = vec![cell];
        }
        p
    }

    /// A tune stepped tick by tick, mixing nothing: the first
    /// channel's (period, volume, playing) after each tick.
    fn ticks(data: &[u8]) -> Vec<(i32, u8, bool)> {
        let mut t = Tune::new(data);
        let mut seen = Vec::new();
        loop {
            t.step_tick();
            if t.ended {
                return seen;
            }
            let ch = &t.channel[0];
            seen.push((ch.sounding, ch.volume, ch.playing));
            assert!(seen.len() < 100_000, "a tune that does not end");
        }
    }

    /// The same of a channel, with whether its note began on the tick,
    /// again or for the first time: (period, volume, begun). A sample
    /// that begins is at its first sample, and nothing here moves it
    /// on but the mark put after each tick.
    fn strikes(data: &[u8], channel: usize) -> Vec<(i32, u8, bool)> {
        let mut t = Tune::new(data);
        let mut seen = Vec::new();
        loop {
            t.step_tick();
            if t.ended {
                return seen;
            }
            let ch = &mut t.channel[channel];
            seen.push((ch.sounding, ch.volume, ch.playing && ch.pos == 0));
            ch.pos = ch.pos.max(1);
            assert!(seen.len() < 100_000, "a tune that does not end");
        }
    }

    /// Not a test: with `CLIP_TICK_DUMP=file`, writes a line a tick of
    /// channel `CLIP_CHANNEL` of the converted module `CLIP_FILE`, or
    /// of the clip built in that `CLIP_INDEX` says: the period in
    /// semitones up from the lowest note, the volume, whether a note
    /// began, where its sample is in the clip, the row's volume
    /// column, effect and parameter, and the global volume; to hold
    /// the effects to another player's render tick by tick
    /// (`scripts/openmpt-channels.py`).
    #[test]
    #[ignore]
    fn dump_the_ticks() {
        let Ok(path) = std::env::var("CLIP_TICK_DUMP") else { return };
        let number = |name: &str| std::env::var(name).map_or(0, |i| i.parse().unwrap());
        let file = std::env::var("CLIP_FILE").ok().map(|path| std::fs::read(path).unwrap());
        let data = file.as_deref().unwrap_or_else(|| crate::melody::CLIP_DATA[number("CLIP_INDEX")]);
        let channel = number("CLIP_CHANNEL");
        let mut t = Tune::new(data);
        let mut out = String::new();
        loop {
            t.step_tick();
            if t.ended {
                break;
            }
            let ch = &mut t.channel[channel];
            let begun = ch.playing && ch.pos == 0;
            ch.pos = ch.pos.max(1);
            let Cell { volume, effect, param, .. } = ch.cell;
            // From C-0 on either table.
            let semitones = if t.amiga { 12.0 * (27392.0 / ch.sounding as f64).log2() } else { (PERIOD_MAX - ch.sounding) as f64 / SEMITONE as f64 };
            let (level, at) = (ch.heard as f64 / 4.0, ch.data);
            out += &format!("{semitones:.4} {level} {} {at} {volume:02X} {effect:02X} {param:02X} {}\n", begun as u8, t.global);
        }
        std::fs::write(path, out).unwrap();
    }

    /// The mix before the shaping, a sample a mixed sample.
    fn mixed(data: &[u8]) -> Vec<i32> {
        let mut t = Tune::new(data);
        let mut out = Vec::new();
        loop {
            let mut acc = [0; RUN];
            let filled = t.runs(&mut acc);
            out.extend_from_slice(&acc[..filled]);
            if filled < RUN {
                return out;
            }
        }
    }

    /// The same from the mixer that went a sample at a time.
    fn mixed_one(data: &[u8]) -> Vec<i32> {
        let mut t = Tune::new(data);
        let mut out = Vec::new();
        loop {
            if t.pool <= 0 {
                t.step_tick();
                t.wind();
            }
            if t.ended {
                return out;
            }
            t.pool -= t.tempo as i32;
            out.push(t.mix_one());
        }
    }

    /// Upward crossings of zero a second.
    fn pitch(samples: &[i32]) -> f64 {
        let ups = samples.windows(2).filter(|w| w[0] <= 0 && w[1] > 0).count();
        ups as f64 * MIX_HZ as f64 / samples.len() as f64
    }

    // Note numbers: C-4 is 49, and an instrument with no relative note
    // plays its sample at 8363 Hz there.
    const C4: u8 = 49;

    #[test]
    fn the_pitch_table_is_an_octave_of_semitones() {
        assert_eq!(POW[0], 32768);
        assert!((POW[384] as f64 - 32768.0 * 2f64.sqrt()).abs() < 1.5);
        assert!((POW[767] as f64 - 65536.0 / 2f64.powf(1.0 / 768.0)).abs() < 1.5);
        assert!(POW.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn a_note_plays_its_sample_at_its_pitch_for_its_rows() {
        // A square of 32 samples at C-4: 8363 / 32 = 261.3 Hz. Speed 6
        // at tempo 125 is 0.12 s a row.
        let rows = 16;
        let data = module(1, 6, 125, &[0], &[pattern(rows, &[(0, (C4, 1, 0, 0, 0))])], &[square(32)]);
        assert!(is_tune(&data) && !is_tune(b"XMC") && !is_tune(&[128; 64]));
        let out = mixed(&data);
        assert_eq!(out.len(), rows * 6 * (MIX_HZ as usize * 5 / 2 / 125));
        assert!((pitch(&out) - 8363.0 / 32.0).abs() < 1.0, "{}", pitch(&out));
        assert_eq!(out.iter().map(|s| s.abs()).max(), Some(127 * 256));
        // An octave up is twice that, a relative note of -12 undoes it,
        // and a finetune of 64 is half a semitone.
        let at = |note, relative, finetune| {
            let ins = Ins { relative, finetune, ..square(32) };
            pitch(&mixed(&module(1, 6, 125, &[0], &[pattern(rows, &[(0, (note, 1, 0, 0, 0))])], &[ins])))
        };
        assert!((at(C4 + 12, 0, 0) - 2.0 * 8363.0 / 32.0).abs() < 2.0);
        assert!((at(C4 + 12, -12, 0) - 8363.0 / 32.0).abs() < 1.0);
        assert!((at(C4, 0, 64) - 8363.0 / 32.0 * 2f64.powf(0.5 / 12.0)).abs() < 1.0);
        // A sample with no loop ends, and the channel with it.
        let once = Ins { loop_len: 0, ..square(320) };
        let out = mixed(&module(1, 6, 125, &[0], &[pattern(rows, &[(0, (C4, 1, 0, 0, 0))])], &[once]));
        let last = out.iter().rposition(|&s| s != 0).unwrap();
        assert!((last as f64 - 320.0 * MIX_HZ as f64 / 8363.0).abs() < 4.0, "{last}");
    }

    #[test]
    fn volume_comes_from_the_instrument_the_column_and_the_effects() {
        let ins = Ins { volume: 48, ..square(32) };
        let play = |cells: &[At]| ticks(&module(1, 4, 125, &[0], &[pattern(4, cells)], std::slice::from_ref(&ins)));
        let volumes = |cells: &[At]| play(cells).iter().map(|t| t.1).collect::<Vec<_>>();
        // The instrument's, then the column's, then C's.
        assert_eq!(volumes(&[(0, (C4, 1, 0, 0, 0)), (1, (0, 0, 0x30, 0, 0)), (2, (0, 0, 0, 0xC, 0x10))])[..12], [48, 48, 48, 48, 32, 32, 32, 32, 16, 16, 16, 16]);
        // A slides on every tick but the first, and remembers its parameter.
        assert_eq!(volumes(&[(0, (C4, 1, 0, 0xA, 0x02)), (1, (0, 0, 0, 0xA, 0))])[..8], [48, 46, 44, 42, 42, 40, 38, 36]);
        assert_eq!(volumes(&[(0, (C4, 1, 0x10, 0xA, 0x40))])[..4], [0, 4, 8, 12]);
        // The fine slides act once, on the first tick; EC cuts on its tick.
        assert_eq!(volumes(&[(0, (C4, 1, 0, 0xE, 0xB3)), (1, (0, 0, 0, 0xE, 0xA1)), (2, (0, 0, 0, 0xE, 0xC2))])[..12], [45, 45, 45, 45, 46, 46, 46, 46, 46, 46, 0, 0]);
        // An instrument number alone brings the volume back.
        assert_eq!(volumes(&[(0, (C4, 1, 0x10, 0, 0)), (1, (0, 1, 0, 0, 0))])[..8], [0, 0, 0, 0, 48, 48, 48, 48]);
        // The mixer's level follows: half volume, half the peak, and the
        // global volume on top.
        let peak = |cells: &[At]| {
            let out = mixed(&module(1, 4, 125, &[0], &[pattern(4, cells)], &[square(32)]));
            out.iter().map(|s| s.abs()).max().unwrap()
        };
        assert_eq!(peak(&[(0, (C4, 1, 0x30, 0, 0))]), 127 * 256 / 2);
        assert_eq!(peak(&[(0, (C4, 1, 0x30, 0x10, 32))]), 127 * 256 / 4);
    }

    #[test]
    fn h_slides_the_global_volume_as_openmpt_does() {
        const H: u8 = GLOBAL_SLIDE;
        // The global volume at each row's end, six ticks a row: each
        // list is a render of openmpt123's, read where a row begins.
        let globals = |cells: &[At]| {
            let data = module(1, 6, 125, &[0], &[pattern(16, cells)], &[square(32)]);
            let mut t = Tune::new(&data);
            let mut seen = Vec::new();
            for tick in 0..6 * cells.len() {
                let before = t.global;
                t.step_tick();
                // The first tick of a row slides nothing.
                assert!(tick % 6 != 0 || cells[tick / 6].1.3 != H || t.global == before);
                if tick % 6 == 5 {
                    seen.push(t.global);
                }
            }
            seen
        };
        // Up by x a tick, down by y, the last parameter for none, and
        // x before y.
        assert_eq!(
            globals(&[(0, (C4, 1, 0, 0x10, 0x20)), (1, (0, 0, 0, H, 0x10)), (2, (0, 0, 0, H, 0)), (3, (0, 0, 0, H, 0x02)), (4, (0, 0, 0, H, 0)), (5, (0, 0, 0, H, 0x34))]),
            [32, 37, 42, 32, 22, 37]
        );
        // With nothing to remember it does nothing; the volume stays
        // in 0 to 64, and comes up from nought.
        assert_eq!(
            globals(&[(0, (C4, 1, 0, 0x10, 0x3C)), (1, (0, 0, 0, H, 0)), (2, (0, 0, 0, H, 0xF0)), (3, (0, 0, 0, H, 0x0F)), (4, (0, 0, 0, H, 0x0F)), (5, (0, 0, 0, H, 0x10))]),
            [60, 60, 64, 0, 0, 5]
        );
        // A tune that fades in from G00 is heard.
        let out = mixed(&module(1, 6, 125, &[0], &[pattern(4, &[(0, (C4, 1, 0, 0x10, 0)), (1, (0, 0, 0, H, 0x40))])], &[square(32)]));
        assert!(out.iter().any(|&s| s != 0));
    }

    #[test]
    fn ee_holds_the_row_as_openmpt_does() {
        // Two channels, the second for an EE beside another effect:
        // the first's ticks as `r_retriggers…` writes them, a row
        // with its repeats between bars. Each is a render of
        // openmpt123's.
        const E: u8 = 0xE;
        const NO: Cell5 = (0, 0, 0, 0, 0);
        let play = |speed: u8, rows: &[(Cell5, Cell5)]| {
            let pattern: Vec<Row> = rows.iter().map(|&(a, b)| vec![a, b]).collect();
            strikes(&module(2, speed, 125, &[0], &[pattern], &[square(32)]), 0)
        };
        let text = |seen: &[(i32, u8, bool)]| seen.iter().map(|&(_, v, begun)| format!("{v}{}", if begun { "*" } else { "" })).collect::<Vec<_>>().join(" ");
        let held = |x: u8| (0, 0, 0, E, 0xE0 | x);
        // A row of three times its ticks, a slide on all but the first.
        assert_eq!(
            text(&play(6, &[((C4, 1, 0x30, 0, 0), NO), ((0, 0, 0, 0xA, 0x01), held(2)), (NO, NO)])[6..]),
            "32 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 15 15 15 15 15 15"
        );
        // The same in the column, with the EE in the cell itself.
        assert_eq!(text(&play(6, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0x61, E, 0xE1), NO), (NO, NO)])[6..]), "64 63 62 61 60 59 58 57 56 55 54 53 53 53 53 53 53 53");
        // The note of the row is begun once; E9's ticks are of the
        // whole count, and so are R's, which goes on in the next row.
        assert_eq!(text(&play(6, &[((C4, 1, 0, 0, 0), held(2)), (NO, NO)])), "64* 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64 64");
        assert_eq!(
            text(&play(6, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0, E, 0x94), held(2)), (NO, NO)])[6..]),
            "64 64 64 64 64* 64 64 64 64* 64 64 64 64* 64 64 64 64* 64 64 64 64 64 64 64"
        );
        assert_eq!(
            text(&play(6, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0, RETRIGGER, 0x84), held(2)), ((0, 0, 0, RETRIGGER, 0x84), NO)])[6..]),
            "64 64 64 64 64* 64 64 64 64* 64 64 64 64* 64 64 64 64* 64 64 64 64* 64 64 64"
        );
        // A note delayed begins at its tick of every repeat, and one
        // delayed past the speed never.
        assert_eq!(
            text(&play(6, &[((C4, 1, 0x20, 0, 0), NO), ((C4 + 12, 1, 0, E, 0xD2), held(2)), (NO, NO)])[6..]),
            "16 16 64* 64 64 64 64 64 64* 64 64 64 64 64 64* 64 64 64 64 64 64 64 64 64"
        );
        assert_eq!(text(&play(3, &[((C4, 1, 0x20, 0, 0), NO), ((C4 + 12, 1, 0, E, 0xD4), held(2)), (NO, NO)])[3..]), "16 16 16 16 16 16 16 16 16 16 16 16");
        // A cut and a key-off past the speed meet their ticks.
        assert_eq!(text(&play(3, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0, E, 0xC4), held(2)), (NO, NO)])[3..]), "64 64 64 64 0 0 0 0 0 0 0 0");
        let off = play(3, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0, 0x14, 7), held(2)), (NO, NO)]);
        assert_eq!(off.len(), 15);
        // The last channel's EE is the row's, and one of nought
        // takes the delay away.
        assert_eq!(play(3, &[((C4, 1, 0, E, 0xE1), held(3))]).len(), 12);
        assert_eq!(play(3, &[((C4, 1, 0, E, 0xE3), held(1))]).len(), 6);
        assert_eq!(play(3, &[((C4, 1, 0, E, 0xE1), held(0))]).len(), 3);
        // The arpeggio's step is from the ticks left of the speed: a
        // repeat's first tick has one, the row's has none. The ticks
        // an octave up, of 0C0 under EE2.
        let up = |speed: u8| {
            let seen = play(speed, &[((C4, 1, 0, 0, 0), NO), ((0, 0, 0, 0, 0xC0), held(2))]);
            let base = seen[0].0;
            (0..3 * speed as usize).filter(|&t| seen[speed as usize + t].0 != base).collect::<Vec<_>>()
        };
        assert_eq!(up(4), [3, 4, 7, 8, 11]);
        assert_eq!(up(5), [1, 4, 6, 9, 11, 14]);
        assert_eq!(up(6), [2, 5, 8, 11, 14, 17]);
    }

    #[test]
    fn e6_loops_the_pattern_as_openmpt_does() {
        // Two channels and rows of three ticks; a row of the first
        // channel is told by the volume its column sets, and what is
        // listed is the rows as they are played, a star where the
        // note begins. Each list is a render of openmpt123's.
        const E: u8 = 0xE;
        const NO: Cell5 = (0, 0, 0, 0, 0);
        const NOTE: (Cell5, Cell5) = ((C4, 1, 0, 0, 0), NO);
        let played = |patterns: &[&[(Cell5, Cell5)]]| {
            let patterns: Vec<Vec<Row>> = patterns.iter().map(|rows| rows.iter().map(|&(a, b)| vec![a, b]).collect()).collect();
            let orders: Vec<u8> = (0..patterns.len() as u8).collect();
            let seen = strikes(&module(2, 3, 125, &orders, &patterns, &[square(32)]), 0);
            seen.chunks(3).map(|row| format!("{}{}", row[0].1, if row[0].2 { "*" } else { "" })).collect::<Vec<_>>().join(" ")
        };
        // A row marked `v`, with an effect in the first channel or the second.
        let own = |v: u8, effect: u8, param: u8| ((0, 0, 0x10 + v, effect, param), NO);
        let other = |v: u8, effect: u8, param: u8| ((0, 0, 0x10 + v, 0, 0), (0, 0, 0, effect, param));
        let row = |v: u8| own(v, 0, 0);
        // Back to the mark x times; with no mark, to the top.
        assert_eq!(played(&[&[NOTE, own(4, E, 0x60), row(8), own(12, E, 0x62), row(16)]]), "64* 4 8 12 4 8 12 4 8 12 16");
        assert_eq!(played(&[&[NOTE, row(4), own(8, E, 0x61), row(12)]]), "64* 4 8 64* 4 8 12");
        // Each channel has its mark and its count: of two that go
        // back on a row the last channel's mark is gone to, and one
        // loop inside another is played again each time round.
        assert_eq!(played(&[&[NOTE, own(4, E, 0x60), other(8, E, 0x60), ((0, 0, 0x1C, E, 0x61), (0, 0, 0, E, 0x61)), row(16)]]), "64* 4 8 12 8 12 16");
        assert_eq!(
            played(&[&[NOTE, other(4, E, 0x60), own(8, E, 0x60), own(12, E, 0x61), other(16, E, 0x62), row(20), row(24)]]),
            "64* 4 8 12 8 12 16 4 8 12 8 12 16 4 8 12 8 12 16 20 24"
        );
        // A loop taken on a row held by EE lands a row past its mark
        // (the held row is listed twice, being two rows long).
        assert_eq!(played(&[&[NOTE, own(4, E, 0x60), ((0, 0, 0x18, E, 0x61), (0, 0, 0, E, 0xE1)), row(12)]]), "64* 4 8 8 8 8 12");
        assert_eq!(played(&[&[NOTE, own(4, E, 0x60), row(8), ((0, 0, 0x1C, E, 0x61), (0, 0, 0, E, 0xE1)), row(16)]]), "64* 4 8 12 12 8 12 12 16");
        // The mark is kept into the next pattern.
        let next = [row(4), row(8), row(12), row(16), row(20)];
        assert_eq!(
            played(&[&[NOTE, row(32), own(33, E, 0x60), row(34), other(35, 0xD, 0)], &[row(4), row(8), row(12), own(16, E, 0x61), row(20)]]),
            "64* 32 33 34 35 4 8 12 16 12 16 20"
        );
        // With a break on the row the tune goes on, to the row the
        // later of the two named.
        assert_eq!(played(&[&[NOTE, own(32, E, 0x60), row(33), ((0, 0, 0x10 + 34, E, 0x61), (0, 0, 0, 0xD, 0)), row(35)], &next]), "64* 32 33 34 4 8 12 16 20");
        assert_eq!(played(&[&[NOTE, other(32, E, 0x60), row(33), ((0, 0, 0x10 + 34, 0xD, 0), (0, 0, 0, E, 0x61)), row(35)], &next]), "64* 32 33 34 8 12 16 20");
        // A pattern that ends after a loop was taken begins the next
        // at the loop's mark; a mark alone does not do it, nor a loop
        // with a break after it.
        assert_eq!(played(&[&[NOTE, row(32), own(33, E, 0x60), own(34, E, 0x61)], &next]), "64* 32 33 34 33 34 12 16 20");
        assert_eq!(played(&[&[NOTE, row(32), own(33, E, 0x60), row(34)], &next]), "64* 32 33 34 4 8 12 16 20");
        assert_eq!(played(&[&[NOTE, row(32), own(33, E, 0x60), own(34, E, 0x61), other(35, 0xD, 0)], &next]), "64* 32 33 34 33 34 35 4 8 12 16 20");
    }

    #[test]
    fn r_retriggers_and_changes_the_volume_as_openmpt_does() {
        // A tick as its volume, with a star where the note begins
        // (again, or as the row's own); a bar between rows. What is
        // expected is what openmpt123 0.8.10 renders of the same rows
        // (2026-11-26, a FastTracker module with its compatible
        // playback), each read off a sample that is a ramp.
        const R: u8 = RETRIGGER;
        let roll = |speed: u8, cells: &[At]| {
            let rows = cells.iter().map(|c| c.0).max().unwrap() + 1;
            let seen = strikes(&module(1, speed, 125, &[0], &[pattern(rows, cells)], &[square(32)]), 0);
            let row = |r: &[(i32, u8, bool)]| r.iter().map(|&(_, v, begun)| format!("{v}{}", if begun { "*" } else { "" })).collect::<Vec<_>>().join(" ");
            seen.chunks(speed as usize).map(row).collect::<Vec<_>>().join(" | ")
        };
        // Once in y ticks, the count carried from row to row; x and y
        // each remembered.
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0, R, 0x12)), (1, (0, 0, 0, R, 0)), (2, (0, 0, 0, R, 0x03)), (3, (0, 0, 0, R, 0x20))]),
            "64* 63* 63 62* 62 61* | 61 60* 60 59* 59 58* | 58 58 57* 57 57 56* | 56 56 54* 54 54 52*"
        );
        // With nothing to remember: every tick, the volume left alone.
        assert_eq!(roll(6, &[(0, (C4, 1, 0, R, 0))]), "64* 64* 64* 64* 64* 64*");
        // A cell that names an instrument starts the count at 1, a
        // note alone leaves it and is not begun twice.
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0, R, 0x14)), (1, (0, 1, 0, R, 0x14)), (2, (0, 0, 0, R, 0x14))]),
            "64* 64 64 63* 63 63 | 64 64 64 63* 63 63 | 63 62* 62 62 62 61*"
        );
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0, R, 0x84)), (1, (C4 + 2, 0, 0, R, 0x84)), (2, (0, 0, 0, R, 0x84))]),
            "64* 64 64 64* 64 64 | 64* 64* 64 64 64 64* | 64 64 64 64* 64 64"
        );
        // Not so on a row with no R, instrument or not: the count is
        // where the last R left it.
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0, R, 0x83)), (1, (C4, 1, 0x20, 0, 0)), (2, (C4, 0, 0, R, 0x93)), (3, (0, 0, 0, 0, 0)), (4, (0, 0, 0, R, 0x93))]),
            "64* 64 64* 64 64 64* | 16* 16 16 16 16 16 | 16* 16 17* 17 17 18* | 18 18 18 18 18 18 | 18 18 19* 19 19 20*"
        );
        // Across rows of another length.
        let cells: Vec<At> = (0..8).map(|r| (r, (if r == 0 { C4 } else { 0 }, (r == 0) as u8, 0, R, 0x85))).collect();
        assert_eq!(roll(3, &cells), "64* 64 64 | 64 64* 64 | 64 64 64 | 64* 64 64 | 64 64 64* | 64 64 64 | 64 64* 64 | 64 64 64");
        // What each x does, a tick at a time and not on the first,
        // which is only counted: from 64 down, from 3 up. 6 is five
        // eighths and E three halves, in quarters of a step, of which
        // the whole steps are seen here.
        let each = |from: u8, x: u8| roll(6, &[(0, (C4, 1, from, 0, 0)), (1, (0, 0, 0, R, x << 4 | 1))]).split(" | ").nth(1).unwrap().replace('*', "");
        let down = ["64 63 62 61 60 59", "64 62 60 58 56 54", "64 60 56 52 48 44", "64 56 48 40 32 24", "64 48 32 16 0 0", "64 40 25 15 9 5", "64 32 16 8 4 2"];
        let up = ["3 3 3 3 3 3", "3 4 5 6 7 8", "3 5 7 9 11 13", "3 7 11 15 19 23", "3 11 19 27 35 43", "3 19 35 51 64 64", "3 4 6 10 15 22", "3 6 12 24 48 64"];
        for (x, want) in down.iter().enumerate() {
            assert_eq!(each(0x50, x as u8 + 1), *want, "x {}", x + 1);
        }
        for (x, want) in up.iter().enumerate() {
            assert_eq!(each(0x13, x as u8 + 8), *want, "x {}", x + 8);
        }
        // A volume in the row's column: the first tick is not counted,
        // and x changes nothing.
        let column = |volume: u8, param: u8| roll(6, &[(0, (C4, 1, 0x28, 0, 0)), (1, (0, 0, volume, R, param)), (2, (0, 0, volume, R, param))]);
        assert_eq!(column(0x30, 0x32), "24* 24 24 24 24 24 | 32 32 32 32* 32 32* | 32 32 32* 32 32* 32");
        // Anything else in the column: counted, and x has its say.
        assert_eq!(column(0xA3, 0x32), "24* 24 24 24 24 24 | 24 24 20* 20 16* 16 | 12* 12 8* 8 4* 4");
        assert_eq!(column(0x62, 0x32), "24* 24 24 24 24 24 | 24 22 16* 14 8* 6 | 2* 0 0* 0 0* 0");
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0x12, 0, 0)), (1, (0, 0, 0x85, R, 0x92)), (2, (0, 0, 0, R, 0x92))]),
            "2* 2 2 2 2 2 | 0 0 1* 1 2* 2 | 3* 3 4* 4 5* 5"
        );
        // E9 counts on the same counter, and leaves it at its last
        // retrigger.
        assert_eq!(
            roll(6, &[(0, (C4, 1, 0, R, 0x84)), (1, (0, 0, 0, 0xE, 0x91)), (2, (0, 0, 0, R, 0x84))]),
            "64* 64 64 64* 64 64 | 64 64* 64* 64* 64* 64* | 64 64 64 64 64* 64"
        );
        // The note begins again at its own period, whatever a
        // portamento has made of it since.
        let seen = strikes(&module(1, 6, 125, &[0], &[pattern(3, &[(0, (C4, 1, 0, 0, 0)), (1, (0, 0, 0, 2, 0x20)), (2, (0, 0, 0, R, 0x83))])], &[square(32)]), 0);
        let slid = PERIOD_C4 + 5 * 0x20 * 4;
        assert_eq!(seen[12..].iter().map(|t| (t.0, t.2)).collect::<Vec<_>>(), [(slid, false), (slid, false), (slid, false), (PERIOD_C4, true), (PERIOD_C4, false), (PERIOD_C4, false)]);
    }

    #[test]
    fn r_in_any_cell_plays_to_the_end_and_the_same_every_way() {
        // Cells of no meaning, half of them with R: any x and y, with
        // a note or none before it, a key-off, an instrument with no
        // sample or one the module has not, anything in the volume
        // column, E9 and the volume effects between. Each tune ends,
        // and is the same mixed by runs as a sample at a time, with
        // its silent channels passed by as with them played, and
        // with its 16-bit sample predicted as with it plain.
        let wave = rung(700);
        let (_, decoded) = pack(&wave, 100, true, false);
        let instruments = |packed: bool| {
            let sixteen = if packed { Ins { wide: wave.clone(), packed: true, loop_start: 100, loop_len: 600, ..square(32) } } else { Ins { wide: decoded.clone(), loop_start: 100, loop_len: 600, ..square(32) } };
            [square(32), sixteen, Ins { sample: vec![], loop_len: 0, ..square(32) }, Ins { sample: (0..40).map(|i| (i * 3) as i8).collect(), loop_len: 0, volume: 40, ..square(32) }]
        };
        let mut random = 2026u32;
        let mut next = |n: u32| {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (random >> 16) % n
        };
        let mut sounding = 0;
        for tune in 0..150 {
            let rows: Vec<Row> = (0..24)
                .map(|_| {
                    (0..3)
                        .map(|_| {
                            let note = [0, 0, C4, C4 + 7, C4 - 12, KEY_OFF][next(6) as usize];
                            let instrument = [0, 0, 1, 2, 3, 4, 5][next(7) as usize];
                            let volume = if next(2) == 0 { 0 } else { next(256) as u8 };
                            let effect = [RETRIGGER, RETRIGGER, RETRIGGER, 0xE, 0xC, 0xA, 3, 0][next(8) as usize];
                            let param = match effect {
                                0xE => [0x90, 0xD0, 0xC0][next(3) as usize] | next(4) as u8,
                                0xC => next(65) as u8,
                                0 => 0,
                                _ => next(256) as u8,
                            };
                            (note, instrument, volume, effect, param)
                        })
                        .collect()
                })
                .collect();
            let speed = 1 + next(7) as u8;
            let clip = |packed: bool| module(3, speed, 125, &[0], core::slice::from_ref(&rows), &instruments(packed));
            let (plain, packed) = (clip(false), clip(true));
            let whole = mixed(&plain);
            assert_eq!(whole.len(), 24 * speed as usize * (MIX_HZ as usize * 5 / 2 / 125), "tune {tune}");
            assert!(whole == mixed_one(&plain), "tune {tune}: by runs");
            assert!(whole == mixed(&packed), "tune {tune}: predicted");
            let play = |eager: bool| {
                EAGER.with(|e| e.set(eager));
                let mut t = Tune::new(&packed);
                let out: Vec<i16> = core::iter::from_fn(|| t.next_sample()).collect();
                EAGER.with(|e| e.set(false));
                out
            };
            assert!(play(true) == play(false), "tune {tune}: passed by");
            sounding += whole.iter().any(|&s| s.abs() > 1000) as usize;
        }
        assert!(sounding > 140, "{sounding} of the tunes sound");
    }

    #[test]
    fn pitch_effects_move_the_period_as_fasttracker_does() {
        let play = |speed, cells: &[At]| {
            ticks(&module(1, speed, 125, &[0], &[pattern(4, cells)], &[square(32)])).iter().map(|t| t.0).collect::<Vec<_>>()
        };
        let base = PERIOD_C4;
        // Arpeggio 0xy at speed 4: base, base, y, x; at speed 6: base, y, x twice.
        let s = SEMITONE;
        assert_eq!(play(4, &[(0, (C4, 1, 0, 0, 0x47))])[..4], [base, base, base - 7 * s, base - 4 * s]);
        assert_eq!(play(6, &[(0, (C4, 1, 0, 0, 0x47))])[..6], [base, base - 7 * s, base - 4 * s, base, base - 7 * s, base - 4 * s]);
        assert_eq!(play(5, &[(0, (C4, 1, 0, 0, 0x47))])[..5], [base, base - 4 * s, base, base - 7 * s, base - 4 * s]);
        // And the row after it is the plain note again.
        assert_eq!(play(4, &[(0, (C4, 1, 0, 0, 0x47))])[4], base);
        // A note that has slid is first taken to the nearest note:
        // 220 units up is 28 from the note 3 semitones up (openmpt123,
        // 2026-11-26).
        let near = base - 3 * s;
        assert_eq!(play(6, &[(0, (C4, 1, 0, 1, 0x0B)), (1, (0, 0, 0, 0, 0x37))])[5..12], [base - 220, base - 220, near - 7 * s, near - 3 * s, base - 220, near - 7 * s, near - 3 * s]);
        // Portamento: four units a tick for each of the parameter, not
        // on the first tick, remembered, each direction by itself: a
        // 200 after only a 1xx has nothing to remember (openmpt123,
        // 2026-11-18; Crush leans on it).
        assert_eq!(play(4, &[(0, (C4, 1, 0, 1, 3)), (1, (0, 0, 0, 1, 0))])[..8], [base, base - 12, base - 24, base - 36, base - 36, base - 48, base - 60, base - 72]);
        assert_eq!(play(4, &[(0, (C4, 1, 0, 1, 3)), (1, (0, 0, 0, 2, 0))])[..8], [base, base - 12, base - 24, base - 36, base - 36, base - 36, base - 36, base - 36]);
        assert_eq!(play(4, &[(0, (C4, 1, 0, 2, 1)), (1, (0, 0, 0, 1, 3)), (2, (0, 0, 0, 2, 0))])[8..12], [base - 24, base - 20, base - 16, base - 12]);
        assert_eq!(play(4, &[(0, (C4, 1, 0, 0xE, 0x12)), (1, (0, 0, 0, 0xE, 0x21))])[..8], [base - 8, base - 8, base - 8, base - 8, base - 4, base - 4, base - 4, base - 4]);
        // Tone portamento goes to the note and stops there; the note
        // does not start the sample again.
        let up = play(4, &[(0, (C4, 1, 0, 0, 0)), (1, (C4 + 1, 0, 0, 3, 8)), (2, (0, 0, 0, 3, 0))]);
        assert_eq!(up[4..12], [base, base - 32, base - 64, base - 64, base - 64, base - 64, base - 64, base - 64]);
        // An instrument's own vibrato: no effect asks for it, it grows to
        // its depth over the sweep and swings both ways.
        let auto = Ins { auto: (4, 16, 32), ..square(32) };
        let seen: Vec<i32> = ticks(&module(1, 16, 125, &[0], &[pattern(1, &[(0, (C4, 1, 0, 0, 0))])], &[auto])).iter().map(|t| t.0 - base).collect();
        assert_eq!(seen[0], 0);
        assert!(seen[1] < 0 && seen[1] > seen[2], "{seen:?}");
        assert_eq!((seen.iter().min(), seen.iter().max()), (Some(&-15), Some(&15)), "{seen:?}");
        // Vibrato swings about the note, down in pitch first (the
        // period up; openmpt123 on Crush's lead, 2026-11-18), and is
        // gone with it.
        let vib = play(6, &[(0, (C4, 1, 0, 4, 0xF4)), (1, (0, 0, 0, 0, 0))]);
        assert_eq!(vib[0], base);
        assert!(vib[1..6].iter().all(|&p| (p - base).abs() <= 4 * 255 / 32) && vib[2] > base && vib[3..6].iter().any(|&p| p < base), "{:?}", &vib[..6]);
        assert_eq!(vib[6], base);
    }

    /// A module on the Amiga frequency table, as OpenMPT plays a
    /// FastTracker one (openmpt123 0.8.10, 2026-11-26: where a ramp
    /// was at each tick's end is where these periods put it, to the
    /// ramp's step).
    #[test]
    fn the_amiga_table_is_played_as_openmpt_does() {
        let amiga = |mut d: Vec<u8>| {
            d[17] |= AMIGA_TABLE;
            d
        };
        let play = |speed, cells: &[At], ins: Ins| ticks(&amiga(module(1, speed, 125, &[0], &[pattern(4, cells)], &[ins]))).iter().map(|t| t.0).collect::<Vec<_>>();
        let note = |note: u8, finetune: i8, relative: i8| play(1, &[(0, (note, 1, 0, 0, 0))], Ins { finetune, relative, ..square(32) })[0];
        // A note's period is the table's, halved an octave.
        for (n, period) in [(C4, 1712), (C4 + 1, 1616), (C4 + 7, 1140), (C4 + 31, 285), (C4 + 40, 169), (C4 - 12, 3424), (C4 - 30, 9664), (1, 27392)] {
            assert_eq!(note(n, 0, 0), period, "note {n}");
        }
        // The finetune in steps of eight, between two entries; the
        // relative note is the note's.
        for (finetune, period) in [(127, 1622), (120, 1622), (9, 1706), (8, 1706), (7, 1712), (-1, 1718), (-8, 1718), (-9, 1724), (-128, 1814), (64, 1664), (-64, 1762)] {
            assert_eq!(note(C4, finetune, 0), period, "finetune {finetune}");
        }
        assert_eq!(note(C4, 0, 7), 1140);
        // C-4 is the linear table's C-4, and half the period an octave.
        let step = |d: Vec<u8>| {
            let mut t = Tune::new(&d);
            t.step_tick();
            t.channel[0].step
        };
        let tune = |note| module(1, 6, 125, &[0], &[pattern(1, &[(0, (note, 1, 0, 0, 0))])], &[square(32)]);
        // (To a part in sixty thousand: the linear table's is rounded down.)
        let same = |a: u32, b: u32| a.abs_diff(b) <= 4;
        assert!(same(step(amiga(tune(C4))), step(tune(C4))));
        assert!(same(step(amiga(tune(C4 + 24))), 4 * step(tune(C4))));
        assert!(same(step(amiga(tune(C4 - 12))), step(tune(C4)) / 2));
        // The slides move the period by what they do on the other table.
        let slides = play(4, &[(0, (C4, 1, 0, 1, 0x10)), (1, (0, 0, 0, 2, 0x08)), (2, (0, 0, 0, 0xE, 0x13)), (3, (0, 0, 0, 0xE, 0x25))], square(32));
        assert_eq!(slides[..16], [1712, 1648, 1584, 1520, 1520, 1552, 1584, 1616, 1604, 1604, 1604, 1604, 1624, 1624, 1624, 1624]);
        let to = play(4, &[(0, (C4, 1, 0, 0, 0)), (1, (C4 + 1, 0, 0, 3, 0x10)), (2, (0, 0, 0, 3, 0))], square(32));
        assert_eq!(to[4..9], [1712, 1648, 1616, 1616, 1616]);
        // Down to the longest period and no further.
        let low = play(6, &[(0, (1, 1, 0, 2, 0xFF)), (1, (0, 0, 0, 2, 0))], square(32));
        assert_eq!(low[..7], [27392, 28412, 29432, 30452, 31472, AMIGA_MAX, AMIGA_MAX]);
        // An arpeggio's notes are the table's: D#4 and G-4 over C-4.
        assert_eq!(play(6, &[(0, (C4, 1, 0, 0, 0x37))], square(32))[..6], [1712, 1140, 1440, 1712, 1140, 1440]);
        // Over a note that has slid, the nearest note's: 1332 is E-4's
        // 1356, and G-4 and B-4 are 3 and 7 above it.
        assert_eq!(play(6, &[(0, (C4, 1, 0, 1, 0x13)), (1, (0, 0, 0, 0, 0x37))], square(32))[5..12], [1332, 1332, 906, 1140, 1332, 906, 1140]);
        // And none goes above C-8.
        assert_eq!(play(6, &[(0, (C4 + 47, 1, 0, 0, 0x1C))], square(32))[..3], [113, 107, 107]);
        // A vibrato swings the period as on the other table.
        let vib = play(6, &[(0, (C4, 1, 0, 4, 0x48))], square(32));
        assert_eq!(vib[..6], [1712, 1712, 1736, 1757, 1770, 1775]);
    }

    /// An instrument's number, as openmpt123 0.8.10 renders each of
    /// these (2026-11-26): the first instrument at volume 64, the
    /// second at 32, the third at 48 and an octave up.
    #[test]
    fn an_instrument_named_is_for_the_next_note_as_openmpt_has_it() {
        let play = |cells: &[Cell5]| {
            let rows: Vec<At> = cells.iter().copied().enumerate().collect();
            let ins = [square(32), Ins { volume: 32, ..square(32) }, Ins { volume: 48, relative: 12, ..square(32) }];
            let seen = strikes(&module(1, 1, 125, &[0], &[pattern(cells.len(), &rows)], &ins), 0);
            seen.iter().map(|&(period, volume, begun)| ((PERIOD_C4 - period) / SEMITONE, volume, begun)).collect::<Vec<_>>()
        };
        let (on, at, top) = ((0, 64, true), (0, 64, false), (12, 64, true));
        // A number alone gives the volume of the instrument sounding
        // back, and the next note with none is the named one's, at
        // the volume there is.
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (0, 3, 0, 0, 0), (C4, 0, 0, 0, 0), (0, 1, 0, 0, 0), (C4, 0, 0, 0, 0)]), [on, at, top, (12, 48, false), (0, 48, true)]);
        assert_eq!(play(&[(C4, 2, 0, 0, 0), (0, 0, 0x18, 0, 0), (0, 3, 0, 0, 0), (0, 2, 0, 0, 0)])[1..], [(0, 8, false), (0, 32, false), (0, 32, false)]);
        // A volume in the column beside it is the volume.
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (0, 3, 0x20, 0, 0)])[1], (0, 16, false));
        // A note slid to keeps its instrument and does not remember
        // the number; a number with the slide and no note is
        // remembered.
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (C4 + 12, 3, 0, 3, 0xFF), (C4, 0, 0, 0, 0)]), [on, at, on]);
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (0, 0, 0x18, 0, 0), (C4, 3, 0, 5, 0), (C4, 0, 0, 0, 0)])[2..], [at, on]);
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (0, 3, 0, 3, 1), (C4, 0, 0, 0, 0)]), [on, at, top]);
        // A key-off is silence with no envelope; beside a number it
        // leaves the volume, and the number is remembered.
        assert_eq!(play(&[(C4, 1, 0, 0, 0), (KEY_OFF, 0, 0, 0, 0), (C4, 1, 0, 0, 0), (KEY_OFF, 1, 0, 0, 0), (C4, 1, 0, 0, 0), (KEY_OFF, 3, 0, 0, 0), (C4, 0, 0, 0, 0)]), [on, (0, 0, false), on, at, on, at, top]);
        // A number no instrument has: the volume back all the same,
        // and a note of it is silence.
        let none = play(&[(C4, 2, 0, 0, 0), (0, 0, 0x18, 0, 0), (0, 9, 0, 0, 0), (C4, 0, 0, 0, 0), (C4, 1, 0, 0, 0)]);
        assert_eq!((none[2], none[4]), ((0, 32, false), on));
        let silent = module(1, 1, 125, &[0], &[pattern(2, &[(0, (C4, 1, 0, 0, 0)), (1, (C4, 9, 0, 0, 0))])], &[square(32)]);
        assert_eq!(ticks(&silent).iter().map(|t| t.2).collect::<Vec<_>>(), [true, false]);
    }

    /// An instrument of several samples, as openmpt123 0.8.10 renders
    /// each of these (2026-11-26): the second instrument's first
    /// sample at volume 64 for the notes below C-4 and from C-5, its
    /// second at 16 between (an octave up here, to tell it by).
    #[test]
    fn an_instrument_of_several_samples_plays_the_notes_own() {
        let play = |cells: &[Cell5]| {
            let rows: Vec<At> = cells.iter().copied().enumerate().collect();
            // The first instrument's sample is the second's table:
            // the bytes before its sample.
            let keys = Ins { sample: (0..KEYS).map(|n| if (48..60).contains(&n) { 3 } else { 2 }).collect(), ..square(32) };
            let mut data = module(1, 1, 125, &[0], &[pattern(cells.len(), &rows)], &[keys, square(32), Ins { volume: 16, relative: 12, ..square(32) }]);
            data[HEADER + 1 + PATTERN_ENTRY + INSTRUMENT_ENTRY + 29] |= 8;
            let seen = strikes(&data, 0);
            seen.iter().map(|&(period, volume, begun)| ((PERIOD_C4 - period) / SEMITONE, volume, begun)).collect::<Vec<_>>()
        };
        // The note's own number chooses, not the relative note's.
        assert_eq!(
            play(&[(C4 - 12, 2, 0, 0, 0), (C4, 2, 0, 0, 0), (C4 + 11, 2, 0, 0, 0), (C4 - 1, 2, 0, 0, 0), (C4 + 12, 2, 0, 0, 0)]),
            [(-12, 64, true), (12, 16, true), (23, 16, true), (-1, 64, true), (12, 64, true)]
        );
        // With no number the volume stays and the sample is the
        // note's; a number alone is the volume of the one sounding.
        assert_eq!(
            play(&[(C4 - 12, 2, 0, 0, 0), (C4, 0, 0, 0, 0), (C4 - 12, 0, 0, 0, 0), (0, 2, 0, 0, 0), (C4, 2, 0, 0, 0), (0, 2, 0, 0, 0), (C4 - 12, 0, 0x30, 0, 0)]),
            [(-12, 64, true), (12, 64, true), (-12, 64, true), (-12, 64, false), (12, 16, true), (12, 16, false), (-12, 32, true)]
        );
        // A note slid to keeps the sample, and its number is that
        // sample's volume.
        assert_eq!(play(&[(C4 - 12, 2, 0, 0, 0), (C4, 2, 0, 3, 0xFF)]), [(-12, 64, true), (-12, 64, false)]);
        assert_eq!(play(&[(C4, 2, 0, 0, 0), (C4 - 12, 2, 0, 3, 0xFF)]), [(12, 16, true), (12, 16, false)]);
        // An instrument of one sample has no table, whatever is
        // before its sample.
        assert_eq!(play(&[(C4, 3, 0, 0, 0), (C4 - 12, 1, 0, 0, 0)]), [(12, 16, true), (-12, 64, true)]);
    }

    /// The first channel's volume as it is heard on each tick, in
    /// 64ths, with the envelope in it.
    fn heard(data: &[u8]) -> Vec<f64> {
        let mut t = Tune::new(data);
        let mut seen = Vec::new();
        loop {
            t.step_tick();
            if t.ended {
                return seen;
            }
            seen.push(t.channel[0].amp as f64 / 64.0);
        }
    }

    /// A row a cell, of one channel.
    fn rows(cells: &[Cell5]) -> Vec<Row> {
        cells.iter().map(|&c| vec![c]).collect()
    }

    /// The tremor, T, and the tremolo, 7, as openmpt123 0.8.10 renders
    /// these rows (2026-11-26): the level at each tick's end.
    #[test]
    fn tremor_and_tremolo_are_openmpts() {
        let play = |speed, cells: &[Cell5]| heard(&module(1, speed, 125, &[0], &[rows(cells)], &[square(32)]));
        let near = |seen: &[f64], want: &[i32]| assert!(seen.len() >= want.len() && seen.iter().zip(want).all(|(s, &w)| (s - w as f64).abs() < 0.8), "{seen:?}\n{want:?}");
        const T: u8 = TREMOR;
        // x + 1 ticks of sound and y + 1 of silence, counted on the
        // ticks but a row's first and across rows; a row with no T
        // stays as the last left it, silent too; the parameter is
        // remembered; an instrument's number ends it, and a volume
        // does not.
        let tremor = play(6, &[
            (C4, 1, 0, 0, 0), (0, 0, 0, T, 0x11), (0, 0, 0, T, 0x11), (0, 0, 0, T, 0), EMPTY, EMPTY, (0, 0, 0, T, 0x20), (0, 0, 0, T, 0x02), EMPTY,
            (0, 1, 0, 0, 0), (0, 0, 0, T, 0), (C4, 0, 0, T, 0), (C4, 1, 0, T, 0x13), (0, 0, 0x30, 0, 0),
        ]);
        #[rustfmt::skip]
        near(&tremor, &[
            64, 64, 64, 64, 64, 64,   64, 64, 64, 0, 0, 64,   64, 64, 0, 0, 64, 64,   64, 0, 0, 64, 64, 0,   0, 0, 0, 0, 0, 0,   0, 0, 0, 0, 0, 0,
            0, 0, 64, 64, 64, 0,   0, 64, 0, 0, 0, 64,   64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64,   64, 64, 0, 0, 0, 64,   64, 0, 0, 0, 64, 0,   64, 64, 64, 0, 0, 0,   0, 0, 0, 0, 0, 0,
        ]);
        // The tremolo: a sine on the volume, up first, on the first
        // tick where the row before left it; a note begins it anew,
        // the volume is the channel's again after it, a volume of
        // nought has none and full volume only its lower half.
        let tremolo = play(6, &[
            (C4, 1, 0x30, 0, 0), (0, 0, 0, 7, 0x44), (0, 0, 0, 7, 0), EMPTY, (0, 0, 0, 7, 0x80), (C4, 1, 0x30, 7, 0), (0, 0, 0x50, 7, 0x4F), (0, 0, 0x10, 7, 0), EMPTY,
        ]);
        #[rustfmt::skip]
        near(&tremolo, &[
            32, 32, 32, 32, 32, 32,   32, 32, 38, 43, 46, 48,   46, 46, 43, 38, 32, 26,   32, 32, 32, 32, 32, 32,   21, 21, 16, 21, 32, 43,
            32, 32, 43, 48, 43, 32,   22, 22, 9, 4, 9, 22,   0, 0, 0, 0, 0, 0,   0, 0, 0, 0, 0, 0,
        ]);
    }

    /// L, the envelope's position, the memory of the fine slides, X
    /// and E5, as openmpt123 0.8.10 has them (2026-11-26).
    #[test]
    fn the_envelopes_position_and_the_fine_slides_are_openmpts() {
        // An envelope that falls from 64 to nought a tick at a time.
        let falling = Ins { envelope: vec![(0, 64), (64, 0)], flags: 1, ..square(32) };
        let seen = heard(&module(1, 4, 125, &[0], &[rows(&[(C4, 1, 0, 0, 0), EMPTY, (0, 0, 0, ENVELOPE_AT, 0x20), EMPTY, (C4, 1, 0, ENVELOPE_AT, 0x30), (0, 0, 0, ENVELOPE_AT, 0x50)])], &[falling]));
        assert_eq!(seen, [64, 63, 62, 61, 60, 59, 58, 57, 32, 31, 30, 29, 28, 27, 26, 25, 16, 15, 14, 13, 0, 0, 0, 0].map(f64::from));
        let play = |cells: &[Cell5]| ticks(&module(1, 1, 125, &[0], &[rows(cells)], &[square(32)])).iter().map(|t| PERIOD_C4 - t.0).collect::<Vec<_>>();
        // E1 and E2 each remember what they slid by.
        let e = |p| (0, 0, 0, 0xE, p);
        assert_eq!(play(&[(C4, 1, 0, 0xE, 0x1F), e(0x10), e(0x10), e(0x2F), e(0x20), e(0x10)]), [60, 120, 180, 120, 60, 120]);
        assert_eq!(play(&[(C4, 1, 0, 0xE, 0x10), e(0x20)]), [0, 0]);
        // X1 and X2 are a quarter of that, with a memory of their own.
        let x = |p| (0, 0, 0, EXTRA_FINE, p);
        assert_eq!(play(&[(C4, 1, 0, EXTRA_FINE, 0x1F), x(0x10), x(0x2F), x(0x20), x(0x10), e(0x10), x(0x30)]), [15, 30, 15, 0, 15, 15, 15]);
        // E5 is the finetune of its row's note and of no other.
        let c = |fx, p| (C4, 0, 0, fx, p);
        assert_eq!(play(&[(C4, 1, 0, 0xE, 0x5F), c(0, 0), c(0xE, 0x50), c(0, 0), c(0xE, 0x5C), e(0x5F), EMPTY]), [56, 0, -64, 0, 32, 32, 32]);
    }

    #[test]
    fn the_song_follows_its_orders_jumps_and_speed() {
        let note = |n| (0, (n, 1, 0, 0, 0));
        let rows = |data: &[u8]| ticks(data).len();
        // Two patterns, played in the order 1, 0, 1.
        let a = pattern(4, &[note(C4)]);
        let b = pattern(8, &[note(C4 + 2)]);
        let data = module(1, 3, 125, &[1, 0, 1], &[a.clone(), b.clone()], &[square(32)]);
        let seen = ticks(&data);
        assert_eq!(seen.len(), (8 + 4 + 8) * 3);
        assert_eq!((seen[0].0, seen[24].0, seen[36].0), (PERIOD_C4 - 2 * SEMITONE, PERIOD_C4, PERIOD_C4 - 2 * SEMITONE));
        // D breaks to a row of the next order, in decimal.
        let brk = pattern(4, &[note(C4), (1, (0, 0, 0, 0xD, 0x05))]);
        assert_eq!(rows(&module(1, 3, 125, &[0, 1], &[brk, b.clone()], &[square(32)])), (2 + 3) * 3);
        // B jumps on; back, or to where it is, is the tune's loop and its end.
        let on = pattern(4, &[note(C4), (0, (0, 0, 0, 0xB, 2))]);
        assert_eq!(rows(&module(1, 3, 125, &[0, 1, 1], &[on, b.clone()], &[square(32)])), (1 + 8) * 3);
        let back = pattern(4, &[note(C4), (2, (0, 0, 0, 0xB, 0))]);
        assert_eq!(rows(&module(1, 3, 125, &[1, 0, 1], &[back, b.clone()], &[square(32)])), (8 + 3) * 3);
        // Played round and round, the jump back is followed and the end
        // of the list goes to the order the header names: no end.
        let rounds = |data: &[u8], ticks: usize| {
            let mut t = Tune::new(data);
            t.set_looping(true);
            (0..ticks).map(|_| { t.step_tick(); assert!(!t.ended); t.channel[0].sounding }).collect::<Vec<_>>()
        };
        let back = pattern(4, &[note(C4), (2, (0, 0, 0, 0xB, 0))]);
        let data = module(1, 3, 125, &[1, 0, 1], &[back, b.clone()], &[square(32)]);
        let seen = rounds(&data, 3 * (8 + 3) * 3);
        assert_eq!(seen[..33], seen[33..66]);
        assert_eq!((seen[0], seen[24], seen[33]), (PERIOD_C4 - 2 * SEMITONE, PERIOD_C4, PERIOD_C4 - 2 * SEMITONE));
        let mut data = module(1, 3, 125, &[1, 0], &[a.clone(), b.clone()], &[square(32)]);
        data[28] = 1;
        let seen = rounds(&data, (8 + 4 + 4) * 3 + 1);
        assert_eq!((seen[0], seen[24], seen[36], seen[47]), (PERIOD_C4 - 2 * SEMITONE, PERIOD_C4, PERIOD_C4, PERIOD_C4));
        // F sets the speed below 0x20 and the tempo from there on.
        let fast = pattern(4, &[(0, (C4, 1, 0, 0xF, 2)), (2, (0, 0, 0, 0xF, 250))]);
        let data = module(1, 6, 125, &[0], &[fast], &[square(32)]);
        assert_eq!(rows(&data), 4 * 2);
        let tick = MIX_HZ as usize * 5 / 2;
        assert_eq!(mixed(&data).len(), 4 * tick / 125 + 4 * tick / 250);
        // A note delayed by EDx starts on tick x; a pattern the file
        // lacks, or a module cut short, ends the tune and nothing more.
        let late = ticks(&module(1, 4, 125, &[0], &[pattern(2, &[(0, (C4, 1, 0, 0xE, 0xD2))])], &[square(32)]));
        assert_eq!(late.iter().map(|t| t.2).collect::<Vec<_>>()[..4], [false, false, true, true]);
        assert_eq!(rows(&module(1, 3, 125, &[0, 9], std::slice::from_ref(&a), &[square(32)])), 4 * 3);
        let whole = module(1, 3, 125, &[0], &[a], &[square(32)]);
        for cut in [HEADER, HEADER + 3, whole.len() - 40] {
            assert!(ticks(&whole[..cut]).len() <= 4 * 3);
        }
    }

    #[test]
    fn the_envelope_shapes_a_note_and_a_key_off_ends_it() {
        // Up over 4 ticks, held at point 1 while the key is down, then
        // down to nothing over 8.
        let ins = Ins { envelope: vec![(0, 0), (4, 64), (12, 0)], flags: 3, sustain: 1, fade: 4096, ..square(32) };
        let amps = |cells: &[At]| {
            let data = module(1, 4, 125, &[0], &[pattern(8, cells)], std::slice::from_ref(&ins));
            let mut t = Tune::new(&data);
            let mut seen = Vec::new();
            loop {
                t.step_tick();
                if t.ended {
                    return seen;
                }
                seen.push(t.channel[0].amp);
            }
        };
        let held = amps(&[(0, (C4, 1, 0, 0, 0))]);
        assert_eq!(held[..6], [0, 1024, 2048, 3072, 4096, 4096]);
        assert_eq!(held[31], 4096, "held at the sustain point");
        // Key off on row 2: the envelope runs on and the fade, an
        // eighth a tick here, runs with it.
        let off = amps(&[(0, (C4, 1, 0, 0, 0)), (2, (KEY_OFF, 0, 0, 0, 0))]);
        assert_eq!(off[7], 4096);
        assert!(off[8..16].windows(2).all(|w| w[1] < w[0]), "{:?}", &off[8..17]);
        assert_eq!(off[16], 0);
        // With no envelope a key-off is silence at once.
        let plain = ticks(&module(1, 4, 125, &[0], &[pattern(4, &[(0, (C4, 1, 0, 0, 0)), (1, (0, 0, 0, 0x14, 2))])], &[square(32)]));
        assert_eq!(plain.iter().map(|t| t.1).collect::<Vec<_>>()[..8], [64, 64, 64, 64, 64, 64, 0, 0]);
    }

    #[test]
    fn a_level_is_ramped_to_and_a_cut_note_dies_away() {
        // A sample that is one level throughout: the mix is the gain.
        let flat = |level: i8| Ins { sample: vec![level; 32], ..square(32) };
        let full = 127 * 256;
        // A note, then silence by the volume column, then the level
        // back, then a note of a silent instrument on top of it.
        let cells = [(0, (C4, 1, 0, 0, 0)), (1, (0, 0, 0x10, 0, 0)), (2, (0, 0, 0x50, 0, 0)), (3, (C4, 2, 0, 0, 0))];
        let out = mixed(&module(1, 4, 125, &[0], &[pattern(4, &cells)], &[flat(127), flat(0)]));
        let row = out.len() / 4;
        let ramp = (MIX_HZ / RAMP_PER_SECOND) as usize;
        // The note starts at its level: an attack is not softened.
        assert_eq!(out[0], full);
        assert!(out[..row].iter().all(|&s| s == full));
        // Down to nothing over the ramp, a step a sample, and up again.
        for (at, from, to) in [(row, full, 0), (2 * row, 0, full)] {
            let part = &out[at - 1..at + ramp + 1];
            assert_eq!((part[0], part[ramp + 1]), (from, to));
            assert!(part.windows(2).all(|w| (w[1] - w[0]).abs() <= full / ramp as i32 + 256), "{part:?}");
            assert!(part.windows(2).all(|w| (w[1] - w[0]).signum() == (to - from).signum() || w[1] == w[0]));
        }
        // The silent note takes the channel within a sample; what was
        // sounding there is not gone within one.
        let part = &out[3 * row - 1..3 * row + 16 * CUT_DECAY as usize];
        assert_eq!(part[0], full);
        assert!(part.windows(2).all(|w| w[1] <= w[0] && w[0] - w[1] <= full / CUT_DECAY + 1), "{:?}", &part[..8]);
        assert!(part[CUT_DECAY as usize] < full / 2 && part[CUT_DECAY as usize] > full / 4);
        assert_eq!(*part.last().unwrap(), 0);
    }

    #[test]
    fn the_shaping_cuts_the_bass_and_fills_the_range() {
        // 261 Hz and its odd harmonics: the fundamental goes, the top
        // stays, and the limiter brings what is left to full scale.
        let data = module(1, 6, 125, &[0], &[pattern(16, &[(0, (C4, 1, 0, 0, 0))])], &[square(32)]);
        let mut t = Tune::new(&data);
        let mut out = Vec::new();
        while let Some(s) = t.next_sample() {
            out.push(s as i32 / STEP);
        }
        assert_eq!(out.len(), 16 * 6 * (PCM_HZ as usize * 5 / 2 / 125));
        assert_eq!(crate::clip::duration_ms(&data), 0, "the converter's figure, which this test module leaves at nought");
        let settled = &out[2000..];
        let peak = settled.iter().map(|s| s.abs()).max().unwrap();
        assert!((120..=127).contains(&peak), "{peak}");
        let mean = settled.iter().sum::<i32>() as f64 / settled.len() as f64;
        assert!(mean.abs() < 1.0, "{mean}");
        // A quieter note comes out as loud: the limiter's gain.
        let quiet = module(1, 6, 125, &[0], &[pattern(16, &[(0, (C4, 1, 0x20, 0, 0))])], &[square(32)]);
        let mut t = Tune::new(&quiet);
        let mut peak = 0;
        while let Some(s) = t.next_sample() {
            peak = peak.max((s as i32 / STEP).abs());
        }
        assert!(peak >= 120, "{peak}");
        // The header may ask for its own drive: at 64 the peaks are half scale.
        let mut soft = data.clone();
        soft[10] = 64;
        soft[11] = 7;
        let mut t = Tune::new(&soft);
        let mut peak = 0;
        while let Some(s) = t.next_sample() {
            peak = peak.max((s as i32 / STEP).abs());
        }
        assert!((58..=64).contains(&peak), "{peak}");
        // The header may put the limiter's knee above the mix: then the
        // gain is a plain one, and the note at a quarter of the volume
        // comes out at a quarter.
        let peak_of = |d: &[u8]| {
            let mut t = Tune::new(d);
            let mut peak = 0;
            while let Some(s) = t.next_sample() {
                peak = peak.max((s as i32 / STEP).abs());
            }
            peak
        };
        let (mut loud, mut soft) = (data.clone(), quiet.clone());
        (loud[29], soft[29]) = (64, 64);
        let (loud, soft) = (peak_of(&loud), peak_of(&soft));
        assert!(loud < 127 && (loud - 4 * soft).abs() <= 4, "{loud} {soft}");
        // With no high-pass asked for the fundamental stays: the output
        // is the square itself, at its 261 Hz and not above.
        let crossings = |d: &[u8]| {
            let mut t = Tune::new(d);
            let mut out = Vec::new();
            while let Some(s) = t.next_sample() {
                out.push(s as i32 / STEP);
            }
            out[2000..].windows(2).filter(|w| w[0] <= 0 && w[1] > 0).count() as f64 * PCM_HZ as f64 / (out.len() - 2000) as f64
        };
        let mut plain = data.clone();
        plain[16] = 0;
        assert!((crossings(&plain) - 261.3).abs() < 3.0, "{}", crossings(&plain));
        assert!(crossings(&data) > 400.0, "{}", crossings(&data));
        assert!(is_old_tune(b"XMC1....") && !is_old_tune(&data));
        // At twice the rate: twice the samples, the same pitch and level.
        // (The section in the test module is for 8 kHz, so no high-pass.)
        let mut fast = plain.clone();
        fast[17] = 1;
        let mut t = Tune::new(&fast);
        assert_eq!((t.rate(), Tune::new(&plain).rate()), (2 * PCM_HZ, PCM_HZ));
        let mut out = Vec::new();
        while let Some(s) = t.next_sample() {
            out.push(s as i32 / STEP);
        }
        assert_eq!(out.len(), 2 * 16 * 6 * (PCM_HZ as usize * 5 / 2 / 125));
        let ups = out[4000..].windows(2).filter(|w| w[0] <= 0 && w[1] > 0).count() as f64 * 2.0 * PCM_HZ as f64 / (out.len() - 4000) as f64;
        assert!((ups - 261.3).abs() < 3.0, "{ups}");
        assert!(out[4000..].iter().map(|s| s.abs()).max().unwrap() >= 120);
        // At the fast rate: 625 samples a tick, the same pitch and level.
        fast[17] = FAST;
        let mut t = Tune::new(&fast);
        assert_eq!(t.rate(), FAST_HZ);
        let mut out = Vec::new();
        while let Some(s) = t.next_sample() {
            out.push(s as i32 / STEP);
        }
        assert_eq!(out.len(), 16 * 6 * 625);
        let ups = out[8000..].windows(2).filter(|w| w[0] <= 0 && w[1] > 0).count() as f64 * FAST_HZ as f64 / (out.len() - 8000) as f64;
        assert!((ups - 261.3).abs() < 3.0, "{ups}");
        assert!(out[8000..].iter().map(|s| s.abs()).max().unwrap() >= 120);
        // A tune of rests is silence, exactly.
        let rests = module(2, 6, 125, &[0], &[vec![vec![EMPTY, EMPTY]; 8]], &[square(32)]);
        let mut t = Tune::new(&rests);
        let mut n = 0;
        while let Some(s) = t.next_sample() {
            assert_eq!(s, 0);
            n += 1;
        }
        assert!(n > 0);
        // A clip may ask for the steps between the 8-bit ones: the same
        // samples before the rounding, and without it the whole steps.
        let mut wide = data.clone();
        wide[17] = 2;
        let (mut narrow, mut wide) = (Tune::new(&data), Tune::new(&wide));
        let (mut n, mut between) = (0, 0);
        while let (Some(a), Some(b)) = (narrow.next_sample(), wide.next_sample()) {
            assert_eq!(a as i32 % STEP, 0);
            assert_eq!(b as i32 / STEP * STEP, a as i32, "sample {n}");
            between += (b as i32 % STEP != 0) as usize;
            n += 1;
        }
        assert_eq!((narrow.next_sample(), wide.next_sample()), (None, None));
        assert!(between > n / 2, "{between} of {n}");
    }

    /// `clip` at an address that is even, or odd: a copy with room
    /// for either, and where in it the clip is.
    fn placed(clip: &[u8], odd: bool) -> (Vec<u8>, usize) {
        let mut room = vec![0u8; clip.len() + 1];
        let shift = !(room.as_ptr() as usize).is_multiple_of(2) as usize ^ odd as usize;
        room[shift..shift + clip.len()].copy_from_slice(clip);
        (room, shift)
    }

    #[test]
    fn a_sample_of_16_bits_is_played_from_an_even_address_alone() {
        // A wave with something in every low byte, a loop off its
        // first sample, a second note 256 samples in and one that
        // runs off its sample's end; and the same at 8 bits.
        let wave: Vec<i16> = (0..600).map(|i| (30000.0 * (i as f64 * 0.21).sin()) as i16 | 1).collect();
        let cells = [(0, (C4, 1, 0, 0, 0)), (3, (C4 + 7, 1, 0x30, 9, 1)), (5, (C4 + 12, 2, 0, 0, 0))];
        let clip = |wide: bool| {
            let sample = if wide { vec![] } else { wave.iter().map(|&s| (s >> 8) as i8).collect() };
            let looped = Ins { sample, wide: if wide { wave.clone() } else { vec![] }, loop_start: 100, loop_len: 500, ..square(32) };
            let once = Ins { loop_len: 0, ..looped.clone() };
            module(1, 6, 125, &[0], &[pattern(8, &cells)], &[looped, once])
        };
        let at = |clip: &[u8], odd: bool| {
            let (room, shift) = placed(clip, odd);
            let d = &room[shift..shift + clip.len()];
            assert_eq!(!(d.as_ptr() as usize).is_multiple_of(2), odd);
            mixed(d)
        };
        for wide in [true, false] {
            let clip = clip(wide);
            let even = at(&clip, false);
            assert!(even.iter().any(|&s| s != 0));
            // At an odd address a sample of 16 bits is not read at
            // all, and is silence for as long; one of 8 bits is the
            // same anywhere.
            let odd = at(&clip, true);
            assert_eq!(odd.len(), even.len());
            assert_eq!(odd == even, !wide);
            assert_eq!(odd.iter().all(|&s| s == 0), wide);
            // The converter's clip is not misplaced, by itself or at
            // an even offset of a pack; at an odd one it is, if it has
            // such a sample.
            assert!(!is_misplaced(&clip, 0) && !is_misplaced(&clip, 1000));
            assert_eq!(is_misplaced(&clip, 1001), wide);
            // A clip that ends inside its last sample plays what it
            // has of it and nothing after: the same clip with that
            // sample's end silent.
            let cut = 150 * if wide { 2 } else { 1 };
            let mut silent = clip.clone();
            let end = silent.len();
            silent[end - cut..].fill(0);
            assert!(at(&clip[..end - cut], false) == at(&silent, false) && at(&silent, false) != even);
            // Half a sample is no sample.
            if wide {
                silent[end - cut - 1] = 0;
                silent[end - cut - 2] = 0;
                assert!(at(&clip[..end - cut - 1], false) == at(&silent, false));
            }
        }
        // And a sample the clip itself has at an odd offset.
        let mut moved = clip(true);
        let entry = HEADER + 1 + PATTERN_ENTRY;
        let was = u32::from_le_bytes(moved[entry..entry + 4].try_into().unwrap());
        moved[entry..entry + 4].copy_from_slice(&(was + 1).to_le_bytes());
        assert!(is_misplaced(&moved, 0) && is_misplaced(&moved, 1) && !is_misplaced(&clip(true), 0));
    }

    #[test]
    fn a_sample_of_16_bits_is_played_as_one() {
        // The same wave both ways, the wide one its bytes times 256:
        // the same mix, to the sample, from a clip of the other magic.
        // The loop starts off the wave's first sample, and the second
        // note starts 256 samples in.
        let wave: Vec<i8> = (0..600).map(|i| (120.0 * (i as f64 * 0.21).sin()) as i8).collect();
        let narrow = Ins { sample: wave.clone(), loop_start: 100, loop_len: 500, ..square(32) };
        let wide = Ins { wide: wave.iter().map(|&s| s as i16 * 256).collect(), ..narrow.clone() };
        let cells = [(0, (C4, 1, 0, 0, 0)), (4, (C4 + 7, 1, 0x30, 9, 1))];
        let play = |ins: &Ins| module(1, 6, 125, &[0], &[pattern(8, &cells)], core::slice::from_ref(ins));
        let (a, b) = (play(&narrow), play(&wide));
        assert_eq!((&a[..4], &b[..4]), (&MAGIC[..], &WIDE_MAGIC[..]));
        assert!(is_tune(&b) && !crate::clip::is_unsupported(&b));
        // A byte more a sample, and one to put them at an even offset.
        assert_eq!(b.len(), a.len() + wave.len() + a.len() % 2);
        let (ma, mb) = (mixed(&a), mixed(&b));
        assert_eq!(ma.len(), mb.len());
        if let Some(i) = (0..ma.len()).find(|&i| ma[i] != mb[i]) {
            panic!("sample {i}: {:?} and {:?}", &ma[i.saturating_sub(3)..i + 4], &mb[i.saturating_sub(3)..i + 4]);
        }
        // And what is under the 8-bit step is played: a wave of a
        // 64th of that step's size is not silence, and peaks where it
        // should, 4 of 32768 at full level.
        let faint = Ins { wide: (0..32).map(|i| if i < 16 { 4 } else { -4 }).collect(), ..square(32) };
        let out = mixed(&module(1, 6, 125, &[0], &[pattern(8, &[(0, (C4, 1, 0, 0, 0))])], &[faint]));
        assert_eq!(out.iter().map(|s| s.abs()).max(), Some(4));
        assert!((pitch(&out) - 8363.0 / 32.0).abs() < 1.0);
    }

    #[test]
    fn a_predicted_sample_is_near_the_sample_it_was() {
        let wave = rung(1000);
        let (data, decoded) = pack(&wave, 0, false, false);
        assert_eq!(data.len(), PACKED_HEAD + wave.len().div_ceil(BLOCK) * BLOCK_BYTES);
        assert_eq!(decoded.len(), wave.len());
        let signal: f64 = wave.iter().map(|&s| (s as f64).powi(2)).sum();
        let noise: f64 = wave.iter().zip(&decoded).map(|(&a, &b)| (a as f64 - b as f64).powi(2)).sum();
        let db = 10.0 * (signal / noise).log10();
        // 8 bits a sample as they are would be under 50 dB.
        std::println!("predicted: {db:.1} dB");
        assert!(db > 60.0, "{db:.1} dB");
        // A block is what `decode_one` makes of it, a sample at a time.
        let mut before = [0i32; 3];
        for (block, want) in data[PACKED_HEAD..].chunks(BLOCK_BYTES).zip(decoded.chunks(BLOCK)) {
            for (&r, &w) in block[1..].iter().zip(want) {
                let got = adpcm::decode_one(adpcm::coefficients(&table(), block[0] >> 4), block[0] & 15, before, r as i8);
                assert_eq!(got, w as i32);
                before = [got, before[0], before[1]];
            }
        }
    }

    #[test]
    fn a_predicted_sample_is_played_as_what_it_decodes_to() {
        // The same samples both ways, predicted and as decoded at 16
        // bits: the same mix, to the sample. Each tune is a way the
        // position can move: through a loop that starts and ends off a
        // block's edge and is longer or shorter than a block, to a
        // sample's end off a block's edge, 256 samples in (9xx), back
        // to the start (E9x), a note cut by another, and a pitch that
        // steps over whole blocks.
        let wave = rung(1000);
        let high = C4 + 60;
        let tunes: [(u32, u32, &[At]); 6] = [
            (0, 0, &[(0, (C4, 1, 0, 0, 0)), (6, (C4 + 5, 1, 0x30, 0, 0))]),
            (100, 500, &[(0, (C4, 1, 0, 0, 0)), (5, (C4 + 7, 1, 0x30, 9, 1)), (9, (C4 - 12, 1, 0, 0xE, 0x92))]),
            (37, 5, &[(0, (C4, 1, 0, 0, 0)), (4, (C4 + 12, 1, 0, 0, 0))]),
            (16, 32, &[(0, (C4 + 24, 1, 0, 0, 0)), (3, (C4, 1, 0, 9, 2))]),
            (0, 1000, &[(0, (high, 1, 0, 0, 0)), (2, (high, 1, 0, 9, 3))]),
            (3, 990, &[(0, (high - 9, 1, 0, 0, 0)), (1, (C4, 1, 0, 0xE, 0x91))]),
        ];
        for (n, &(loop_start, loop_len, cells)) in tunes.iter().enumerate() {
            let len = if loop_len == 0 { 777 } else { (loop_start + loop_len) as usize };
            let (_, decoded) = pack(&wave[..len], loop_start as usize, loop_len != 0, false);
            let wide = Ins { wide: decoded, loop_start, loop_len, ..square(32) };
            let play = |ins: &Ins| module(1, 6, 125, &[0], &[pattern(12, cells)], core::slice::from_ref(ins));
            let a = play(&wide);
            let ma = mixed(&a);
            assert!(ma.iter().any(|&s| s.abs() > 1000), "tune {n} is silence");
            // And the same with seek points before the blocks, used or not.
            for seeks in [false, true] {
                let packed = Ins { wide: wave[..len].to_vec(), packed: true, seeks, ..wide.clone() };
                let b = play(&packed);
                assert_eq!((&a[..4], &b[..4]), (&WIDE_MAGIC[..], &PACKED_MAGIC[..]));
                assert!(is_tune(&b) && !crate::clip::is_unsupported(&b));
                let points = if seeks { seek_points(len) * SEEK_BYTES } else { 0 };
                // The 16 bits' clip may have a byte before them, for an even offset.
                let packed = PACKED_HEAD + points + len.div_ceil(BLOCK) * BLOCK_BYTES;
                assert!(a.len() - 2 * len - (b.len() - packed) <= 1);
                let mb = mixed(&b);
                assert_eq!(ma.len(), mb.len());
                if let Some(i) = (0..ma.len()).find(|&i| ma[i] != mb[i]) {
                    panic!("tune {n}, seek points {seeks}, sample {i}: {:?} and {:?}", &ma[i.saturating_sub(3)..i + 4], &mb[i.saturating_sub(3)..i + 4]);
                }
            }
        }
        // Two channels on the one sample do not share what they decode.
        let (_, decoded) = pack(&wave[..600], 100, true, false);
        let wide = Ins { wide: decoded, loop_start: 100, loop_len: 500, ..square(32) };
        let packed = Ins { wide: wave[..600].to_vec(), packed: true, ..wide.clone() };
        let cells = [(0, (C4, 1, 0, 0, 0)), (1, (C4 + 7, 1, 0, 0, 0))];
        let two = |ins: &Ins| {
            let rows: Vec<Row> = (0..8).map(|r| if r == 0 { vec![cells[0].1, (0, 0, 0, 0, 0)] } else if r == 1 { vec![(0, 0, 0, 0, 0), cells[1].1] } else { vec![(0, 0, 0, 0, 0); 2] }).collect();
            module(2, 6, 125, &[0], &[rows], core::slice::from_ref(ins))
        };
        assert_eq!(mixed(&two(&wide)), mixed(&two(&packed)));
        // A clip cut short in a predicted sample is played to its end
        // with nothing read past it.
        let whole = two(&packed);
        let mut t = Tune::new(&whole[..whole.len() - 300]);
        while t.next_sample().is_some() {}
    }

    /// The mix with every channel played a sample at a time, silent
    /// or not, as the mixer did before a tick moved the silent ones.
    fn mixed_eagerly(data: &[u8]) -> Vec<i32> {
        EAGER.with(|e| e.set(true));
        let out = mixed(data);
        EAGER.with(|e| e.set(false));
        out
    }

    #[test]
    fn a_silent_channel_is_where_the_mixer_would_have_left_it() {
        // A note, silence by the volume (C00) for some rows, and the
        // level back with no new note: the sample goes on from where
        // the silence left it, to the sample, whether the mixer walked
        // it there or the ticks moved it. Through a loop shorter than
        // a block and one longer than `REACH`, to a sample's end in
        // the silence, with a pitch that slides while nothing is
        // heard, and with a note cut (a new one, a retrigger) where
        // the silence is.
        let wave = rung(9000);
        let hush = (1, (0, 0, 0, 0xC, 0));
        let back = |row: usize| (row, (0, 0, 0x50, 0, 0));
        let tunes: [(u32, u32, Vec<At>); 7] = [
            (0, 8000, vec![(0, (C4, 1, 0, 0, 0)), hush, back(9)]),
            (4100, 4700, vec![(0, (C4 + 12, 1, 0, 0, 0)), hush, back(14)]),
            (37, 5, vec![(0, (C4, 1, 0, 0, 0)), hush, back(6)]),
            (0, 0, vec![(0, (C4, 1, 0, 0, 0)), hush, back(4), (5, (0, 0, 0, 0xC, 0)), back(15)]),
            (100, 7000, vec![(0, (C4, 1, 0, 0, 0)), hush, (2, (0, 0, 0, 1, 0x20)), (3, (0, 0, 0, 1, 0)), back(7)]),
            (100, 7000, vec![(0, (C4, 1, 0, 0, 0)), hush, (5, (C4 + 3, 1, 0x10, 0, 0)), (7, (0, 0, 0, 0xE, 0x93)), back(10)]),
            (0, 9000, vec![(0, (C4 + 30, 1, 0, 0, 0)), hush, back(15)]),
        ];
        for (n, (loop_start, loop_len, cells)) in tunes.iter().enumerate() {
            let len = if *loop_len == 0 { 6000 } else { (loop_start + loop_len) as usize };
            let (_, decoded) = pack(&wave[..len], *loop_start as usize, *loop_len != 0, false);
            let play = |ins: &Ins| module(1, 6, 125, &[0], &[pattern(16, cells)], core::slice::from_ref(ins));
            let wide = Ins { wide: decoded.clone(), loop_start: *loop_start, loop_len: *loop_len, ..square(32) };
            let narrow = Ins { sample: decoded.iter().map(|&s| (s >> 8) as i8).collect(), wide: vec![], ..wide.clone() };
            let packed = |seeks| Ins { wide: wave[..len].to_vec(), packed: true, seeks, ..wide.clone() };
            let reference = mixed_eagerly(&play(&wide));
            assert!(reference.iter().any(|&s| s.abs() > 1000), "tune {n} is silence");
            // Silent for a row at least, and heard after it.
            let row = reference.len() / 16;
            assert!(reference[2 * row..3 * row].iter().all(|&s| s == 0), "tune {n} has no silence");
            for (kind, ins) in [("narrow", narrow), ("wide", wide.clone()), ("packed", packed(false)), ("packed with seek points", packed(true))] {
                let clip = play(&ins);
                let want = if kind == "narrow" { mixed_eagerly(&clip) } else { reference.clone() };
                // By runs as a sample at a time, the silent ones passed
                // by or not, and at a tempo whose ticks fall anywhere.
                assert!(mixed(&clip) == mixed_one(&clip), "tune {n}, {kind}: by runs");
                EAGER.with(|e| e.set(true));
                assert!(mixed(&clip) == mixed_one(&clip), "tune {n}, {kind}: by runs, every channel played");
                EAGER.with(|e| e.set(false));
                let odd = module(1, 3, 133, &[0], &[pattern(16, cells)], core::slice::from_ref(&ins));
                assert!(mixed(&odd) == mixed_one(&odd), "tune {n}, {kind}: by runs at 133");
                FILLS.with(|f| f.set(0));
                let got = mixed(&clip);
                let fills = FILLS.with(|f| f.get());
                assert_eq!(got.len(), want.len());
                if let Some(i) = (0..got.len()).find(|&i| got[i] != want[i]) {
                    panic!("tune {n}, {kind}, sample {i} of {}: {:?} and {:?}", got.len(), &got[i.saturating_sub(3)..i + 4], &want[i.saturating_sub(3)..i + 4]);
                }
                // And no more decoded for it than when every sample is.
                if kind.starts_with("packed") {
                    FILLS.with(|f| f.set(0));
                    mixed_eagerly(&clip);
                    let eager = FILLS.with(|f| f.get());
                    assert!(fills <= eager, "tune {n}, {kind}: {fills} blocks, {eager} played through");
                }
            }
        }
    }

    /// Every module built in (`ALARM_CLIP`) mixed by runs against the
    /// mixer that went a sample at a time: the same sums.
    #[test]
    fn the_clips_built_in_are_mixed_by_runs_as_they_were_a_sample_at_a_time() {
        for (k, data) in crate::melody::CLIP_DATA.iter().enumerate().filter(|(_, d)| is_tune(d)) {
            assert!(mixed(data) == mixed_one(data), "{}", crate::melody::CLIP_NAMES[k].0);
        }
    }

    #[test]
    fn a_predicted_sample_heard_again_costs_a_few_blocks_at_most() {
        // The blocks decoded for one mixed sample, the worst there is:
        // two for a channel that plays on, `REACH` and two for one
        // heard after a silence, however long that was, and a tick's
        // own for one kept up with in its silence. A loop of 8000
        // samples, 500 blocks, silent from its second row for 28 rows.
        let wave = rung(8000);
        let cells = [(0, (C4 + 12, 1, 0, 0, 0)), (1, (0, 0, 0, 0xC, 0)), (29, (0, 0, 0x50, 0, 0))];
        for seeks in [false, true] {
            let ins = Ins { wide: wave.clone(), loop_len: 8000, packed: true, seeks, ..square(32) };
            let clip = module(1, 6, 125, &[0], &[pattern(32, &cells)], core::slice::from_ref(&ins));
            let mut t = Tune::new(&clip);
            let (mut worst, mut all, mut n) = (0, 0, 0usize);
            // The row the level comes back on, and its first sample.
            let (mut at_back, mut heard) = (0, 0);
            loop {
                FILLS.with(|f| f.set(0));
                let tick = t.pool <= 0;
                if tick {
                    t.step_tick();
                    t.wind();
                }
                if t.ended {
                    break;
                }
                t.pool -= t.tempo as i32;
                let was_quiet = t.channel[0].quiet;
                t.mix(&mut [0]);
                let fills = FILLS.with(|f| f.get());
                if tick && t.row == 30 && t.tick == 1 {
                    assert!(!was_quiet);
                    (at_back, heard) = (n, fills);
                }
                worst = worst.max(fills);
                all += fills;
                n += 1;
            }
            std::println!("a silence of 28 rows in a loop of 500 blocks, seek points {seeks}: {all} blocks, {worst} at once at worst, {heard} when it is heard again");
            assert!(at_back > 0 && heard <= REACH as usize + 2, "{heard}");
            // A tick is 1250 mixed samples, 335 of the sample's at this
            // pitch: 21 blocks.
            assert!(worst <= REACH as usize + 2 + 22, "{worst}");
            if seeks {
                // Nothing decoded in the silence.
                assert!(all < 2 * 4 * 21 * 6 + REACH as usize, "{all}");
            }
        }
    }

    /// Not a test of the player but a count, over the modules built in
    /// (`ALARM_CLIP`): the output with the silent channels passed by
    /// against the same with each played, which must be the same, the
    /// blocks decoded both ways, and the most in one render.
    #[test]
    fn the_clips_built_in_sound_the_same_with_their_silent_channels_passed_by() {
        for (k, data) in crate::melody::CLIP_DATA.iter().enumerate().filter(|(_, d)| is_tune(d)) {
            let play = |eager: bool| {
                EAGER.with(|e| e.set(eager));
                FILLS.with(|f| f.set(0));
                let mut t = Tune::new(data);
                let (mut out, mut worst, mut before) = (Vec::new(), 0, 0);
                while let Some(s) = t.next_sample() {
                    out.push(s);
                    let fills = FILLS.with(|f| f.get());
                    worst = worst.max(fills - before);
                    before = fills;
                }
                EAGER.with(|e| e.set(false));
                (out, before, worst)
            };
            let (eager, lazy) = (play(true), play(false));
            std::println!("{}: {} blocks decoded, {} with every channel played; in one render {} at most, {}", crate::melody::CLIP_NAMES[k].0, lazy.1, eager.1, lazy.2, eager.2);
            assert!(eager.0 == lazy.0, "{}", crate::melody::CLIP_NAMES[k].0);
            assert!(lazy.1 <= eager.1);
        }
    }

    #[test]
    fn a_note_far_into_a_predicted_sample_starts_from_a_seek_point() {
        // A long sample, a recording's length, and a note that starts
        // 65 280 samples into it (9FF), 4080 blocks: with seek points
        // the note's first sample costs the two blocks any start
        // does; without, every block before it. The mix is the same
        // samples' at 16 bits either way, through an offset that is on
        // no seek point's block run's edge too (9FE, then on).
        let wave: Vec<i16> = (0..70_000).map(|i| (20000.0 * ((i as f64 * 0.05).sin() + 0.4 * (i as f64 * 0.23).sin()) / 1.4) as i16).collect();
        let (_, decoded) = pack(&wave, 0, false, false);
        let wide = Ins { wide: decoded, ..square(32) };
        let wide = Ins { loop_len: 0, ..wide };
        let cells = [(0, (C4, 1, 0, 9, 0xFF)), (1, (C4 + 12, 1, 0, 9, 0xFE))];
        let play = |ins: &Ins| module(1, 6, 125, &[0], &[pattern(2, &cells)], core::slice::from_ref(ins));
        let reference = mixed(&play(&wide));
        assert!(reference.iter().any(|&s| s.abs() > 1000));
        let fills = |seeks: bool| {
            let clip = play(&Ins { wide: wave.clone(), packed: true, seeks, ..wide.clone() });
            // The first sample of the mix, and what it took.
            let mut t = Tune::new(&clip);
            FILLS.with(|n| n.set(0));
            t.next_sample();
            let first = FILLS.with(|n| n.get());
            FILLS.with(|n| n.set(0));
            assert_eq!(mixed(&clip), reference, "seek points {seeks}");
            (first, FILLS.with(|n| n.get()))
        };
        let (with, without) = (fills(true), fills(false));
        std::println!("9FF on a predicted sample: {} blocks for the first sample with seek points, {} without; the tune {} and {}", with.0, without.0, with.1, without.1);
        assert_eq!(with.0, 2);
        assert_eq!(without.0, 0xFF * SEEK / BLOCK + 2);
        // The whole tune, two notes: the blocks it plays and no more.
        assert!(with.1 < 400 && without.1 > 2 * 0xFE * SEEK / BLOCK, "{} and {}", with.1, without.1);
    }
}
