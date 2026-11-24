// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Vasily Khoruzhick

//! How deep the main stack has been, for the bench: the `stack-watch`
//! feature. What the statics and the code in RAM leave of DRAM is the
//! main stack (`agents/sizes.md`), and nothing says how much of it is
//! used: the tasks' futures are statics, so the depth is that of the
//! deepest poll, with the interrupts that came on top of it, the
//! interrupt executor's tasks among them.
//!
//! `paint` fills the part not in use with a mark, and `Report` looks
//! for the lowest word that is no longer the mark: the deepest the
//! stack has been since it was painted. `main` paints as it starts,
//! logs the boot's depth once the tasks are spawned and paints again,
//! so the heartbeat's figure is the running firmware's, as far as
//! what ran has shown: it wants the clock doing everything it does
//! at once, and for a while.
//!
//! Only the main stack. The Wi-Fi driver's tasks have stacks of their
//! own on the heap, which esp-rtos checks; an interrupt that comes
//! while one of them runs is on that stack and not seen here.

use core::arch::asm;
use core::fmt;

/// What an unused word holds.
const MARK: u32 = 0xA5A5_A5A5;
/// Left alone at the stack's end: esp-hal keeps its guard word there
/// (`stack-guard-offset`, 60 bytes from the end).
const FLOOR: usize = 128;
/// Left alone under the stack pointer when painting.
const MARGIN: usize = 256;

unsafe extern "C" {
    // The main stack's two ends, from esp-hal's linker script: the
    // symbols' addresses are the ends, lowest and highest.
    static _stack_end: u8;
    static _stack_start: u8;
}

/// The stack's lowest address that is painted, and its top.
fn ends() -> (usize, usize) {
    ((&raw const _stack_end) as usize + FLOOR, (&raw const _stack_start) as usize)
}

/// Mark the main stack below where it is in use. From the main task.
pub fn paint() {
    let (lowest, top) = ends();
    let sp: usize;
    // SAFETY: reads a register.
    unsafe { asm!("mv {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags)) };
    // Not on the main stack: nothing to paint, and `Report` says so.
    if sp <= lowest + MARGIN || sp > top {
        return;
    }
    let mut at = lowest;
    while at < (sp - MARGIN) & !3 {
        // SAFETY: `at` is a word of the main stack, by the linker's
        // symbols, at least `MARGIN` bytes under the stack pointer of
        // the thread whose stack it is, so no frame of this thread is
        // there, and none of a function this one calls: it calls
        // none. An interrupt that comes now builds its frames under
        // the stack pointer and above `at` or over it, and they are
        // dead again before this loop goes on, one core running both.
        // No other thread has this stack. The word is aligned, `at`
        // starting on a word (the linker aligns the stack to 4) and
        // going by 4.
        unsafe { (at as *mut u32).write_volatile(MARK) };
        at += 4;
    }
}

/// Bytes of the main stack, and how many of them have been in use at
/// the deepest since `paint`; `None` if the lowest word painted is
/// gone, which is a stack that was never painted or one that has been
/// all the way down.
fn deepest() -> (usize, Option<usize>) {
    let (lowest, top) = ends();
    let size = top - (lowest - FLOOR);
    let mut at = lowest;
    // SAFETY: a volatile read of an aligned word of the main stack
    // (see `paint`), below anything this function has a reference to;
    // whatever an interrupt leaves there is a `u32`.
    while at < top && unsafe { (at as *const u32).read_volatile() } == MARK {
        at += 4;
    }
    (size, (at != lowest).then_some(top - at))
}

/// The main stack for the log.
pub struct Report;

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match deepest() {
            (size, Some(used)) => write!(f, "{size} bytes, {used} used at the deepest, {} never", size - used),
            (size, None) => write!(f, "{size} bytes, not painted or used to the end"),
        }
    }
}
