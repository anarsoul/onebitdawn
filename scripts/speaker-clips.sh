#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
# Make the default clip pack for a clock with a speaker in the buzzer's
# place (speaker-mod/README.md): the eight clips of clips/, the
# XM modules converted again from mods/ with the bass left in (a high-pass
# at 100 Hz, once) and the limiter letting go over 128 ms and not 16:
# as hard a drive as the buzzer's clips have, but a gain that follows
# the tune's level and not its waveform, which on a speaker is heard
# as overdrive. Some 2 dB under the buzzer's clips; with no limiter at
# all (--knee) they were 6 dB under and too quiet (2026-11-15).
# Synchronization has the noise at full level, the high-pass at
# 100 Hz, the release at 128 ms, and a gentler limiter than the
# modules': its knee at 37 (22 dB over the player's own, so the tune's
# quiet parts are at a plain gain) and its peaks aimed 1.4 dB over
# full scale, the user's choice by ear of 2026-11-16 over the hard
# limiter, which the buzzer's pack has. The knee wants a firmware of
# that day or later; an older one limits hard.
# Suspended Seconds is a piano, whose attacks a limiter that follows
# the waveform flattens into clicks: its limiter lets go over a
# second (release=13) from a low knee, so it rides a passage's level
# and not a note's (the user's choice, 2026-11-19; the buzzer's clip
# has release=10).
# Starlight and Aurora is this pack's alone, its ninth and last: a
# soft piano under 350 Hz and a bass under 110, which on a buzzer does
# not sound (the user, 2026-11-20), so the buzzer's pack and clips/
# have no clip of it. Its limiter is off, a knee of 64 being over the
# tune's own peak: at Suspended Seconds' 12 its four parts together
# were always over the knee and every new peak was flattened at full
# scale, 1254 samples of the clip, which the user heard as clicks on
# test-clock's speaker; at 48 it was 63, at 64 none, 5 dB quieter
# than at 12 and as loud as Suspended Seconds (the user, 2026-11-20:
# "Лимитер выключай вообще, knee=64"). Its samples are of 16 bits, and
# so are Suspended Seconds' piano's: cut to 8 the piano alone, as the
# tune starts, hisses. The pack maker keeps them predicted, in 8 bits
# a sample and a sixteenth more, unless told. That
# makes them clips a firmware up to v0.5 refuses, and this pack with
# them. Keygen's one 16-bit sample is 8 bits in a 16-bit one's place
# and is written as 8, so its clip is what it was. The high-pass stays at 100 Hz: the user's
# choice on white-clock's speaker over 60 Hz, with either limiter
# ("баса почти не слышно, ну и ладно").
# The pack maker's `--sound-output speaker --limiter loudest` is these settings, so the
# pack is the default one's sources through it: packs/clips-default-speaker.bin,
# under the default pack's names, so a clock's melody setting means the
# same tune after it (the ninth but for a clock that had it chosen and
# is given the buzzer's pack).
# With `amplifier` after the path the pack is for a speaker on an
# amplifier at the line output, whose volume is the amplifier's to
# set: the modules that give no knee of their own (Keygen,
# the three of Grieg's and Serial Dawn) are made with `--limiter cleanest`, their
# knee fitted so the limiter flattens next to no peak (and
# Synchronization's, whose peaks are then aimed at full scale and not
# 1.4 dB over it), some 6 dB
# quieter and with none of the clicks the hard limiter leaves under a
# loud tune (the user, 2026-11-24, who heard them on headphones). A
# speaker in the buzzer's place, on its transistor, has no volume to
# turn up and gets the pack as it was. The two tunes with a knee of
# their own are the same in both.
# Usage: scripts/speaker-clips.sh [pack.bin [amplifier]]
#                                               the pack's path, from the
#                                               repository's root; the
#                                               release names its own
#        API_TOKEN=… scripts/clips.sh <ip> packs/clips-default-speaker.bin
set -eu

cd "$(dirname "$0")/.."
case ${2:-} in
    '') limiter=loudest; default=packs/clips-default-speaker.bin ;;
    amplifier) limiter=cleanest; default=packs/clips-default-amplifier.bin ;;
    *) echo "usage: $0 [pack.bin [amplifier]]" >&2; exit 2 ;;
esac
pack=${1:-$default}
cargo pack -o "$pack" --sound-output speaker --limiter "$limiter" \
    mods/Orion-Nero6.3.0.2.xm:name=keygen \
    mods/nq-synchronization.pt3:restart,name=synchronization mods/nq-lera.xm:name=lera \
    mods/grieg-morning-bright.xm mods/grieg-morning-chip.xm mods/grieg-mountain-king-bright.xm \
    mods/suspended-seconds.xm:knee=12,release=13 mods/serial-dawn.xm \
    mods/starlight-aurora.xm:knee=64,release=13
