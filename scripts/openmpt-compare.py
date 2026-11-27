#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
"""Hold the clock's renders of a collection's modules to openmpt123's.

    cargo collection --raw /tmp/raw ~/Music/modules
    scripts/openmpt-compare.py /tmp/raw [--worst 20] [--list FILE]

Reads `index.tsv` and the `.raw` files `cargo collection --raw` wrote
(the speaker's clips with the limiter off), renders each module with
openmpt123 at the same rate, mono, and prints a line a module, the
worst first:

    length   the clock's length less openmpt123's, in seconds
    loud     the correlation of the two loudnesses over time, in half
             seconds: 1 is the same shape, under 0.5 wants a look
    gaps     the share of half seconds in which one sounds and the other
             does not: a tune silent where it should not be, or the
             other way

It is a coarse measure: it finds a tune that is cut short, runs long,
is silent, or has an instrument missing, and says nothing of one wrong
note. The loudness of two right renders correlates at 0.9 or so; it
is lower where the tune uses what the player lacks (the pack maker
names it) and for a tune of even loudness, where there is no shape to
follow. The clock ends a tune played once at a jump back in the order
list; openmpt123 may go round once more, which is a length that
differs and no fault. `agents/collection.md` has what was found so.

Needs openmpt123 and numpy. `--list FILE` takes only the modules whose
paths FILE names, one a line.
"""
import argparse, os, struct, subprocess, sys, tempfile

import numpy as np


def wav(path):
    """The samples of a WAV file of 32-bit floats, by its chunks."""
    raw = open(path, 'rb').read()
    at = 12
    while at + 8 <= len(raw):
        tag, size = raw[at:at + 4], struct.unpack_from('<I', raw, at + 4)[0]
        if tag == b'data':
            size = min(size, len(raw) - at - 8) // 4 * 4
            return np.frombuffer(raw[at + 8:at + 8 + size], dtype='<f4').astype(np.float64)
        at += 8 + size + (size & 1)
    raise ValueError('no data chunk in ' + path)


def loudness(x, rate):
    """The RMS of each half second."""
    w = rate // 2
    n = len(x) // w
    return np.sqrt((x[:n * w].reshape(n, w) ** 2).mean(axis=1)) if n else np.zeros(0)


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('raw', help="the folder `cargo collection --raw` wrote")
    p.add_argument('--worst', type=int, default=0, help='print only so many lines, the worst')
    p.add_argument('--list', help='a file of module paths: only these')
    args = p.parse_args()
    only = set(open(args.list).read().split('\n')) if args.list else None
    rows = []
    with tempfile.TemporaryDirectory() as tmp:
        ref_path = os.path.join(tmp, 'ref.wav')
        for line in open(os.path.join(args.raw, 'index.tsv')):
            n, rate, _ms, path = line.rstrip('\n').split('\t', 3)
            if only is not None and path not in only:
                continue
            rate = int(rate)
            ours = np.fromfile(os.path.join(args.raw, n + '.raw'), dtype='<i2').astype(np.float64) / 32768
            done = subprocess.run(
                ['openmpt123', '--batch', '--quiet', '--force', '--samplerate', str(rate), '--channels', '1', '--float', '-o', ref_path, path],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if done.returncode != 0:
                print('openmpt123 does not play: ' + path)
                continue
            ref = wav(ref_path)
            a, b = loudness(ours, rate), loudness(ref, rate)
            k = min(len(a), len(b))
            a, b = a[:k], b[:k]
            if k < 4 or a.max() == 0 or b.max() == 0 or a.std() == 0 or b.std() == 0:
                loud = float('nan')
            else:
                loud = float(np.corrcoef(a, b)[0, 1])
            gaps = float(((a > a.max() * 0.02) != (b > b.max() * 0.02)).mean()) if k and a.max() > 0 and b.max() > 0 else 1.0
            rows.append((len(ours) / rate - len(ref) / rate, loud, gaps, len(ours) / rate, path))
    # The worst first: a length that differs, then a shape that does.
    def badness(r):
        length, loud, gaps = abs(r[0]), r[1], r[2]
        return -(min(length, 60) / 2 + (1 - (0 if loud != loud else loud)) * 10 + gaps * 20)
    rows.sort(key=badness)
    print('length    loud  gaps  seconds  module')
    for length, loud, gaps, seconds, path in rows[:args.worst or None]:
        print('%+7.1f  %5.2f  %4.2f  %7.1f  %s' % (length, loud, gaps, seconds, path))
    if rows:
        louds = sorted(r[1] for r in rows if r[1] == r[1])
        print('%d modules: %d within 0.3 s of openmpt123 in length; loudness correlation median %.2f, %d under 0.5; %d with gaps over 0.1' % (
            len(rows), sum(abs(r[0]) <= 0.3 for r in rows), louds[len(louds) // 2] if louds else float('nan'),
            sum(x < 0.5 for x in louds), sum(r[2] > 0.1 for r in rows)))


if __name__ == '__main__':
    sys.exit(main())
