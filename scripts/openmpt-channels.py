#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
"""Hold the clock's module player to openmpt123 tick by tick, on the
channels of real modules that use an effect.

    scripts/openmpt-channels.py --effect R ~/Music/modules/a.xm ~/Music/modules/b.xm
    scripts/xm-effects.py ~/Music/modules --effect R --paths > /tmp/r.txt
    scripts/openmpt-channels.py --effect R --list /tmp/r.txt [--show 5]

Run from the repository's root. For each channel of each module that
has the effect (as FastTracker names it: A, E6, R; or `any` for every
channel with a note), the module is written again with that channel
alone, every instrument a ramp at its own volume with no envelope,
and the tempo held at 125. Of the other channels only what moves the
tune or its level is kept: jumps, breaks, speeds, pattern loops and
delays, the global volume. openmpt123 renders that module, the pack
maker converts it and the player's ticks are dumped
(`dump_the_ticks`, `core/src/xm.rs`); a ramp makes a tick's slope in
the render the volume times the pitch and shows where a note began,
and the two are compared on every tick:

    a.xm ch 3: 412 ticks of R: the level differs on 0, a note begins in one alone on 0; other ticks that differ: 2

`--show N` prints a channel's first N ticks of the effect that differ.

Differences that are the measure's and not the player's, each seen:
a note so low that the ramp's steps are a large part of a tick's
slope; a volume slide, which OpenMPT ramps across the tick; an effect
the player lacks elsewhere in the channel (the pack maker names it),
after which the rows no longer line up; a ramp at its end, which is
passed over. Look at a channel that differs before believing it.

The module is written under a tracker's name OpenMPT does not know:
FastTracker's own in the header made it take the rebuilt file for an
old ModPlug Tracker's and play some effects by other rules.

Needs openmpt123, numpy and the repository's cargo aliases.
"""
import argparse, os, struct, subprocess, sys, tempfile

import numpy as np

RAMP = 120000
RATE = 48000
TICK = RATE * 5 // 2 // 125
NAMES = {chr(ord('G') + n): 16 + n for n in range(20)}


def effect_of(name):
    """(effect number, the E effect's high nibble or None) of a name; None for `any`."""
    if name == 'any':
        return None
    name = name.upper()
    if len(name) == 2 and name[0] == 'E':
        return 0xE, int(name[1], 16)
    return (NAMES[name] if name in NAMES else int(name, 16)), None


def has(cell, effect):
    if effect is None:
        return cell[0] != 0
    return cell[3] == effect[0] and (effect[1] is None or cell[4] >> 4 == effect[1]) and (cell[3] != 0 or cell[4] != 0)


def parse(d):
    """The header's size, the channels, the patterns' cells, each instrument's volume."""
    u16 = lambda at: struct.unpack_from('<H', d, at)[0] if at + 2 <= len(d) else 0
    u32 = lambda at: struct.unpack_from('<I', d, at)[0] if at + 4 <= len(d) else 0
    hsize, channels, patterns, instruments = u32(60), u16(68), u16(70), u16(72)
    at, pats = 60 + hsize, []
    for _ in range(patterns):
        length, rows, size = u32(at), u16(at + 5), u16(at + 7)
        q, end, cells = at + length, min(at + length + size, len(d)), []
        while q < end:
            b = d[q]
            q += 1
            c = [0] * 5
            if b & 0x80:
                for i in range(5):
                    if b & (1 << i) and q < end:
                        c[i] = d[q]
                        q += 1
            else:
                c[0] = b
                c[1:5] = d[q:q + 4].ljust(4, b'\0')
                q += 4
            cells.append(c)
        if size == 0:
            cells = [[0] * 5 for _ in range(rows * channels)]
        pats.append((rows, cells))
        at += length + size
    volumes = []
    for _ in range(instruments):
        # As the pack maker reads them: nought past the file's end, a
        # sample's header 40 bytes whatever the instrument says.
        size, samples = u32(at) or 263, u16(at + 27)
        if at >= len(d) or samples == 0:
            volumes.append(0 if at < len(d) else 64)
            at += size
            continue
        head = at + size
        volumes.append(d[head + 12] if head + 12 < len(d) else 64)
        at = head + 40 * samples + sum(u32(head + 40 * k) for k in range(samples))
    return hsize, channels, pats, volumes


def rebuilt(d, hsize, channels, pats, volumes, keep):
    """The module with the channel `keep` alone and ramps for instruments."""
    out = bytearray(d[:60 + hsize])
    out[38:58] = b'openmpt-channels.py'.ljust(20)
    struct.pack_into('<H', out, 74, struct.unpack_from('<H', d, 74)[0] | 1)     # the linear table
    struct.pack_into('<H', out, 78, 125)
    for rows, cells in pats:
        body = bytearray()
        for i, c in enumerate(cells):
            if i % channels != keep:
                moves = c[3] in (0xB, 0xD, 0x10, 0x11) or (c[3] == 0xF and 0 < c[4] < 0x20) or (c[3] == 0xE and c[4] >> 4 in (6, 0xE))
                c = [0, 0, 0, c[3], c[4]] if moves else [0] * 5
            elif c[3] == 0xF and c[4] >= 0x20:
                c = c[:3] + [0, 0]
            if not any(c):
                body.append(0x80)
            elif all(c) and c[0] < 0x80:
                body += bytes(c)
            else:
                body.append(0x80 | sum(1 << k for k in range(5) if c[k]))
                body += bytes(v for v in c if v)
        out += struct.pack('<IBHH', 9, 0, rows, len(body)) + body
    deltas = b''.join(struct.pack('<h', 1 if i % 4 == 0 else 0) for i in range(RAMP))
    for volume in volumes:
        head = struct.pack('<I', 263) + b'ramp'.ljust(22) + b'\0' + struct.pack('<H', 1) + struct.pack('<I', 40)
        out += head.ljust(263, b'\0')
        out += struct.pack('<IIIBbBBbB', len(deltas), 0, 0, volume, 0, 0x10, 128, 0, 0) + b'ramp'.ljust(22) + deltas
    return bytes(out)


def samples(wav):
    raw = open(wav, 'rb').read()
    at = 12
    while at + 8 <= len(raw):
        tag, size = raw[at:at + 4], struct.unpack_from('<I', raw, at + 4)[0]
        if tag == b'data':
            size = min(size, len(raw) - at - 8) // 4 * 4
            return np.frombuffer(raw[at + 8:at + 8 + size], dtype='<f4').astype(np.float64)
        at += 8 + size + (size & 1)
    raise ValueError('no data in the render')


def compare(tag, keep, x, ours, effect, show):
    """One channel's line; True if every tick of the effect is the same."""
    n = min(len(ours), len(x) // TICK)
    ref, mine = [], []
    for k in range(n):
        a, b = k * TICK + TICK * 4 // 10, k * TICK + TICK * 9 // 10
        slope = (x[b] - x[a]) / (b - a)
        age = x[a] / slope - TICK * 4 // 10 if slope > 1e-9 else None
        ref.append((slope, age is not None and age < TICK * 0.5))
        semitones, volume, begun = float(ours[k][0]), int(ours[k][1]), ours[k][2] == '1'
        # The level is the channel's volume under the global one.
        mine.append((volume * int(ours[k][7]) / 64 * 2 ** (semitones / 12), begun, volume))
    rs, ms = np.array([r[0] for r in ref]), np.array([m[0] for m in mine])
    good = (rs > 1e-9) & (ms > 0)
    if not good.any():
        print('%s ch %d: nothing to compare' % (tag, keep))
        return None
    scale = np.median(ms[good] / rs[good])
    level = begins = total = other = shown = 0
    pos = 0.0
    for k in range(n):
        r, m = rs[k] * scale, ms[k]
        # Where the ramp is: at its end in either is no difference.
        step = 8363 * 2 ** ((float(ours[k][0]) - 48) / 12) * 0.02
        pos = step if mine[k][1] or ref[k][1] else pos + step
        if pos > RAMP - 2 * step or (r < 0.3 and m == 0):
            continue
        differs = abs(r - m) > max(0.08 * max(r, m), 1.2 * m / max(mine[k][2], 1))
        begun = ref[k][1] != mine[k][1] and r > 0.3 and m > 0
        cell = [0, 0, int(ours[k][4], 16), int(ours[k][5], 16), int(ours[k][6], 16)]
        if effect is not None and not has(cell, effect):
            other += differs or begun
            continue
        total += 1
        level += differs
        begins += begun
        if (differs or begun) and shown < show:
            shown += 1
            print('   tick %d: openmpt123 %.1f%s, the player %.1f%s (volume %d, semitone %.2f, column %s, effect %s %s)' % (
                k, r, '*' if ref[k][1] else '', m, '*' if mine[k][1] else '', mine[k][2], float(ours[k][0]), ours[k][4], ours[k][5], ours[k][6]))
    print("%s ch %d: %d ticks%s: the level differs on %d, a note begins in one alone on %d%s" % (
        tag, keep, total, '' if effect is None else ' of the effect', level, begins, '' if effect is None else '; other ticks that differ: %d' % other))
    return level == 0 and begins == 0


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('modules', nargs='*')
    p.add_argument('--effect', required=True, help='A, E6, R…, or `any`')
    p.add_argument('--list', help='a file of module paths, one a line')
    p.add_argument('--show', type=int, default=0, help="print a channel's first N differing ticks")
    args = p.parse_args()
    effect = effect_of(args.effect)
    paths = args.modules + ([l for l in open(args.list).read().split('\n') if l] if args.list else [])
    same = channels_seen = 0
    with tempfile.TemporaryDirectory() as tmp:
        xm, wav, xmc, dump = (os.path.join(tmp, 'm' + e) for e in ('.xm', '.wav', '.xmc', '.ticks'))
        for path in paths:
            tag = os.path.basename(path)[:44]
            d = open(path, 'rb').read()
            if not d.startswith(b'Extended Module: '):
                continue
            hsize, channels, pats, volumes = parse(d)
            if not 0 < channels <= 20:
                print('%s: %d channels, not converted' % (tag, channels))
                continue
            for keep in sorted({i % channels for _, cells in pats for i, c in enumerate(cells) if has(c, effect)}):
                open(xm, 'wb').write(rebuilt(d, hsize, channels, pats, volumes, keep))
                subprocess.run(['openmpt123', '--batch', '--quiet', '--force', '--samplerate', str(RATE), '--channels', '1', '--float',
                                '--filter', '1', '--ramping', '0', '-o', wav, xm], check=True, stdout=subprocess.DEVNULL)
                made = subprocess.run(['cargo', 'pack', '--clip', xmc, xm + ':samples=16,passes=0'], capture_output=True, text=True)
                if made.returncode:
                    print('%s ch %d: not converted: %s' % (tag, keep, made.stderr.strip()[-120:]))
                    continue
                subprocess.run(['cargo', 'host-test', '-q', 'dump_the_ticks', '--', '--ignored'], capture_output=True, check=True,
                               env=dict(os.environ, CLIP_FILE=xmc, CLIP_TICK_DUMP=dump, CLIP_CHANNEL=str(keep)))
                result = compare(tag, keep, samples(wav), [l.split() for l in open(dump)], effect, args.show)
                if result is not None:
                    channels_seen += 1
                    same += result
    print('%d channels, %d of them the same on every tick%s' % (channels_seen, same, '' if effect is None else ' of the effect'))


if __name__ == '__main__':
    sys.exit(main())
