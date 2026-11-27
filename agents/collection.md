# The module player against a collection and against OpenMPT

How to hold the pack maker's tracker converters and the clock's module
player (`core/src/xm.rs`) to a collection of modules kept outside the
repository, and to openmpt123, which is the reference: what a module
is heard in today. Read it before adding or changing an effect, the
XM, MOD or S3M converter, or how a tune ends.

Nothing here runs in the CI: the modules are other people's music and
stay where they are, and the tools need `openmpt123` (0.8.10 when this
was written; the package `libopenmpt` or `openmpt123`) and `numpy`.
Run everything from the repository's root. The collection used so far
is the user's key-generator music, 937 XM modules; any folder of
`.xm`, `.mod` and `.s3m` does.

## The tools

| tool | what it answers |
| --- | --- |
| `cargo collection path…` | does every module convert or get refused in words, and does its clip play to an end; what do the modules use that the player lacks |
| `scripts/xm-effects.py path…` | which effects do the modules use, and which modules use a given one |
| `scripts/openmpt-ticks.py 'rows'` | what does OpenMPT do with these rows, tick by tick: the rules of an effect |
| `scripts/openmpt-channels.py --effect X module…` | does the player do the same as OpenMPT on the channels of real modules that use the effect, tick by tick |
| `scripts/openmpt-compare.py dir` | are whole tunes as long and as loud over time as OpenMPT plays them |

Each script's `--help` is its full account; `cargo collection`'s is the
header of `tools/pack/examples/collection.rs`.

## Adding or changing an effect

The order that worked for R, H, EE and E6 (their sections are in
`PLAN.md` or `HISTORY.md`):

1. **What is it worth.** `scripts/xm-effects.py DIR` lists every effect
   by the modules that use it and how many of those convert;
   `--effect E6` lists the modules. `cargo collection DIR` ends with
   what the converter names as not played, by files.
2. **The rules, from renders.** Do not write an effect from memory of
   a tracker's manual or from one replayer's source: FastTracker's own
   replayer and OpenMPT differ in corners, and the user chose OpenMPT.
   Ask it, a rule a render:

   ```
   scripts/openmpt-ticks.py 'row(C4, 1, 0x30); row(fx=0xA, p=0x01, other=(E, 0xE2)); row()'
   scripts/openmpt-ticks.py --speed 3 --rows 'row(vol=0x14, fx=E, p=0x60); row(vol=0x18); row(vol=0x1C, fx=E, p=0x62); row(vol=0x20)'
   ```

   Try the memory (a parameter of nought), both ends of a range, the
   effect beside each thing it could meet on a row (a note, an
   instrument alone, the volume column, a delay, a break, the same
   effect in the other channel), another speed, and what the next row
   and the next pattern inherit. A render that surprises is a rule.
   OpenMPT's source (`soundlib/Snd_fx.cpp`, `Load_xm.cpp`) and
   FastTracker's (8bitbubsy's `ft2-clone`, `pmp_main.c` of `ft2play`)
   explain a render; they do not replace it.
3. **The test.** The expectations of the player's test are those
   renders, written as the script prints them
   (`r_retriggers…`, `h_slides…`, `ee_holds…`, `e6_loops…` in
   `core/src/xm.rs`); each case is a fresh module, as each render is.
4. **The pack maker.** An effect the player gains is one a firmware
   before it lacks: the maker names it as not played for that firmware
   (`Firmware::retriggers`, `Shape::retrigger`) and for no later one.
   An effect that changes how long a tune is goes into the length the
   clip's header gives (`xmc::convert`'s row-by-row count, which must
   follow `Tune::step_tick`), and one that can keep a tune from ending
   is refused there.
5. **Real modules, tick by tick.**

   ```
   scripts/xm-effects.py DIR --effect R --paths > /tmp/r.txt
   scripts/openmpt-channels.py --effect R --list /tmp/r.txt
   ```

   Look at the channels that differ (`--show 5`) before believing
   them: the script's `--help` lists the differences that are the
   measure's. For R, 156 of 169 channels were the same and the six of
   the rest looked at were the measure's.
6. **Whole tunes.**

   ```
   cargo collection --raw /tmp/raw --list /tmp/r.txt
   scripts/openmpt-compare.py /tmp/raw
   ```

   Lengths that agree to a few tenths of a second, a loudness that
   follows OpenMPT's (a correlation of 0.9 or so in the median), no
   gaps. Run it over a control of modules without the effect when the
   figures are in doubt: what the player lacks elsewhere lowers them
   too.
7. **The whole collection**, with no `--raw`: no panic, no tune that
   does not end, nothing newly refused or silent. Seventeen hours of
   music take some ten minutes.
8. The firmware's build, clippy, `.text`, `.rwtext` and `.stack`
   against a build of `HEAD` (`agents/sizes.md`), and the documents: the
   work's section, `CHANGELOG.md`, `MANUAL.md` and the pack maker's
   page where they say what a firmware lacks, `agents/clips.md`.

A module with the effect heard on a clock is still owed after all of
it, and is the user's.

## What the measures cannot say

- `openmpt-compare.py` hears length, silence and a missing
  instrument, not a wrong note. With the limiter on (a pack's clips as
  they are made) loudness is flat and the correlation means nothing:
  that is why `--raw` turns it off.
- A tune played once ends for the clock at a jump back in the order
  list or at the list's end; OpenMPT ends at a row it has played in the
  same state, so after a pattern loop it may go round a jump back once
  more. Three modules of the collection are longer in OpenMPT for
  that, by 7, 37 and 42 seconds, and are right as they are.
- OpenMPT plays a file it takes for an old ModPlug Tracker's by other
  rules for some effects (R for one). `openmpt123 --info FILE` says
  what it takes a file for. The scripts write their modules under a
  tracker's name it does not know, so they get FastTracker's rules;
  28 of the collection's 99 modules with an R are played by the
  others, and sound otherwise in OpenMPT than on the clock.
- A render's slope tells a pitch to a few parts in a hundred, the
  ramp having a step every fourth sample: enough for an octave, not
  for a table. Where the ramp is at a tick's end is told to four
  samples whatever came before, so a pitch is held to OpenMPT by
  adding up where a model's periods put the ramp, tick by tick, and
  a finetune by the place after a second of one note (the Amiga
  table's section in `PLAN.md` was measured so).
- A level that changes every tick (an envelope, a tremolo) is not
  read from a ramp's slope, which has the level's own slope in it:
  give every instrument a sample of one value and read the render at
  each tick's end, which is the volume (the tremolo and L were
  measured so, with `openmpt-ticks.py`'s module patched).
  `openmpt-channels.py` cannot hold a tremolo to OpenMPT for the
  same reason.
- OpenMPT moves a level to its new value across a tick, the global
  volume above all: a slide's ticks read low or high in a render, and
  are right where the next row begins.

## What the collection has shown (2026-11-26)

Of its 1196 modules (937 of them XM) 1129 convert and play to their
ends, 35 hours, with no panic; 59 of the 67 refused have more than
twenty channels. Found with
it and fixed: files that end before their instruments do and
instruments that give a wrong size for a sample's header (80
modules), the effects R, H, EE and E6, the Amiga frequency table
(245 modules), an instrument named with no note, the effects 7, T,
X, L and E5, orders with no pattern. Found and not fixed: `PLAN.md`'s
backlog.
