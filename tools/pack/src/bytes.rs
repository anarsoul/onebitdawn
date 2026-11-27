// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! Reading a file that may be cut short or lie about itself: a field
//! past the end is an error, a run of bytes past the end is as much of
//! it as there is. `Padded` is the other reading, for where a player
//! of the format goes on: a field past the end is nought.

pub const SHORT: &str = "the file is cut short";

#[derive(Clone, Copy)]
pub struct Data<'a>(pub &'a [u8]);

impl<'a> Data<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The bytes from `from` to `to`, or what there is of them.
    pub fn slice(&self, from: usize, to: usize) -> &'a [u8] {
        let to = to.min(self.0.len());
        &self.0[from.min(to)..to]
    }

    /// `len` bytes at `at`, all of them.
    pub fn bytes(&self, at: usize, len: usize) -> Result<&'a [u8], String> {
        at.checked_add(len).and_then(|end| self.0.get(at..end)).ok_or_else(|| SHORT.to_string())
    }

    pub fn u8(&self, at: usize) -> Result<u8, String> {
        self.0.get(at).copied().ok_or_else(|| SHORT.to_string())
    }

    pub fn i8(&self, at: usize) -> Result<i8, String> {
        Ok(self.u8(at)? as i8)
    }

    pub fn u16(&self, at: usize) -> Result<u16, String> {
        let b = self.bytes(at, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u16be(&self, at: usize) -> Result<u16, String> {
        let b = self.bytes(at, 2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn u32(&self, at: usize) -> Result<u32, String> {
        let b = self.bytes(at, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// A `u32` as an offset or a length.
    pub fn size(&self, at: usize) -> Result<usize, String> {
        Ok(self.u32(at)? as usize)
    }
}

/// A file read as if noughts followed it for ever.
#[derive(Clone, Copy)]
pub struct Padded<'a>(pub &'a [u8]);

impl<'a> Padded<'a> {
    /// A place in the file, the end for any past it: what is read
    /// there is the same, and sums of such do not overflow.
    pub fn at(&self, at: usize) -> usize {
        at.min(self.0.len())
    }

    /// The bytes from `from` to `to`, or what there is of them.
    pub fn slice(&self, from: usize, to: usize) -> &'a [u8] {
        Data(self.0).slice(from, to)
    }

    fn field<const N: usize>(&self, at: usize) -> [u8; N] {
        let mut field = [0; N];
        let have = self.slice(at, at.saturating_add(N));
        field[..have.len()].copy_from_slice(have);
        field
    }

    pub fn u8(&self, at: usize) -> u8 {
        self.field::<1>(at)[0]
    }

    pub fn i8(&self, at: usize) -> i8 {
        self.u8(at) as i8
    }

    pub fn u16(&self, at: usize) -> u16 {
        u16::from_le_bytes(self.field(at))
    }

    /// A `u32` as an offset or a length.
    pub fn size(&self, at: usize) -> usize {
        u32::from_le_bytes(self.field(at)) as usize
    }
}

/// A text field: up to its first NUL, as Latin-1.
pub fn text(field: &[u8]) -> String {
    field.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()
}
