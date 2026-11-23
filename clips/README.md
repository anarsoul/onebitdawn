# Clips

The clips of the default clip pack, the extra alarm melodies a clock
has in its `clips` partition (see "Clip packs" in `MANUAL.md`):

```
scripts/clips.sh                                       # packs/clips-default.bin, these eight
cargo pack -o packs/one.bin clips/keygen.xmc           # a pack of one of them
ALARM_CLIP=clips/keygen.xmc cargo build --release      # or built into the firmware, the older way
```

The first three are other people's music and are **not** covered by
the project's MIT OR Apache-2.0 licence; each is credited below with
its terms (the first is CC BY 3.0), and
copyright stays with its author. The next three were made here, and
so was the last. The seventh is the project's author's own tune, and
is **not** under the project's licence either; so is a ninth that only the speaker's pack
has, at the end of this file.

## `keygen.xmc`

"keygentune2003 orn" by **Dualtrax** (of Chemical Reaction and
Analogik), a chiptune written as a FastTracker 2 module for the key
generators of the group ORiON. The module is signed in its instrument
names, `dualtrax/cro/`; see
[Dualtrax on Demozoo](https://demozoo.org/sceners/5763/). The music
is Dualtrax's, and copyright remains with its author. He has
published it as
["dualtrax - orion keygentune 2003"](https://soundcloud.com/dualtrax/dualtrax-orion-keygentune-2003)
under the Creative Commons Attribution 3.0 Unported licence,
[CC BY 3.0](https://creativecommons.org/licenses/by/3.0/), and it is
here, and in the clip packs, under that licence. `keygen.xmc` is
changed from his module only as described below: converted for the
clock's player, and labelled "Keygen" on the clock's pages.

The module itself is in `mods/` as it was found, and this file is
what the pack maker (`tools/pack`) makes of it for the clock's
own player (`core/src/xm.rs`), with its defaults: the same patterns
and samples rearranged, nothing rendered.

```
cargo pack --clip clips/keygen.xmc mods/Orion-Nero6.3.0.2.xm
```

## `synchronization.pt3c`

"synchronization" (2015) by **n1k-o** (Oleg Nikitin), a ZX Spectrum
tune for the AY-3-8910, written in Vortex Tracker II; the module is
signed `nq`. It is here with his permission, given to this project
(2026-11-16). The music is his; copyright remains with
its author, and nothing here grants any right to it beyond that.

The module is in `mods/` as it was found, and this file is the module
as it is behind the header the pack maker gives it for the
clock's own player and AY (`core/src/pt3.rs`, `core/src/ay.rs`):
nothing rendered. The header asks the player for its limiter, letting
go over 128 ms, and a high-pass at 100 Hz.

```
cargo pack --clip clips/synchronization.pt3c mods/nq-synchronization.pt3:restart,name=synchronization
```

## `lera.xmc`

"lerka" (2012) by **n1k-o** (Oleg Nikitin), a chiptune written as a
FastTracker 2 module for the key generators of the group TSRh; the
module is signed `nq/skrju`, which is he, and titled
`nq/skrju - lerka`. It is here with his permission, given to this
project (2026-11-26). The music is his; copyright remains with its
author, and nothing here grants any right to it beyond that.

The module is in `mods/` as it was found, and this file is what the
pack maker makes of it for the clock's own player, with its defaults:
the same patterns and samples rearranged, nothing rendered, labelled
"Lera" on the clock's pages. One thing in the patterns is written
another way: the module's drum rolls are the effect `R02`, which the
player has not, and the pack maker writes an `R` that changes no
volume as `E9`, the same retrigger, which it has.

```
cargo pack --clip clips/lera.xmc mods/nq-lera.xm:name=lera
```

## `morning_mood.xmc`, `morning_mood_chip.xmc` and `mountain_king.xmc`

Two movements of **Edvard Grieg**'s first *Peer Gynt* suite, Op. 46
(1888; he died in 1907 and the music is in the public domain), as
tracker modules made for this project: *Morning Mood* (No. 1) twice,
with an orchestra's recorded instruments and with plain waves as a
chiptune has them, and *In the Hall of the Mountain King* (No. 4) with
the orchestra's. A fourth, the Mountain King with plain waves, is in
`mods/` and not in the pack.

- **The notes.** The Mountain King's are the Mutopia Project's piano
  score, [piece 1888](https://www.mutopiaproject.org/cgibin/piece-info.cgi?id=1888),
  typeset by Coyau and placed in the public domain; its MIDI file and
  LilyPond source are in `mods/src/` as Mutopia gives them. Morning's
  were read here off the composer's own arrangement for piano four
  hands, Edition Peters 2432, in the Sibley Music Library's
  [scan](https://hdl.handle.net/1802/16617). It is shortened: bars 1
  to 31 and 56 to 76, where a bar's rest ends it.
- **The instruments.** The orchestra's are recordings from Versilian
  Studios' [VSCO 2 Community Edition](https://github.com/sgossner/VSCO-2-CE),
  which is CC0, cut down to a note an instrument. The plain waves are
  made by the script.
- **The arrangements and the modules** are this project's, under its
  licence, MIT OR Apache-2.0.

The modules are written by `scripts/mkxm.py` from the tunes in
`mods/src/`, and the clips are the pack maker's defaults of them:

```
scripts/mkxm.py mods/src/morning.py mods/grieg-morning-bright.xm bright
scripts/mkxm.py mods/src/morning.py mods/grieg-morning-chip.xm chip
scripts/mkxm.py mods/src/mountain_king.py mods/grieg-mountain-king-bright.xm bright
scripts/mkxm.py mods/src/mountain_king.py mods/grieg-mountain-king-chip.xm chip
cargo pack --clip clips/morning_mood.xmc mods/grieg-morning-bright.xm
cargo pack --clip clips/morning_mood_chip.xmc mods/grieg-morning-chip.xm
cargo pack --clip clips/mountain_king.xmc mods/grieg-mountain-king-bright.xm
```

## `suspended_seconds.xmc`

"Suspended Seconds" (2026) by **Vasily Khoruzhick**, a tune for
piano, written for this clock. **Copyright (c) 2026 Vasily
Khoruzhick. May be copied and distributed, unmodified or as converted
from its module by onebitdawn's tools, as part of onebitdawn clip
packs, including commercially; all other rights reserved.** The tune, its MIDI file
(`mods/src/suspended_seconds.mid`), the module
(`mods/suspended-seconds.xm`) and this clip are under these terms and
not under the project's licence.

- **The notes** are the composer's MIDI file, with nothing arranged:
  each note's velocity is its volume, a note let go under the sustain
  pedal rings until the pedal comes up, and the notes' starts are on
  their eighths.
- **The piano** is the upright of Versilian Studios'
  [VSCO 2 Community Edition](https://github.com/sgossner/VSCO-2-CE),
  which is CC0, at mezzo-forte: a recording every C and G, the first
  second or so of each and a loop, with the string's decay in the
  instrument's envelope.
- **The script** that writes the module, `mods/src/suspended_seconds.py`
  with `scripts/mkxm.py`, is the project's, under its licence.

A piano's attacks are flattened into clicks by a limiter that follows
the waveform, so this clip's limiter lets go slowly, from a low knee:

```
scripts/mkxm.py mods/src/suspended_seconds.py mods/suspended-seconds.xm
cargo pack --clip clips/suspended_seconds.xmc mods/suspended-seconds.xm:knee=12,release=10
```

For a speaker (`scripts/speaker-clips.sh`) it is `knee=12,release=13`.
The piano's samples are of 16 bits in the module and the clip keeps
them in about a byte each, predicted, which is what the pack maker
does unless told; such a clip needs a firmware after v0.5.

## `serial_dawn.xmc`

"Serial Dawn" (2026), a chiptune in the manner of the key generators'
music, written for this project as an alarm for a buzzer and a small
speaker: a rising arpeggio, then A minor's tonic, subdominant and
dominant under a pulse lead. Nothing in it is recorded: its
instruments are pulse waves, a swept sine and noise that the script
makes. The tune, the script and the module are this project's, under
its licence, MIT OR Apache-2.0.

```
scripts/mkxm.py mods/src/serial_dawn.py mods/serial-dawn.xm
cargo pack --clip clips/serial_dawn.xmc mods/serial-dawn.xm
```

A ringing alarm plays it round from after its introduction.

## "Starlight & Aurora", in the speaker's pack only

"Starlight & Aurora" (2026) by **Vasily Khoruzhick**, a tune for
piano, a bell, a lead and a bass ("Starlight and Aurora" on the
clock's pages, where a label may not have an `&`). **Copyright (c)
2026 Vasily Khoruzhick. May be copied and distributed, unmodified or
as converted from its module by onebitdawn's tools, as part of
onebitdawn clip packs, including commercially; all other rights
reserved.** The tune, its MIDI file
(`mods/src/starlight_aurora.mid`), the module
(`mods/starlight-aurora.xm`) and the clip made of it are under these
terms and not under the project's licence. There is no clip of it
here: it is a soft piano under 350 Hz over a bass under 110, which a
buzzer does not say, so only the pack for a clock with a speaker
(`scripts/speaker-clips.sh`) has it, made from the module.

- **The notes** are the composer's MIDI file, with nothing arranged:
  four parts, each note's velocity its volume, the times as they are
  in the file.
- **The piano** is [Surge XT](https://surge-synthesizer.github.io/)'s
  patch "Acoustic Grand" (of its LinnStrument MPE patches), which the
  composer played the part with: each of the part's seven notes
  rendered from the synthesizer by itself, soft and loud, the first
  0.9 s of each and a loop, with the note's decay in the instrument's
  envelope. Surge XT is free software under the GPL 3; nothing of its
  code is here, only the sound of ten notes.
- **The bell** stands for JE-8086's "MKS80 Bell/Space", which the
  composer used: made in the script after what his recording of the
  part measures as, a tone two octaves over the note, a click, and
  the note itself coming up late.
- **The lead and the bass** are sines made in the script; the bass
  has harmonics added, to be heard by on a small speaker.
- **The script** that writes the module, `mods/src/starlight_aurora.py`
  with `scripts/mkxm.py`, is the project's, under its licence, as is
  `scripts/surgenotes.py`, which renders the piano's notes (it needs
  Surge XT's VST3 plugin and `pedalboard`).

```
scripts/surgenotes.py "/usr/share/surge-xt/patches_3rdparty/LinnStrument MPE/Keys/Acoustic Grand.fxp" \
    xm/surge/acoustic-grand F3:56 F3:78 G3:56 A3:78 C4:56 C4:78 D4:56 E4:56 E4:78 F4:78
scripts/mkxm.py mods/src/starlight_aurora.py mods/starlight-aurora.xm
scripts/speaker-clips.sh        # the module with knee=64,release=13, the pack's ninth clip
```

The limiter is off, a knee of 64 being over the tune's peak: at the
defaults the piano alone is as loud as the four parts together, and
at the other tune's knee of 12 the peaks are flattened at full
scale, which a speaker plays as clicks. Its samples are of 16
bits and the clip keeps them predicted, in 8 bits a sample and a
sixteenth more, as the pack maker does unless told; such a clip needs
a firmware after v0.5.
