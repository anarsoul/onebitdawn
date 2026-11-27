#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
"""Which effects the XM modules of a collection use.

    scripts/xm-effects.py ~/Music/modules              # every effect, by modules
    scripts/xm-effects.py ~/Music/modules --effect E6  # the modules with it
    scripts/xm-effects.py ~/Music/modules --effect R --paths > list.txt

With no `--effect`: a line an effect (0 to F, E0 to EF, G and on, as
FastTracker names them) and a volume column's kind (`v6` is the column's
6x), with the number of modules that use it and of those the number
the pack maker converts today (32 channels at most), the most
used first. It is how to tell what an effect the
player lacks would be worth.

With `--effect`: the modules that use it, the most cells first, each
with the cells and whether it converts (`channels` is why not).
`--table amiga` or `--table linear` takes only the modules on that
frequency table; with `--effect any`, every module of it. `--paths` prints the paths alone, of those that convert, for
`cargo collection --list` and `scripts/openmpt-compare.py --list`.

Paths are files or folders, searched to the bottom for `.xm`. An
arpeggio is counted only with a parameter, and a cell of the order
list's unused patterns counts as any other.
"""
import argparse, os, struct, sys
from collections import Counter


def name(effect, param):
    if effect == 0xE:
        return 'E%X' % (param >> 4)
    return '%X' % effect if effect < 16 else chr(ord('G') + effect - 16)


def scan(path):
    """(why it does not convert or '', a Counter of effect names) or None."""
    d = open(path, 'rb').read()
    if not d.startswith(b'Extended Module: ') or len(d) < 80:
        return None
    u16 = lambda at: struct.unpack_from('<H', d, at)[0] if at + 2 <= len(d) else 0
    u32 = lambda at: struct.unpack_from('<I', d, at)[0] if at + 4 <= len(d) else 0
    channels, patterns, flags = u16(68), u16(70), u16(74)
    why = 'channels' if not 0 < channels <= 32 else ''
    amiga = not flags & 1
    used = Counter()
    at = 60 + u32(60)
    for _ in range(patterns):
        q = at + u32(at)
        end = min(q + u16(at + 7), len(d))
        at = end
        while q < end:
            b = d[q]
            q += 1
            volume = effect = param = 0
            if b & 0x80:
                q += (b & 1) + (b >> 1 & 1)
                if b & 4 and q < end:
                    volume = d[q]
                    q += 1
                if b & 8 and q < end:
                    effect = d[q]
                    q += 1
                if b & 16 and q < end:
                    param = d[q]
                    q += 1
            elif q + 4 <= end:
                volume, effect, param = d[q + 1], d[q + 2], d[q + 3]
                q += 4
            else:
                break
            if effect or param:
                used[name(effect, param)] += 1
            if volume >= 0x60:
                used['v%X' % (volume >> 4)] += 1
    return why, used, amiga


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('paths', nargs='+')
    p.add_argument('--effect', help='an effect as FastTracker names it: A, E6, R, v6')
    p.add_argument('--table', choices=('amiga', 'linear'), help='only the modules on this frequency table')
    p.add_argument('--paths', dest='only_paths', action='store_true', help='with --effect: the paths of the modules that convert, alone')
    args = p.parse_args()
    files = []
    for path in args.paths:
        if os.path.isdir(path):
            for top, _, names in sorted(os.walk(path)):
                files += [os.path.join(top, n) for n in sorted(names) if n.lower().endswith('.xm')]
        else:
            files.append(path)
    modules, convertible = Counter(), Counter()
    found = []
    total = 0
    for path in files:
        try:
            got = scan(path)
        except OSError:
            got = None
        if got is None:
            continue
        why, used, amiga = got
        if args.table and (args.table == 'amiga') != amiga:
            continue
        total += 1
        for effect in used:
            modules[effect] += 1
            convertible[effect] += not why
        if args.effect == 'any':
            found.append((sum(used.values()), why, path))
        elif args.effect and used.get(args.effect):
            found.append((used[args.effect], why, path))
    if args.effect:
        for cells, why, path in sorted(found, key=lambda f: (-f[0], f[2])):
            if args.only_paths:
                if not why:
                    print(path)
            else:
                print('%6d  %-8s  %s' % (cells, why or 'converts', path))
        if not args.only_paths:
            print('%d of %d modules use %s, %d of them convert' % (len(found), total, args.effect, sum(not f[1] for f in found)))
    else:
        print('effect  modules  convert')
        for effect, n in sorted(modules.items(), key=lambda m: (-m[1], m[0])):
            print('%-6s  %7d  %7d' % (effect, n, convertible[effect]))
        print('%d modules' % total)


if __name__ == '__main__':
    sys.exit(main())
