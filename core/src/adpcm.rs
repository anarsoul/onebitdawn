// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! A sampled clip that is not a byte a sample: IMA ADPCM at four bits,
//! 16 bits predicted in eight, and 16 bits as they are.
//!
//! Sampled sound has nothing a reference could name (a bar played
//! again is not the same samples), so the way to store it smaller is
//! a lossy one. `Decoder` gives each kind's samples, 16 bits each, in
//! `xm::STEP`ths of PCM's 8-bit step as a tune's are.
//!
//! **`MAGIC`, IMA ADPCM.** Each sample is a 4-bit code, the step from
//! the last sample in units that grow after a large code and shrink
//! after a small one. Half the flash of 8-bit PCM at the same rate,
//! and heard as much nearer to it than PCM at half the rate
//! (HISTORY.md, 2026-11-16). The clip is the magic, the rate as a
//! `u32`, then the codes, two a byte, the high nibble first: sox's raw
//! `ima`, so the body plays on the host. The coder starts from a
//! prediction of 0 and step index 0. `tools/pack/src/adpcm.rs` is the
//! encoder.
//!
//! **`PACKED_MAGIC`, predicted.** A sample of 16 bits is kept as what
//! is left of it after the three before it have predicted it: the
//! SNES's and CD-XA's scheme with a residual of 8 bits, in blocks of
//! `BLOCK` samples, each a byte naming a predictor and a shift and
//! then the residuals (`decode_block`). A byte a sample and a
//! sixteenth more, and for a piano some 25 dB nearer the 16 bits
//! than 8 bits are (PLAN.md, 2026-11-22). The predictors are a table
//! of `TRAINED` the encoder fits to the sound and writes once
//! (`TABLE_BYTES`), and one more that is not in the table and cannot
//! be left out of it: predictor 0 predicts nothing, so its sample is
//! its residual shifted, which reaches all of 16 bits whatever came
//! before. A trained predictor can be further off at full scale than
//! a residual reaches; a table of those alone was 9 dB worse than
//! none on a loud recording. The clip is the magic, the rate, the
//! number of samples as a `u32`, the table, the blocks. A module's
//! predicted samples are the same blocks behind a table of their own
//! (`xm.rs`).
//!
//! **`WIDE_MAGIC`, 16 bits.** The magic, the rate, the samples,
//! little-endian: twice 8-bit PCM's flash, and nothing lost.

use crate::clip::{FAST_HZ, PCM_HZ};

/// The first bytes of an ADPCM clip.
pub const MAGIC: &[u8; 4] = b"ADP1";
/// Of a predicted one.
pub const PACKED_MAGIC: &[u8; 4] = b"ADP2";
/// Of one of 16 bits as they are.
pub const WIDE_MAGIC: &[u8; 4] = b"PCM2";
/// The magic and the rate.
const HEADER: usize = 8;
/// A predicted clip's: those, the number of samples, the table.
pub const PACKED_HEADER: usize = HEADER + 4 + TABLE_BYTES;

/// Samples in a block of predicted sound, and its bytes: the
/// predictor in the high nibble and the shift in the low, then a
/// residual a sample.
pub const BLOCK: usize = 16;
pub const BLOCK_BYTES: usize = 1 + BLOCK;
/// The predictors a table holds, numbered from 1: predictor 0 is no
/// prediction and is not in it.
pub const TRAINED: usize = 15;
/// A table: for each predictor three coefficients as `i16`, in 64ths,
/// of the last sample, the one before and the one before that.
pub const TABLE_BYTES: usize = TRAINED * 6;
/// The largest coefficient there is, in 64ths: one past it is held to
/// it, so that three of them times a sample fit 32 bits.
pub const COEFFICIENT_MAX: i32 = 1023;
/// The largest shift the encoder uses: a residual of 8 bits then
/// reaches all of 16.
pub const SHIFT_MAX: u8 = 8;

/// The step sizes, by index.
pub const STEPS: [i16; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190,
    209, 230, 253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499,
    2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385,
    24623, 27086, 29794, 32767,
];
/// What a code's magnitude adds to the step index.
pub const INDEX: [i8; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// How a clip's samples are kept.
#[derive(Clone, Copy, PartialEq)]
enum Coding {
    Ima,
    Packed,
    Wide,
}

const fn coding(data: &[u8]) -> Option<Coding> {
    if data.len() < HEADER || data[2] != b'P' && data[2] != b'M' {
        return None;
    }
    let magic = [data[0], data[1], data[2], data[3]];
    match &magic {
        MAGIC => Some(Coding::Ima),
        PACKED_MAGIC if data.len() >= PACKED_HEADER => Some(Coding::Packed),
        WIDE_MAGIC => Some(Coding::Wide),
        _ => None,
    }
}

/// Whether `data` is a clip of one of the three kinds.
pub const fn is_clip(data: &[u8]) -> bool {
    coding(data).is_some()
}

/// Samples a second of the clip `data`: what `PCM1` may say, and
/// `PCM_HZ` for anything else.
pub const fn rate(data: &[u8]) -> u32 {
    if !is_clip(data) {
        return PCM_HZ;
    }
    match u32::from_le_bytes([data[4], data[5], data[6], data[7]]) {
        rate if rate == 2 * PCM_HZ || rate == FAST_HZ => rate,
        _ => PCM_HZ,
    }
}

/// Samples in the clip `data`: two a byte after an ADPCM clip's
/// header, what a predicted clip says of as many as its blocks hold,
/// one for two bytes at 16 bits.
pub const fn samples(data: &[u8]) -> usize {
    match coding(data) {
        Some(Coding::Ima) => 2 * (data.len() - HEADER),
        Some(Coding::Packed) => {
            let said = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            let held = (data.len() - PACKED_HEADER) / BLOCK_BYTES * BLOCK;
            if said < held { said } else { held }
        }
        Some(Coding::Wide) => (data.len() - HEADER) / 2,
        None => 0,
    }
}

/// How long the clip plays, in milliseconds.
pub const fn duration_ms(data: &[u8]) -> u32 {
    (samples(data) as u64 * 1000 / rate(data) as u64) as u32
}

/// A predictor's coefficients in 64ths, of the last sample, the one
/// before and the one before that: nothing for predictor 0, whatever
/// the table says, and the table's for 1 to `TRAINED`.
#[inline(always)]
pub fn coefficients(table: &[u8], predictor: u8) -> [i32; 3] {
    let Some(at) = (predictor as usize).checked_sub(1).map(|n| n % TRAINED * 6) else {
        return [0; 3];
    };
    let k = |n: usize| match table.get(at + 2 * n..at + 2 * n + 2) {
        Some(b) => (i16::from_le_bytes([b[0], b[1]]) as i32).clamp(-COEFFICIENT_MAX, COEFFICIENT_MAX),
        None => 0,
    };
    [k(0), k(1), k(2)]
}

/// A predicted sample from the three before it, the newest first, and
/// its residual: held to 16 bits, which the encoder counts on.
#[inline(always)]
pub fn decode_one(k: [i32; 3], shift: u8, before: [i32; 3], residual: i8) -> i32 {
    let prediction = (k[0] * before[0] + k[1] * before[1] + k[2] * before[2]) >> 6;
    (prediction + ((residual as i32) << (shift & 15))).clamp(-32768, 32767)
}

/// The samples of a block: `block` is its `BLOCK_BYTES`, `table` the
/// table its predictor is of, `before` the three samples before it,
/// the newest first, and then the block's last three.
#[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"), inline(never))]
pub fn decode_block(table: &[u8], before: &mut [i32; 3], block: &[u8; BLOCK_BYTES], out: &mut [i16; BLOCK]) {
    let (k, shift) = (coefficients(table, block[0] >> 4), block[0] & 15);
    let mut p = *before;
    for (o, &r) in out.iter_mut().zip(&block[1..]) {
        let s = decode_one(k, shift, p, r as i8);
        *o = s as i16;
        p = [s, p[0], p[1]];
    }
    *before = p;
}

/// The clip's samples, one at a time.
pub struct Decoder<'a> {
    coding: Coding,
    /// The codes, the blocks or the samples, without the header.
    codes: &'a [u8],
    /// A predicted clip's table.
    table: &'a [u8],
    /// Samples in the clip.
    len: usize,
    /// The next sample; for ADPCM, the next code, counted in nibbles.
    at: usize,
    /// ADPCM: the sample last given, and the step's place in `STEPS`.
    predicted: i32,
    index: i32,
    /// Predicted: the block `at` is in, as decoded, and the three
    /// samples before the next block, the newest first.
    block: [i16; BLOCK],
    before: [i32; 3],
    /// Whether the clip starts over at its end.
    looping: bool,
}

impl<'a> Decoder<'a> {
    /// A decoder of the clip `data`; of nothing, if it is not one.
    pub const fn new(data: &'a [u8]) -> Self {
        let (kind, codes, table): (Coding, &[u8], &[u8]) = match coding(data) {
            Some(Coding::Ima) => (Coding::Ima, data.split_at(HEADER).1, &[]),
            Some(Coding::Packed) => {
                let (head, blocks) = data.split_at(PACKED_HEADER);
                (Coding::Packed, blocks, head.split_at(HEADER + 4).1)
            }
            Some(Coding::Wide) => (Coding::Wide, data.split_at(HEADER).1, &[]),
            None => (Coding::Ima, &[], &[]),
        };
        Self { coding: kind, codes, table, len: samples(data), at: 0, predicted: 0, index: 0, block: [0; BLOCK], before: [0; 3], looping: false }
    }

    /// Samples in the clip.
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Go round at the end, or stop there. The coder's state starts
    /// over with the clip, as the encoder's did.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// The next sample, `None` after the last of a clip played once.
    pub fn next_sample(&mut self) -> Option<i16> {
        if self.at >= self.len() {
            if !self.looping || self.is_empty() {
                return None;
            }
            (self.at, self.predicted, self.index, self.before) = (0, 0, 0, [0; 3]);
        }
        match self.coding {
            Coding::Ima => Some(self.next_code()),
            Coding::Packed => {
                let within = self.at % BLOCK;
                if within == 0 {
                    // `len` is no more than the blocks hold.
                    let block = self.codes[self.at / BLOCK * BLOCK_BYTES..].first_chunk::<BLOCK_BYTES>()?;
                    decode_block(self.table, &mut self.before, block, &mut self.block);
                }
                self.at += 1;
                Some(self.block[within])
            }
            Coding::Wide => {
                let at = 2 * self.at;
                self.at += 1;
                Some(i16::from_le_bytes([self.codes[at], self.codes[at + 1]]))
            }
        }
    }

    /// The next sample of an ADPCM clip.
    fn next_code(&mut self) -> i16 {
        let byte = self.codes[self.at / 2];
        let code = if self.at & 1 == 0 { byte >> 4 } else { byte & 15 };
        self.at += 1;
        let step = STEPS[self.index as usize] as i32;
        // (magnitude + 1/2) * step / 4, each term rounded down.
        let mut diff = step >> 3;
        if code & 1 != 0 {
            diff += step >> 2;
        }
        if code & 2 != 0 {
            diff += step >> 1;
        }
        if code & 4 != 0 {
            diff += step;
        }
        if code & 8 != 0 {
            diff = -diff;
        }
        self.predicted = (self.predicted + diff).clamp(i16::MIN as i32, i16::MAX as i32);
        self.index = (self.index + INDEX[(code & 7) as usize] as i32).clamp(0, STEPS.len() as i32 - 1);
        self.predicted as i16
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A table of the predictors a sound needs no fitting for: the
    /// last sample, the line through the last two, the parabola
    /// through the last three, and three of the SNES's; the rest are
    /// nothing.
    pub(crate) fn table() -> [u8; TABLE_BYTES] {
        let mut table = [0u8; TABLE_BYTES];
        for (n, k) in [[64i16, 0, 0], [128, -64, 0], [192, -192, 64], [60, 0, 0], [122, -60, 0], [115, -52, 0]].iter().enumerate() {
            for (i, c) in k.iter().enumerate() {
                table[6 * n + 2 * i..][..2].copy_from_slice(&c.to_le_bytes());
            }
        }
        table
    }

    /// Samples as blocks, with every predictor of `table` and every
    /// shift tried for each, and what they decode to.
    pub(crate) fn blocks(table: &[u8], wave: &[i16]) -> (Vec<u8>, Vec<i16>) {
        let (mut blocks, mut decoded, mut before) = (Vec::new(), Vec::<i16>::new(), [0i32; 3]);
        for chunk in wave.chunks(BLOCK) {
            let mut best: Option<(i64, [u8; BLOCK_BYTES])> = None;
            for predictor in 0..=TRAINED as u8 {
                let k = coefficients(table, predictor);
                for shift in 0..=SHIFT_MAX {
                    let (mut block, mut p, mut error) = ([0u8; BLOCK_BYTES], before, 0i64);
                    block[0] = predictor << 4 | shift;
                    for (i, &want) in chunk.iter().enumerate() {
                        let nothing = decode_one(k, shift, p, 0);
                        let r = ((want as i32 - nothing + (1 << shift >> 1)) >> shift).clamp(-128, 127) as i8;
                        let got = decode_one(k, shift, p, r);
                        block[1 + i] = r as u8;
                        error += ((got - want as i32) as i64).pow(2);
                        p = [got, p[0], p[1]];
                    }
                    if best.is_none_or(|b| error < b.0) {
                        best = Some((error, block));
                    }
                }
            }
            let block = best.unwrap().1;
            let mut out = [0i16; BLOCK];
            decode_block(table, &mut before, &block, &mut out);
            decoded.extend(&out[..chunk.len()]);
            blocks.extend(block);
        }
        (blocks, decoded)
    }

    /// A wave a predictor has something to say of, with partials and a
    /// level that falls, near full scale at its start.
    pub(crate) fn rung(n: usize) -> Vec<i16> {
        (0..n)
            .map(|i| {
                let t = i as f64;
                let fall = (-t / 400.0).exp();
                (30000.0 * fall * (0.6 * (t * 0.11).sin() + 0.3 * (t * 0.31).sin() + 0.1 * (t * 0.8).sin())) as i16
            })
            .collect()
    }

    fn packed(rate: u32, said: u32, table: &[u8], blocks: &[u8]) -> Vec<u8> {
        let mut data = PACKED_MAGIC.to_vec();
        data.extend(rate.to_le_bytes());
        data.extend(said.to_le_bytes());
        data.extend(table);
        data.extend(blocks);
        data
    }

    #[test]
    fn a_predicted_clip_is_its_blocks_decoded() {
        let wave = rung(1000);
        let (blocks, decoded) = blocks(&table(), &wave);
        assert_eq!(blocks.len(), 63 * BLOCK_BYTES);
        let data = packed(16_000, 1000, &table(), &blocks);
        assert_eq!(data.len(), PACKED_HEADER + blocks.len());
        assert!(is_clip(&data) && !crate::clip::is_unsupported(&data));
        assert_eq!((rate(&data), samples(&data), duration_ms(&data)), (16_000, 1000, 62));
        let mut decoder = Decoder::new(&data);
        assert_eq!(decoder.len(), 1000);
        let got: Vec<i16> = core::iter::from_fn(|| decoder.next_sample()).collect();
        assert_eq!(got, decoded);
        assert_eq!(decoder.next_sample(), None, "and it stays ended");
        // Near the samples it was: 8 bits a sample as they are would
        // be under 50 dB.
        let signal: f64 = wave.iter().map(|&s| (s as f64).powi(2)).sum();
        let noise: f64 = wave.iter().zip(&got).map(|(&a, &b)| (a as f64 - b as f64).powi(2)).sum();
        let db = 10.0 * (signal / noise).log10();
        assert!(db > 60.0, "{db:.1} dB");
        // Round and round, the coder starts over with the clip.
        let mut decoder = Decoder::new(&data);
        decoder.set_looping(true);
        let twice: Vec<i16> = (0..2000).map(|_| decoder.next_sample().unwrap()).collect();
        assert_eq!((&twice[..1000], &twice[1000..]), (&decoded[..], &decoded[..]));
        // A clip that says more samples than its blocks hold has what
        // they hold; one cut short of its table is no clip, and no PCM
        // either.
        let short = packed(16_000, 5000, &table(), &blocks[..10 * BLOCK_BYTES]);
        assert_eq!(samples(&short), 160);
        let mut decoder = Decoder::new(&short);
        assert_eq!(core::iter::from_fn(|| decoder.next_sample()).count(), 160);
        assert!(!is_clip(&data[..PACKED_HEADER - 1]) && crate::clip::is_unsupported(&data[..PACKED_HEADER - 1]));
        let empty = packed(16_000, 0, &table(), &[]);
        assert!(is_clip(&empty) && Decoder::new(&empty).next_sample().is_none());
    }

    #[test]
    fn predictor_0_predicts_nothing_whatever_the_table_holds() {
        // A table of the largest coefficients there are, and past
        // them: predictor 0 is still a sample's residual, shifted, and
        // reaches both ends of 16 bits from anywhere.
        let mut table = [0u8; TABLE_BYTES];
        for pair in table.as_chunks_mut::<2>().0 {
            *pair = i16::MAX.to_le_bytes();
        }
        assert_eq!(coefficients(&table, 0), [0; 3]);
        assert_eq!(coefficients(&table, 1), [COEFFICIENT_MAX; 3]);
        assert_eq!(coefficients(&[], 7), [0; 3]);
        let mut block = [0u8; BLOCK_BYTES];
        block[0] = SHIFT_MAX;
        for (i, r) in block[1..].iter_mut().enumerate() {
            *r = if i % 2 == 0 { 127 } else { -128i8 as u8 };
        }
        let (mut before, mut out) = ([32767, -32768, 32767], [0i16; BLOCK]);
        decode_block(&table, &mut before, &block, &mut out);
        assert!(out.iter().enumerate().all(|(i, &s)| s == if i % 2 == 0 { 127 << 8 } else { -32768 }), "{out:?}");
        // Any other predictor of that table is held to 16 bits and
        // does not overflow on the way.
        block[0] = 0x10 | SHIFT_MAX;
        decode_block(&table, &mut before, &block, &mut out);
        assert!(out.iter().all(|&s| s == 32767 || s == -32768), "{out:?}");
        // The encoder's best for a wave that slams between the rails
        // is predictor 0 at the largest shift, a step off at most.
        let slam: Vec<i16> = (0..64).map(|i| if i / 3 % 2 == 0 { 32767 } else { -32768 }).collect();
        let (coded, decoded) = blocks(&table, &slam);
        assert!(coded.chunks(BLOCK_BYTES).all(|b| b[0] >> 4 == 0), "{:?}", coded.chunks(BLOCK_BYTES).map(|b| b[0]).collect::<Vec<_>>());
        assert!(slam.iter().zip(&decoded).all(|(&a, &b)| (a as i32 - b as i32).abs() <= 255));
    }

    #[test]
    fn a_clip_of_16_bits_is_its_samples() {
        let wave = rung(300);
        let mut data = WIDE_MAGIC.to_vec();
        data.extend(31_250u32.to_le_bytes());
        data.extend(wave.iter().flat_map(|s| s.to_le_bytes()));
        // A last odd byte is no sample.
        data.push(0x55);
        assert!(is_clip(&data) && !crate::clip::is_unsupported(&data));
        assert_eq!((rate(&data), samples(&data)), (31_250, 300));
        let mut decoder = Decoder::new(&data);
        assert_eq!(core::iter::from_fn(|| decoder.next_sample()).collect::<Vec<_>>(), wave);
        let mut decoder = Decoder::new(&data);
        decoder.set_looping(true);
        assert_eq!((0..600).map(|_| decoder.next_sample().unwrap()).collect::<Vec<_>>(), [&wave[..], &wave[..]].concat());
        // The clip's player takes both kinds for what they are.
        for clip in [&data[..], &packed(16_000, 300, &table(), &blocks(&table(), &wave).0)] {
            let mut stream = crate::clip::Stream::new(clip, false);
            let mut buf = [0u8; crate::clip::CHUNK];
            let mut chunks = 0;
            while !stream.done() && chunks < 10_000 {
                stream.fill(&mut buf);
                chunks += 1;
            }
            assert!(stream.done() && chunks > 1, "{chunks} chunks");
        }
    }

    fn clip(rate: u32, codes: &[u8]) -> Vec<u8> {
        let mut data = MAGIC.to_vec();
        data.extend_from_slice(&rate.to_le_bytes());
        data.extend_from_slice(codes);
        data
    }

    /// The plain encoder: each sample's nearest code, the decoder's own
    /// state carried along.
    fn encode(samples: &[i16]) -> Vec<u8> {
        let (mut predicted, mut index) = (0i32, 0i32);
        let mut nibbles = Vec::new();
        for &s in samples {
            let step = STEPS[index as usize] as i32;
            let mut diff = s as i32 - predicted;
            let mut code = 0u8;
            if diff < 0 {
                code = 8;
                diff = -diff;
            }
            let mut moved = step >> 3;
            for (bit, part) in [(4, step), (2, step >> 1), (1, step >> 2)] {
                if diff >= part {
                    code |= bit;
                    diff -= part;
                    moved += part;
                }
            }
            predicted = (predicted + if code & 8 != 0 { -moved } else { moved }).clamp(i16::MIN as i32, i16::MAX as i32);
            index = (index + INDEX[(code & 7) as usize] as i32).clamp(0, 88);
            nibbles.push(code);
        }
        nibbles.chunks(2).map(|pair| pair[0] << 4 | pair.get(1).copied().unwrap_or(0)).collect()
    }

    /// Codes that run the step up and the prediction into both rails,
    /// and what sox made of them (`sox -t ima -r 16000 -c 1`).
    #[test]
    fn the_samples_are_what_sox_decodes() {
        const CODES: [u8; 32] = [
            0xE6, 0x07, 0x79, 0x7E, 0xC7, 0x95, 0x77, 0x00, 0x75, 0x79, 0x07, 0xC7, 0x5C, 0x7C, 0xC0, 0x75, 0x79, 0x6F, 0x06, 0x97, 0xCF, 0x96, 0x7C,
            0xC5, 0xE7, 0x97, 0xC7, 0xC5, 0x39, 0x0E, 0x3C, 0x37,
        ];
        const SAMPLES: [i16; 64] = [
            -10, 10, 12, 50, 133, 97, 262, -46, -425, 340, 12, 1106, 3291, 7975, 8644, 9252, 17554, 30606, 32767, 21595, 24980, 32767, -4095, 32767,
            32767, -4095, 32767, -4095, -32768, -28673, 27190, 32767, 32767, 20481, 32767, -28669, -24574, 23841, 11555, 32767, -4095, -32768, -32768,
            15647, 32767, -4095, -32768, 12285, -32768, 28668, 16382, 32767, -4095, 32767, -4095, 32767, 32767, 21595, 24980, -15031, 13638, -19880,
            8789, 32767,
        ];
        let data = clip(16_000, &CODES);
        let mut decoder = Decoder::new(&data);
        let got: Vec<i16> = core::iter::from_fn(|| decoder.next_sample()).collect();
        assert_eq!(got, SAMPLES);
        assert_eq!(decoder.next_sample(), None, "and it stays ended");
    }

    #[test]
    fn a_sine_comes_back() {
        let sine: Vec<i16> = (0..8000).map(|i| (12_000.0 * (i as f64 * 0.2).sin()) as i16).collect();
        let data = clip(16_000, &encode(&sine));
        let mut decoder = Decoder::new(&data);
        let got: Vec<i16> = core::iter::from_fn(|| decoder.next_sample()).collect();
        assert_eq!(got.len(), sine.len());
        // Past the first samples, where the step is still growing.
        let signal: f64 = sine[200..].iter().map(|&s| (s as f64).powi(2)).sum();
        let error: f64 = sine[200..].iter().zip(&got[200..]).map(|(&a, &b)| (a as f64 - b as f64).powi(2)).sum();
        let snr = 10.0 * (signal / error).log10();
        assert!(snr > 30.0, "a 500 Hz sine at 16 kHz within 30 dB, got {snr:.1}");
    }

    #[test]
    fn a_looping_clip_starts_over_as_it_started() {
        let data = clip(16_000, &[0x17, 0x7A, 0xF3]);
        let mut once = Decoder::new(&data);
        let pass: Vec<i16> = core::iter::from_fn(|| once.next_sample()).collect();
        assert_eq!(pass.len(), 6);
        let mut round = Decoder::new(&data);
        round.set_looping(true);
        for k in 0..18 {
            assert_eq!(round.next_sample(), Some(pass[k % 6]), "sample {k}");
        }
        // An empty clip goes nowhere, looped or not.
        let empty = clip(16_000, &[]);
        let mut decoder = Decoder::new(&empty);
        decoder.set_looping(true);
        assert_eq!(decoder.next_sample(), None);
    }

    #[test]
    fn the_header_says_the_rate_and_the_length() {
        let data = clip(16_000, &[0; 8000]);
        assert!(is_clip(&data));
        assert_eq!((rate(&data), samples(&data), duration_ms(&data)), (16_000, 16_000, 1000));
        assert_eq!(rate(&clip(FAST_HZ, &[])), FAST_HZ);
        assert_eq!(rate(&clip(8_000, &[])), PCM_HZ);
        assert_eq!(rate(&clip(44_100, &[])), PCM_HZ, "a rate the line cannot run at");
        // Not a clip: too short for the header, or another magic.
        assert!(!is_clip(b"ADP1\x80\x3e\x00") && !is_clip(b"PCM1\x80\x3e\x00\x00"));
        assert_eq!((samples(b"ADP1"), Decoder::new(b"PCM1\x80\x3e\x00\x00\x77").next_sample()), (0, None));
    }
}
