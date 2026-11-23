// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! The encoders for `onebitdawn_core::adpcm`'s decoder.
//!
//! IMA ADPCM, four bits a sample, half the flash of 8-bit PCM: the
//! encoder keeps the `search` best paths through the coder's states
//! (1 is the plain encoder, a sample's nearest code), which is a
//! decibel less error for the same decoder.
//!
//! Predicted sound, 16 bits in a byte a sample and a sixteenth more
//! (`predicted`): a table of predictors fitted to the sound, then for
//! each block the predictor and the shift that leave the least error.
//! A module's 16-bit samples are kept the same way (`xmc.rs`).

use onebitdawn_core::adpcm::{BLOCK, BLOCK_BYTES, COEFFICIENT_MAX, INDEX, MAGIC, PACKED_MAGIC, SHIFT_MAX, STEPS, TABLE_BYTES, TRAINED, WIDE_MAGIC, coefficients, decode_one};

/// Paths the encoder keeps.
pub const SEARCH: usize = 4;

/// The decoder's step: the sample a code gives and the next index.
fn step(code: u8, predicted: i32, index: i32) -> (i32, i32) {
    let size = STEPS[index as usize] as i32;
    let mut diff = size >> 3;
    if code & 1 != 0 {
        diff += size >> 2;
    }
    if code & 2 != 0 {
        diff += size >> 1;
    }
    if code & 4 != 0 {
        diff += size;
    }
    let predicted = if code & 8 != 0 { predicted - diff } else { predicted + diff }.clamp(-32768, 32767);
    (predicted, (index + INDEX[(code & 7) as usize] as i32).clamp(0, 88))
}

/// The samples the codes give.
pub fn decode(codes: &[u8]) -> Vec<i16> {
    let (mut predicted, mut index) = (0, 0);
    codes
        .iter()
        .map(|&code| {
            (predicted, index) = step(code, predicted, index);
            predicted as i16
        })
        .collect()
}

/// The code the plain encoder gives a sample.
fn nearest(sample: i32, predicted: i32, index: i32) -> u8 {
    let size = STEPS[index as usize] as i32;
    let mut diff = sample - predicted;
    let mut code = if diff < 0 { 8 } else { 0 };
    diff = diff.abs();
    for (bit, part) in [(4, size), (2, size >> 1), (1, size >> 2)] {
        if diff >= part {
            code |= bit;
            diff -= part;
        }
    }
    code
}

/// The codes for `samples`: of the paths through the coder's states,
/// the `search` with the least squared error so far are kept at each
/// sample, and the best at the end is the clip.
pub fn encode(samples: &[i16], search: usize) -> Vec<u8> {
    if search <= 1 {
        let (mut predicted, mut index) = (0, 0);
        return samples
            .iter()
            .map(|&sample| {
                let code = nearest(sample as i32, predicted, index);
                (predicted, index) = step(code, predicted, index);
                code
            })
            .collect();
    }
    // A path: its error, the state it is in, and its last code in
    // `trail`, where each code names the one before it.
    const START: u32 = u32::MAX;
    let mut trail: Vec<(u8, u32)> = Vec::with_capacity(samples.len() * search);
    let mut paths: Vec<(u64, (i32, i32), u32)> = vec![(0, (0, 0), START)];
    let mut reached: Vec<(u64, (i32, i32), u8, u32)> = Vec::new();
    for &sample in samples {
        reached.clear();
        for &(error, (predicted, index), before) in &paths {
            for code in 0..16 {
                let state = step(code, predicted, index);
                let off = (sample as i64 - state.0 as i64).unsigned_abs();
                let total = error + off * off;
                // A state keeps the place it was first reached in,
                // which is what breaks a tie between two paths.
                match reached.iter_mut().find(|path| path.1 == state) {
                    Some(path) => {
                        if total < path.0 {
                            *path = (total, state, code, before);
                        }
                    }
                    None => reached.push((total, state, code, before)),
                }
            }
        }
        reached.sort_by_key(|path| path.0);
        paths.clear();
        for &(total, state, code, before) in reached.iter().take(search) {
            trail.push((code, before));
            paths.push((total, state, (trail.len() - 1) as u32));
        }
    }
    let mut codes = Vec::with_capacity(samples.len());
    let mut at = paths[0].2;
    while at != START {
        let (code, before) = trail[at as usize];
        codes.push(code);
        at = before;
    }
    codes.reverse();
    codes
}

/// An ADPCM clip of these samples at `rate` a second.
pub fn clip(samples: &[i16], rate: u32, search: usize) -> Vec<u8> {
    let mut samples = samples.to_vec();
    // Whole bytes: an odd clip ends on its last sample again.
    if samples.len() % 2 == 1 {
        samples.push(samples[samples.len() - 1]);
    }
    let codes = encode(&samples, search);
    let mut out = MAGIC.to_vec();
    out.extend(rate.to_le_bytes());
    out.extend(codes.as_chunks::<2>().0.iter().map(|pair| pair[0] << 4 | pair[1]));
    out
}

/// Predicted sound: the table of its predictors, its blocks, and the
/// samples the decoder makes of them.
pub struct Predicted {
    pub table: [u8; TABLE_BYTES],
    pub blocks: Vec<u8>,
    pub decoded: Vec<i16>,
}

/// What a block says of how well a predictor would do on it, without
/// its samples: for the three samples before each of its own, their
/// products with each other (`rr`), with the sample (`r`), and the
/// samples' own squares.
#[derive(Clone, Copy, Default)]
struct Sums {
    rr: [[f64; 3]; 3],
    r: [f64; 3],
    yy: f64,
}

impl Sums {
    fn add(&mut self, other: &Sums) {
        for i in 0..3 {
            for j in 0..3 {
                self.rr[i][j] += other.rr[i][j];
            }
            self.r[i] += other.r[i];
        }
        self.yy += other.yy;
    }

    /// The squares of what predictor `a` leaves of the block.
    fn left(&self, a: &[f64; 3]) -> f64 {
        let mut e = self.yy;
        for i in 0..3 {
            e -= 2.0 * a[i] * self.r[i];
            for j in 0..3 {
                e += a[i] * self.rr[i][j] * a[j];
            }
        }
        e
    }

    /// The predictor that leaves the least, with `ridge` on the
    /// diagonal so that silence and a pure tone have an answer.
    fn fit(&self, ridge: f64) -> [f64; 3] {
        let mut m = [[0f64; 4]; 3];
        for i in 0..3 {
            m[i][..3].copy_from_slice(&self.rr[i]);
            m[i][i] += ridge;
            m[i][3] = self.r[i];
        }
        // Gauss, the largest pivot first.
        for col in 0..3 {
            let pivot = (col..3).max_by(|&a, &b| m[a][col].abs().total_cmp(&m[b][col].abs())).unwrap_or(col);
            m.swap(col, pivot);
            if m[col][col] == 0.0 {
                return [0.0; 3];
            }
            for row in 0..3 {
                if row != col {
                    let f = m[row][col] / m[col][col];
                    for k in col..4 {
                        m[row][k] -= f * m[col][k];
                    }
                }
            }
        }
        [m[0][3] / m[0][0], m[1][3] / m[1][1], m[2][3] / m[2][2]]
    }
}

/// The predictors a sound needs no fitting for, in 64ths: the last
/// sample, the line through the last two, the parabola through the
/// last three (FLAC's fixed ones), and three of the SNES's and
/// CD-XA's. Where the fitting starts from.
const KNOWN: [[i32; 3]; 6] = [[64, 0, 0], [128, -64, 0], [192, -192, 64], [60, 0, 0], [122, -60, 0], [115, -52, 0]];
/// Times the predictors are fitted to the blocks that chose them.
const ROUNDS: usize = 25;

/// A table of `TRAINED` predictors for these samples: each block goes
/// to the predictor that leaves the least of it, each predictor is
/// fitted again to the blocks it got, and so round and round
/// (Lloyd's). "No prediction" is one a block may go to and is never
/// fitted: it is not in the table, the decoder has it as predictor 0
/// whatever the table holds. Nothing here is random, and the
/// arithmetic is what every machine does alike, so a sound has one
/// table.
fn train(samples: &[i16]) -> [u8; TABLE_BYTES] {
    let at = |i: usize, back: usize| if i >= back { samples[i - back] as f64 } else { 0.0 };
    let blocks: Vec<Sums> = (0..samples.len() / BLOCK)
        .map(|b| {
            let mut sums = Sums::default();
            for (i, &sample) in samples.iter().enumerate().skip(b * BLOCK).take(BLOCK) {
                let (y, h) = (sample as f64, [at(i, 1), at(i, 2), at(i, 3)]);
                for j in 0..3 {
                    for k in 0..3 {
                        sums.rr[j][k] += h[j] * h[k];
                    }
                    sums.r[j] += h[j] * y;
                }
                sums.yy += y * y;
            }
            sums
        })
        .collect();
    let trace: f64 = blocks.iter().map(|s| s.rr[0][0] + s.rr[1][1] + s.rr[2][2]).sum();
    let ridge = 1e-3 * trace / blocks.len().max(1) as f64;
    // From the known ones, and blocks' own at even steps through the
    // sound.
    let mut table = [[0f64; 3]; TRAINED];
    for (n, entry) in table.iter_mut().enumerate() {
        *entry = match KNOWN.get(n) {
            Some(k) => k.map(|c| c as f64 / 64.0),
            None if blocks.is_empty() => [0.0; 3],
            None => blocks[(n - KNOWN.len() + 1) * blocks.len() / (TRAINED - KNOWN.len() + 1)].fit(ridge),
        };
    }
    for _ in 0..ROUNDS {
        let mut got = [Sums::default(); TRAINED];
        let mut any = [false; TRAINED];
        for sums in &blocks {
            let (mut best, mut least) = (None, sums.yy);
            for (n, a) in table.iter().enumerate() {
                let left = sums.left(a);
                if left < least {
                    (best, least) = (Some(n), left);
                }
            }
            if let Some(n) = best {
                got[n].add(sums);
                any[n] = true;
            }
        }
        for n in 0..TRAINED {
            if any[n] {
                table[n] = got[n].fit(ridge);
            }
        }
    }
    let mut out = [0u8; TABLE_BYTES];
    for (n, a) in table.iter().enumerate() {
        for (i, c) in a.iter().enumerate() {
            let c = (c * 64.0).round().clamp(-COEFFICIENT_MAX as f64, COEFFICIENT_MAX as f64) as i16;
            out[6 * n + 2 * i..][..2].copy_from_slice(&c.to_le_bytes());
        }
    }
    out
}

/// A block as one predictor and one shift make it: the squares of
/// what it is off by, its bytes, the samples it decodes to, and the
/// last three of them, the newest first.
struct Tried {
    error: u64,
    block: [u8; BLOCK_BYTES],
    got: [i16; BLOCK],
    last: [i32; 3],
}

/// These 16-bit samples predicted: a table fitted to them (`train`),
/// then a block for every `BLOCK` of them with the predictor and the
/// shift that leave the least error, every pair tried, predictor 0
/// among them, with the decoder's own arithmetic on the samples as it
/// will decode them: a sample's error is its own residual's rounding,
/// and nothing adds up. A last block is filled out with residuals of
/// nothing.
pub fn predicted(samples: &[i16]) -> Predicted {
    let table = train(samples);
    let (mut blocks, mut decoded, mut before) = (Vec::with_capacity(samples.len().div_ceil(BLOCK) * BLOCK_BYTES), Vec::with_capacity(samples.len()), [0i32; 3]);
    for chunk in samples.chunks(BLOCK) {
        let mut best: Option<Tried> = None;
        for predictor in 0..=TRAINED as u8 {
            let k = coefficients(&table, predictor);
            for shift in 0..=SHIFT_MAX {
                let (mut block, mut got, mut last, mut error) = ([0u8; BLOCK_BYTES], [0i16; BLOCK], before, 0u64);
                block[0] = predictor << 4 | shift;
                for (i, &want) in chunk.iter().enumerate() {
                    let nothing = decode_one(k, shift, last, 0);
                    let residual = ((want as i32 - nothing + (1 << shift >> 1)) >> shift).clamp(-128, 127) as i8;
                    let sample = decode_one(k, shift, last, residual);
                    block[1 + i] = residual as u8;
                    got[i] = sample as i16;
                    error += (sample - want as i32).unsigned_abs().pow(2) as u64;
                    last = [sample, last[0], last[1]];
                }
                if best.as_ref().is_none_or(|b| error < b.error) {
                    best = Some(Tried { error, block, got, last });
                }
            }
        }
        let Tried { block, got, last, .. } = best.expect("a block has a predictor");
        decoded.extend(&got[..chunk.len()]);
        // What the decoder has after a whole block; after a last one
        // that is filled out nothing follows.
        before = last;
        blocks.extend(block);
    }
    Predicted { table, blocks, decoded }
}

/// A predicted clip of these samples at `rate` a second.
pub fn packed_clip(samples: &[i16], rate: u32) -> Vec<u8> {
    let sound = predicted(samples);
    let mut out = PACKED_MAGIC.to_vec();
    out.extend(rate.to_le_bytes());
    out.extend((samples.len() as u32).to_le_bytes());
    out.extend(sound.table);
    out.extend(sound.blocks);
    out
}

/// A clip of these samples as they are, 16 bits each.
pub fn wide_clip(samples: &[i16], rate: u32) -> Vec<u8> {
    let mut out = WIDE_MAGIC.to_vec();
    out.extend(rate.to_le_bytes());
    out.extend(samples.iter().flat_map(|s| s.to_le_bytes()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    /// A random walk under a square wave, loud and quiet by turns.
    fn signal() -> Vec<i16> {
        let (mut x, mut level) = (12345i64, 0i64);
        (0..30001i64)
            .map(|k| {
                x = (x * 1103515245 + 12345) & 0x7FFF_FFFF;
                level = (level + (x >> 16) % 4001 - 2000).clamp(-30000, 30000);
                (level + if k / 37 % 2 == 1 { 8000 } else { -8000 } * (k / 5000 % 3)).clamp(-32768, 32767) as i16
            })
            .collect()
    }

    /// The codes are those of `scripts/adpcm.py`, which the encoder was
    /// first written as: the digests are of its output for this signal
    /// (2026-11-19).
    #[test]
    fn the_encoder_is_the_one_the_clips_were_made_with() {
        for (search, digest) in [
            (1, "d11e626de6b75b3ff011c3ac4bc07d14bfc72605f6f08b3ba6e52bf90322c2b3"),
            (4, "17dd7df9daa4dd80006b1e9fed31bdacea7df1bbe0f3be401a781dab90b144ca"),
        ] {
            let made = clip(&signal(), 16000, search);
            assert_eq!(&made[..8], b"ADP1\x80\x3e\0\0");
            assert_eq!(made.len(), 8 + 15001);
            let hex: String = Sha256::digest(&made[8..]).iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(hex, digest, "search {search}");
        }
    }

    #[test]
    fn the_search_is_nearer_than_the_plain_encoder_and_the_clock_decodes_it() {
        let samples = signal();
        let error = |search| -> f64 {
            let codes = encode(&samples, search);
            decode(&codes).iter().zip(&samples).map(|(&a, &b)| (a as f64 - b as f64).powi(2)).sum()
        };
        assert!(error(4) < error(1));
        // The clock's decoder gives what this one does.
        let made = clip(&samples, 16000, 4);
        let mut theirs = onebitdawn_core::adpcm::Decoder::new(&made);
        let codes: Vec<u8> = made[8..].iter().flat_map(|b| [b >> 4, b & 15]).collect();
        for ours in decode(&codes) {
            assert_eq!(theirs.next_sample(), Some(ours));
        }
    }
}
