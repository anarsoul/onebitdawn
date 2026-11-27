#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
"""What openmpt123 plays of a few rows, tick by tick: the reference an
effect of the clock's module player is written to.

    scripts/openmpt-ticks.py 'row(C4, 1, 0x30); row(fx=0xA, p=0x01); row()'
    scripts/openmpt-ticks.py --speed 3 --rows 'row(C4, 1); row(vol=0x14, fx=E, p=0x60); row(vol=0x18, fx=E, p=0x61); row(vol=0x1C)'

The rows are Python. `row(note, ins, vol, fx, p, other=(fx, p), speed=n)`
is a row of a module of two channels: a cell of the first, which is
what is listened to, and in the second an effect of its own (`other`)
or a speed (`speed`, the effect F), for what must stand beside the
first's effect. `gap(n)` is n empty rows, `pattern()` ends the pattern
(they are 64 rows otherwise). `C4` is the note C-4 (49; a semitone is
1), `E`, `G`, `H`, `K`, `R` are those effects' numbers. Instrument 1 is
a ramp at volume 64 and 2 the same at 32; both run for 14 seconds at
C-4 and then are silent, so a long case begins its note again.

The module starts with two rows of its own, a C-4 of instrument 1 and
an empty one, which give the unit and are not printed: the rows given
are rows 2 and on of the first pattern, and a pattern loop with no
mark goes back into those two.

A sample that is a ramp makes a tick's slope in the render the volume
times the pitch, and shows where a note began: the figures are that, in
64ths of a C-4 at full volume, so a volume of 32 is 32, and the same
an octave up 64. `*` is a note begun on the tick, `--` a ramp at its
end. Without `--rows`, a line a row given, its ticks as many as the
speed, times one more than an EE on the row has; that is right while
no jump, break or loop takes the tune elsewhere. With `--rows`, what
is played from the top in rows of `--speed` ticks, each as its second
tick's figure, whatever row it is: for loops, breaks and delays, with
each row told by a volume in its column (`vol=0x10 + n` shows as n).

OpenMPT plays a FastTracker module by FastTracker's rules unless it
takes the file for another tracker's; this one is written under a
name it does not know, which is what it wants. `--amiga` writes
the module on the Amiga frequency table, and `--decimals 2` prints a
figure finely enough to tell one table's pitch from the other's. OpenMPT moves a volume
to its new value over the tick, global volume above all, so a slide's
figures lag: read them where the next row begins. `--keep FILE`
leaves the module there, to convert and hold the player to
(`dump_the_ticks`, `agents/collection.md`).

Needs openmpt123 and numpy.
"""
import argparse, os, struct, subprocess, sys, tempfile

import numpy as np

C4 = 49
E, G, H, K, R = 0x0E, 0x10, 0x11, 0x14, 0x1B
RAMP = 120000          # samples of an instrument: 14 s at C-4
RATE = 48000
TICK = RATE * 5 // 2 // 125


def module(spec, speed, amiga=False):
    """The module's bytes, and (speed, ticks) of each row as given."""
    patterns, rows = [], []

    def row(note=0, ins=0, vol=0, fx=0, p=0, other=None, speed=None):
        assert other is None or speed is None, 'the second channel has one effect'
        rows.append(((note, ins, vol, fx, p), (0xF, speed) if speed else other))

    def gap(n=1):
        for _ in range(n):
            row()

    def pattern():
        patterns.append(list(rows))
        rows.clear()

    row(C4, 1)
    row()
    exec(spec, {'row': row, 'gap': gap, 'pattern': pattern, 'C4': C4, 'E': E, 'G': G, 'H': H, 'K': K, 'R': R})
    gap(2)
    while rows:
        patterns.append(rows[:64])
        del rows[:64]

    def cell(c):
        return bytes(c) if any(c) else b'\x80'

    out = bytearray(b'Extended Module: ' + b'ticks'.ljust(20) + b'\x1a' + b'openmpt-ticks.py'.ljust(20) + struct.pack('<H', 0x0104))
    out += struct.pack('<IHHHHHHHH', 276, len(patterns), 0, 2, len(patterns), 2, 0 if amiga else 1, speed, 125)
    out += bytes(range(len(patterns))).ljust(256, b'\0')
    listing, now = [], speed
    for rows in patterns:
        data = b''.join(cell(c) + cell((0, 0, 0) + other if other else ()) for c, other in rows)
        out += struct.pack('<IBHH', 9, 0, len(rows), len(data)) + data
        for c, other in rows:
            if other and other[0] == 0xF and 0 < other[1] < 0x20:
                now = other[1]
            held = 0
            for fx, p in ((c[3], c[4]),) + ((other,) if other else ()):
                if fx == E and p >> 4 == 0xE:
                    held = p & 0xF
            listing.append((now * (1 + held), ' '.join('%02X' % v for v in c) + ('  ' + '%X%02X' % other if other else '')))
    deltas = b''.join(struct.pack('<h', 1 if i % 4 == 0 else 0) for i in range(RAMP))
    for volume in (64, 32):
        head = struct.pack('<I', 263) + b'ramp'.ljust(22) + b'\0' + struct.pack('<H', 1) + struct.pack('<I', 40)
        out += head.ljust(263, b'\0')
        out += struct.pack('<IIIBbBBbB', len(deltas), 0, 0, volume, 0, 0x10, 128, 0, 0) + b'ramp'.ljust(22) + deltas
    return bytes(out), listing


def render(path):
    """openmpt123's render: 48 kHz, mono, no interpolation and no ramps of its own."""
    with tempfile.TemporaryDirectory() as tmp:
        wav = os.path.join(tmp, 'ticks.wav')
        subprocess.run(['openmpt123', '--batch', '--quiet', '--force', '--samplerate', str(RATE), '--channels', '1', '--float',
                        '--filter', '1', '--ramping', '0', '-o', wav, path], check=True, stdout=subprocess.DEVNULL)
        raw = open(wav, 'rb').read()
    at = 12
    while at + 8 <= len(raw):
        tag, size = raw[at:at + 4], struct.unpack_from('<I', raw, at + 4)[0]
        if tag == b'data':
            size = min(size, len(raw) - at - 8) // 4 * 4
            return np.frombuffer(raw[at + 8:at + 8 + size], dtype='<f4').astype(np.float64)
        at += 8 + size + (size & 1)
    raise ValueError('no data in the render')


def ticks(x, decimals=0):
    """Each tick as (figure, begun), or None past the render's end."""
    unit = None
    out = []
    for pos in range(0, len(x) - TICK + 1, TICK):
        a, b = pos + TICK * 4 // 10, pos + TICK * 9 // 10
        slope = (x[b] - x[a]) / (b - a)
        if unit is None:
            if slope <= 0:
                raise ValueError('the render begins with no sound')
            unit = slope / 64
        if abs(slope) < unit * 0.2:
            out.append(('0' if abs(x[a]) < unit * 5 else '--', False))
        else:
            # The samples played before the tick began: none, for a note begun on it.
            age = x[a] / slope - (a - pos)
            # A slope the wrong way is a level on its way down within the tick.
            out.append(('?', False) if slope < 0 else ('%.*f' % (decimals, slope / unit), age < TICK * 0.5))
    return out


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('rows', help='the rows, as Python')
    p.add_argument('--speed', type=int, default=6, help="the module's speed, ticks a row (6)")
    p.add_argument('--rows', dest='stream', action='store_true', help='what is played, a figure a row of --speed ticks')
    p.add_argument('--amiga', action='store_true', help="the module on the Amiga frequency table, not the linear one")
    p.add_argument('--decimals', type=int, default=0, help='so many decimals to a figure, for a pitch (0)')
    p.add_argument('--keep', help='write the module here as well')
    args = p.parse_args()
    data, listing = module(args.rows, args.speed, args.amiga)
    with tempfile.TemporaryDirectory() as tmp:
        path = args.keep or os.path.join(tmp, 'ticks.xm')
        open(path, 'wb').write(data)
        seen = ticks(render(path), args.decimals)
    text = lambda t: t[0] + ('*' if t[1] else '')
    if args.stream:
        rows = [seen[at:at + args.speed] for at in range(2 * args.speed, len(seen), args.speed)]
        print(' '.join(row[min(1, len(row) - 1)][0] + ('*' if any(t[1] for t in row) else '') for row in rows))
        return
    at = 0
    for n, (count, cells) in enumerate(listing):
        row = seen[at:at + count]
        at += count
        if n >= 2 and n < len(listing) - 2:
            print('%-22s %s' % (cells, ' '.join(text(t) for t in row) if row else '(not played)'))


if __name__ == '__main__':
    sys.exit(main())
