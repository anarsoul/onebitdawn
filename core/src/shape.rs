// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! What a player on the chip does to its mix before the modulator, and
//! `tools/pack/src/wav.rs` does to a PCM clip on the host, since the
//! buzzer is the same: the bass cut by a high-pass and the level pushed
//! hard into a limiter. How much bass and how hard are the clip's to
//! say, in its header; a tune whose melody sits under 900 Hz wants a
//! lower cut than the PCM clips' 900 Hz twice over.
//!
//! The tracker modules (`xm`) and the SID tunes (`sid`) both end here.

/// The limiter: a level under this gets the full gain, one over it is
/// brought down to it. One channel at full volume peaks at 32768
/// before the filter, so this is 20 dB of drive, as the host's
/// conversion gives a PCM clip.
const LIMIT: i32 = 3000;
/// The limiter lets go by a 128th a sample at `clip::PCM_HZ`, 16 ms.
const RELEASE_SHIFT: u32 = 7;
/// What the limiter aims a peak at, of a full scale of 127: just that.
/// With these two the test modules come out at the RMS the host's
/// conversion of them has, 0.42 of full scale. A release of 4 ms and
/// peaks aimed a quarter over full scale, the excess clipped, are 3 dB
/// more (0.60, a sixth of the samples at the rails) and were the
/// default while the tunes ran at 8 kHz; at 16 kHz the user's ear
/// took the cleaner one (2026-11-12). A clip may still ask for its own.
const DRIVE: i32 = 127;

/// What one 8-bit step is in the samples the shaper gives.
pub const STEP: i32 = 256;

/// A second-order section, transposed direct form II.
#[derive(Clone, Copy, Default)]
struct Biquad {
    z1: i64,
    z2: i64,
}

impl Biquad {
    /// `k` is b0, b1, b2, a1, a2 in Q14.
    ///
    /// Each product is of two `i32` into an `i64`, two multiplies on
    /// the chip where two `i64` are four: the coefficients are
    /// `i16`s, and the output is an `i32` unless the filter has run
    /// away, which `wild` is for.
    #[inline(always)]
    fn run(&mut self, k: &[i32; 5], x: i32) -> i32 {
        let wide = |a: i32, b: i32| a as i64 * b as i64;
        let whole = (wide(k[0], x) + self.z1) >> 14;
        let y = whole as i32;
        if y as i64 != whole {
            return self.wild(k, x, whole);
        }
        self.z1 = wide(k[1], x) - wide(k[3], y) + self.z2;
        self.z2 = wide(k[2], x) - wide(k[4], y);
        y
    }

    /// The same with an output no `i32` holds, in full, as every
    /// sample was.
    #[cold]
    #[inline(never)]
    fn wild(&mut self, k: &[i32; 5], x: i32, y: i64) -> i32 {
        self.z1 = k[1] as i64 * x as i64 - k[3] as i64 * y + self.z2;
        self.z2 = k[2] as i64 * x as i64 - k[4] as i64 * y;
        y as i32
    }
}

/// The high-pass and the limiter of one tune being played.
pub struct Shaper {
    /// The high-pass: its section, how many of them, and their states.
    highpass: [i32; 5],
    passes: usize,
    filter: [Biquad; 2],
    envelope: i32,
    drive: i32,
    release: u32,
    /// The limiter's knee: a level under it gets the full gain.
    knee: i32,
    /// Whether the samples keep what the limiter's division leaves
    /// under the 8-bit step.
    wide: bool,
    /// The gain `inverse` was prepared for (0 for none yet) and
    /// 2^32 over it, rounded down: what `divide` multiplies by.
    divisor: u32,
    inverse: u32,
}

/// A gain under this is divided by with a multiply (`Shaper::scale`):
/// the level times the drive is then under 2^31, and so is a
/// remainder times `STEP`. Twenty channels at full level are under
/// 2^20 before the filter.
const SMALL: i32 = 1 << 23;
const _: () = assert!(SMALL as i64 * u8::MAX as i64 <= 1 << 31 && SMALL as i64 * STEP as i64 <= 1 << 31);

/// `a / g` and what is left, for `m` 2^32 over `g` rounded down
/// (`g` 2 at least, so it fits).
///
/// 2^32 / g - 1 < m <= 2^32 / g, so a * m / 2^32 is a / g at most and
/// more than a / g - a / 2^32, which is more than a / g - 1 for any
/// `a` a `u32` holds: rounded down it is the quotient or one under
/// it, and the remainder says which.
#[inline(always)]
fn divide(a: u32, g: u32, m: u32) -> (u32, u32) {
    let q = ((a as u64 * m as u64) >> 32) as u32;
    let r = a - q * g;
    if r >= g { (q + 1, r - g) } else { (q, r) }
}

impl Shaper {
    /// As a clip's header gives them: the limiter's `drive` and
    /// `release` (0 for the defaults; the release is given for
    /// `clip::PCM_HZ`, and `octaves` says how many times the samples'
    /// rate is twice that, or about, where a shift more each is the
    /// same time), the high-pass's section in
    /// Q14 (b0 b1 b2 a1 a2, worked out by the converter since the chip
    /// has no tangent) and how many times it is applied (0 to 2), and
    /// whether the samples are to be finer than the 8-bit step; and
    /// the limiter's `knee` in 1024s of the mix's units (0 for `LIMIT`):
    /// 32 is one voice at full level, and a knee above a tune's own
    /// peak is the limiter off, a plain gain, for a speaker, where the
    /// buzzer's 20 dB of drive is heard as overdrive.
    pub fn new(drive: u8, release: u8, octaves: u32, section: [i16; 5], passes: u8, wide: bool, knee: u8) -> Self {
        Self {
            highpass: section.map(|k| k as i32),
            passes: (passes as usize).min(2),
            filter: [Biquad::default(); 2],
            envelope: 0,
            drive: match drive {
                0 => DRIVE,
                d => d as i32,
            },
            release: match release {
                0 => RELEASE_SHIFT,
                r => (r as u32).min(14),
            } + octaves,
            knee: match knee {
                0 => LIMIT,
                k => k as i32 * 1024,
            },
            wide,
            divisor: 0,
            inverse: 0,
        }
    }

    /// One sample of the mix, 32768 for one voice at full level, as the
    /// modulator's: signed, in `STEP`ths of the 8-bit step a PCM clip
    /// has, so full scale is 127 of those. Whole steps unless `wide`.
    #[cfg_attr(all(feature = "ram", target_os = "none"), unsafe(link_section = ".rwtext"), inline(never))]
    pub fn run(&mut self, x: i32) -> i16 {
        let mut y = x;
        for filter in &mut self.filter[..self.passes] {
            y = filter.run(&self.highpass, y);
        }
        self.envelope = (self.envelope - (self.envelope >> self.release)).max(y.abs());
        let gain = self.envelope.max(self.knee);
        self.scale(y, gain)
    }

    /// `y * drive / gain` in `STEP`ths, cut to full scale: with what
    /// is under a step if `wide`, whole steps if not. `gain` is `|y|`
    /// at least, the envelope being that.
    ///
    /// With no division while the gain is what it was, which under
    /// the knee is always: 2^32 over it is kept and multiplied by
    /// (`divide`), twice, once for the steps and once for what the
    /// first leaves. A limiter at work moves the gain every sample
    /// and pays one division of 32 bits for it, where there was one
    /// of 64 from the ROM every sample. The value is `slow`'s, both
    /// truncating toward nought.
    #[inline(always)]
    fn scale(&mut self, y: i32, gain: i32) -> i16 {
        if !(2..SMALL).contains(&gain) {
            return self.slow(y, gain);
        }
        let g = gain as u32;
        if g != self.divisor {
            // 2^32 / g, as 2^32 - 1 over it and one more if that left g - 1.
            let q = u32::MAX / g;
            self.inverse = if u32::MAX - q * g == g - 1 { q + 1 } else { q };
            self.divisor = g;
        }
        let (steps, left) = divide(y.unsigned_abs() * self.drive as u32, g, self.inverse);
        let level = if self.wide { (steps * STEP as u32 + divide(left * STEP as u32, g, self.inverse).0).min(127 * STEP as u32) } else { steps.min(127) * STEP as u32 };
        if y < 0 { -(level as i32) as i16 } else { level as i16 }
    }

    /// The same by division, as it was for every sample: for a gain
    /// `scale` does not take, which no tune's mix comes to.
    #[cold]
    #[inline(never)]
    fn slow(&self, y: i32, gain: i32) -> i16 {
        if self.wide {
            (y as i64 * (self.drive * STEP) as i64 / gain as i64).clamp(-127 * STEP as i64, 127 * STEP as i64) as i16
        } else {
            ((y * self.drive / gain).clamp(-127, 127) * STEP) as i16
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn the_scaling_by_a_multiply_is_the_division_it_was() {
        // Every drive's ends and the default, wide and not, gains from
        // the least there is to both sides of `SMALL`, and for each
        // the levels at both ends, round nought and through the
        // middle; a gain that is `|y|` at least, as the envelope's is.
        let gains = [2, 3, 7, 1023, 1024, 2999, 3000, 3001, 12 * 1024, 65535, 65536, 65537, 255 * 1024, 1_000_003, SMALL - 1, SMALL, SMALL + 1, 1 << 24, 100_000_000];
        let mut seed = 0x2545_F491u32;
        let mut tried = 0u32;
        for drive in [1u8, 2, 12, 127, 128, 254, 255] {
            for wide in [false, true] {
                let mut fast = Shaper::new(drive, 0, 0, [0; 5], 0, wide, 0);
                let slow = Shaper::new(drive, 0, 0, [0; 5], 0, wide, 0);
                for &gain in &gains {
                    let mut levels = std::vec![0, 1, 2, 3, gain / 2, gain / 2 + 1, gain - 2, gain - 1, gain];
                    for _ in 0..4000 {
                        seed ^= seed << 13;
                        seed ^= seed >> 17;
                        seed ^= seed << 5;
                        levels.push((seed % (gain as u32 + 1)) as i32);
                    }
                    // What the division's own `i32` holds, where it is not `wide`.
                    let fits = |y: &i32| y.abs() <= gain && (wide || y.abs() as i64 * (drive as i64) < 1 << 31);
                    for y in levels.into_iter().flat_map(|y| [y, -y]).filter(fits) {
                        assert_eq!(fast.scale(y, gain), slow.slow(y, gain), "{y} by {gain}, drive {drive}, wide {wide}");
                        tried += 1;
                    }
                    // Another gain between this one and the next.
                    assert_eq!(fast.scale(-1, gains[0]), slow.slow(-1, gains[0]));
                }
            }
        }
        assert!(tried > 2_000_000);
        // And every level there is under a small gain, each sign.
        for gain in [2, 3, 1024, 3000] {
            for wide in [false, true] {
                let mut fast = Shaper::new(0, 0, 0, [0; 5], 0, wide, 0);
                for y in -gain..=gain {
                    assert_eq!(fast.scale(y, gain), fast.slow(y, gain), "{y} by {gain}");
                }
            }
        }
    }

    #[test]
    fn the_shaper_runs_as_it_did_with_the_limiter_at_work_and_at_rest() {
        // A level that swells over the knee and falls under it, so the
        // gain is the knee's, then the envelope's, a new one each
        // sample, then the knee's again: `run` against the same
        // envelope with the division.
        for (wide, knee) in [(true, 0u8), (false, 0), (true, 64), (true, 1)] {
            let mut shaper = Shaper::new(0, 0, 2, [0; 5], 0, wide, knee);
            let mut envelope = 0i32;
            let mut over = 0;
            for i in 0..60_000i32 {
                let swell = if (20_000..30_000).contains(&i) { 90_000 } else { 900 };
                let y = (swell as f64 * (i as f64 * 0.37).sin()) as i32;
                envelope = (envelope - (envelope >> shaper.release)).max(y.abs());
                let gain = envelope.max(shaper.knee);
                over += (gain > shaper.knee) as u32;
                assert_eq!(shaper.run(y), shaper.slow(y, gain), "sample {i}");
            }
            assert!(over > 5000 && over < 55_000, "{over}");
        }
    }
}
