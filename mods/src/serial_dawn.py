# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
"""Serial Dawn, a tune in the manner of the key generators' chiptunes,
written for the clock's alarm, for `scripts/mkxm.py`.

Nothing in it is recorded: the instruments are pulses of three widths,
a swept sine for the kick and noise for the snare and the hat, all
made here. It is for a buzzer and a small speaker alike, so the bass
is a pulse an octave above where a bass would sit, under the lead's
level, and is there for its harmonics.

A minor, a row a sixteenth, a bar sixteen rows, a pattern four bars:

- the introduction, four bars: sixteenths climbing through Am, Dm, E
  and E7, from one quiet voice to full level;
- A, eight bars: Am, Dm, E, Am twice (tonic, subdominant, dominant,
  tonic), the lead over chord stabs;
- B, eight bars: F, G, Em, Am, F, G, E, E, which ends on the
  introduction's climb;
- A again, eight bars, with rolling sixteenths under the lead.

Played round and round it starts over from A: the restart position is
the pattern after the introduction.
"""

import numpy as np

from mkxm import Module, Sample, midi_hz, pitch_of, pulse

BAR = 16
SPEED = 6
BPM = 150      # a tick is 2.5 / 150 s, a row 100 ms
INTRO, A1, B, A2, END = 0, 4 * BAR, 12 * BAR, 20 * BAR, 28 * BAR
ECHO = 3       # rows the echo is behind

# The chords: the root's name, and the two notes over it as effect 0
# has them, in semitones.
MINOR, MAJOR = 0x37, 0x47
CHORDS = {"Am": ("A", MINOR), "Dm": ("D", MINOR), "Em": ("E", MINOR),
          "E": ("E", MAJOR), "F": ("F", MAJOR), "G": ("G", MAJOR)}
# The bass's root for each, none under A2.
BASS = {"Am": "A2", "Dm": "D3", "Em": "E3", "E": "E3", "F": "F3", "G": "G3"}

PART_A = ["Am", "Dm", "E", "Am", "Am", "Dm", "E", "Am"]
PART_B = ["F", "G", "Em", "Am", "F", "G", "E", "E"]

LEAD_A = [
    "A5/3 C6/3 E6/2 D6/2 C6/2 B5/2 C6/2",
    "D6/3 F6/3 A6/2 G6/2 F6/2 E6/2 F6/2",
    "E6/3 G#5/3 B5/2 E6/2 D6/2 B5/2 G#5/2",
    "A5/6 r/2 E5/2 A5/2 C6/2 E6/2",
    "A6/3 E6/3 C6/2 E6/2 A6/2 G6/2 E6/2",
    "F6/3 D6/3 A5/2 D6/2 F6/2 E6/2 D6/2",
    "E6/2 E6/1 E6/1 G#6/2 B6/2 A6/2 G#6/2 E6/2 D6/2",
]
# A's last bar: a close the first time, and the pickup into A again
# the second, which is where the tune goes round.
CLOSE = "C6/2 B5/2 A5/8 r/4"
PICKUP = "A5/6 r/2 E5/2 A5/2 C6/2 E6/2"
LEAD_B = [
    "C6/4 A5/2 C6/2 F6/4 E6/2 C6/2",
    "D6/4 B5/2 D6/2 G6/4 F6/2 D6/2",
    "E6/4 B5/2 E6/2 G6/3 E6/3 B5/2",
    "A5/3 C6/3 E6/2 A6/6 r/2",
    "A6/2 F6/2 C6/2 F6/2 A6/2 F6/2 C7/2 A6/2",
    "B6/2 G6/2 D6/2 G6/2 B6/2 G6/2 D7/2 B6/2",
    "G#6/4 E6/4 B5/4 G#5/4",
]


def climb(names, steps=4):
    """Sixteenths going up a chord: four notes from its first, four
    from its second, and so on, a bar of them."""
    notes = [pitch_of(n) for n in names.split()]
    ladder = notes + [n + 12 for n in notes] + [n + 24 for n in notes]
    return [ladder[i + j] for i in range(steps) for j in range(4)]


def kick(rate=16000, seconds=0.09):
    """A sine falling from 420 Hz to 110: the thump with little under
    it, since nothing here would say it."""
    t = np.arange(int(seconds * rate)) / rate
    hz = 110 + 310 * np.exp(-t / 0.018)
    x = np.sin(2 * np.pi * np.cumsum(hz) / rate) * np.exp(-t / 0.03)
    return Sample("kick", None, 60, rate, made=(x, rate, midi_hz(60), None), gain=0.9)


def hat(rate=32000, seconds=0.05):
    """Noise with its bass taken out, a twentieth of a second."""
    n = int(seconds * rate)
    x = np.diff(np.random.default_rng(2).uniform(-1, 1, n + 1)) * np.exp(-np.arange(n) / rate / 0.012)
    return Sample("hat", None, 60, rate, made=(x, rate, midi_hz(60), None), gain=0.6)


def tune():
    m = Module("Serial Dawn", channels=14, speed=SPEED, bpm=BPM, pattern_rows=4 * BAR)

    def wave(name, duty, note, gain, **how):
        # A narrow pulse is mostly on one side of nothing, and a note
        # of it that starts or stops is a step, heard as a thump on a
        # speaker: the wave is put about its mean, which leaves its
        # peak further out, and `gain` is of that peak.
        cycle = pulse(duty)
        return m.instrument(Sample.wave(name, cycle - cycle.mean(), note, gain=gain), **how)

    # A held note falls a little after its start, as a plucked one
    # would, and is let down quickly: the notes are short.
    lead = wave("lead", 0.25, 84, 0.75, release=3, fall=(10, 44), vibrato=(30, 5, 20))
    echo = wave("echo", 0.5, 84, 0.4, release=3, fall=(6, 36))
    pluck = wave("pluck", 0.25, 76, 0.75, release=3, fall=(4, 30))
    stab = wave("stab", 0.125, 69, 0.85, release=2, fall=(4, 40))
    bass = wave("bass", 0.25, 50, 0.75, release=2, fall=(5, 40))
    drum = m.instrument(kick())
    snare = m.instrument(Sample.noise("snare", 0.16, 22050, 0.13))
    tick = m.instrument(hat())

    leads, echoes = m.voices([0, 1]), m.voices([2, 3])
    plucks, stabs = m.voices([4, 5, 6]), m.voices([7, 8])
    lows = m.voices([9, 10])
    kicks, snares, hats = m.voices([11]), m.voices([12]), m.voices([13])

    def play(voices, at, length, pitch, ins, vol, short=0.0):
        """A note that is let go inside the tune: the row after the
        last is A's first."""
        stop = min(at + length - short, END - 1 / SPEED)
        if at < END and stop > at:
            voices.note(at, stop - at, pitch, ins, vol)

    def echoed(at, length, pitch, ins, vol, short=0.0, behind=True):
        play(leads if ins is lead else plucks, at, length, pitch, ins, vol, short)
        if behind:
            play(echoes, at + ECHO, min(length, 2), pitch, echo, vol * 0.36, short)

    def phrase(at, notes, vol=52):
        for token in notes.split():
            name, length = token.split("/")
            length = int(length)
            if name != "r":
                # Detached, a long note more than a short one.
                echoed(at, length, pitch_of(name), lead, vol, short=0.5 if length > 1 else 0.25)
            at += length
        return at

    def chord(at, length, name, vol, octave=4):
        """A chord on one channel, as a chiptune has it: its root with
        effect 0 going round the other two notes, a tick each."""
        root, kind = CHORDS[name]
        play(stabs, at, length, pitch_of(f"{root}{octave}"), stab, vol)
        channel = stabs.lines[stabs.last].channel
        for row in range(at, at + length):
            c = m.cell(row, channel)
            if c[3] == 0:
                c[4] = kind

    def roll(at, rows, first, last):
        """A snare roll that grows."""
        for i in range(rows):
            snares.note(at + i, 1, 60, snare, first + (last - first) * i / max(1, rows - 1))

    def drums(at, fill=False, four=True):
        """A bar: the kick on one and three and once more before four,
        the snare on two and four, hats between."""
        for row in (0, 8, 11):
            kicks.note(at + row, 1, 60, drum, 46)
        snares.note(at + 4, 1, 60, snare, 34)
        if fill:
            roll(at + 12, 4, 22, 40)
        elif four:
            snares.note(at + 12, 1, 60, snare, 34)
        for row in range(2, BAR, 4):
            hats.note(at + row, 1, 60, tick, 22)
        for row in range(0, BAR, 4):
            hats.note(at + row, 1, 60, tick, 10)

    def low(at, name, vol=32):
        """The bass: its root in eighths, three, three and two, the
        last of each group an octave up."""
        root = pitch_of(BASS[name])
        for row, up in ((0, 0), (2, 0), (4, 12), (6, 0), (8, 0), (10, 12), (12, 0), (14, 12)):
            play(lows, at + row, 2, root + up, bass, vol, short=0.5)

    def stabbed(at, name, vol=26):
        for row in (0, 3, 6, 8, 11, 14):
            chord(at + row, 2, name, vol)

    def rolling(at, name, vol=20):
        """Sixteenths up and down the chord, under the lead."""
        root, kind = CHORDS[name]
        base = pitch_of(f"{root}4")
        third, fifth = base + (kind >> 4), base + (kind & 0xF)
        for i, pitch in enumerate([base, third, fifth, base + 12, fifth, third] * 3):
            if i < BAR:
                play(plucks, at + i, 1, pitch, pluck, vol, short=0.25)

    # The introduction. One voice, then its echo, then the bass and
    # the hats, then the roll; the level rises all the way.
    rising = (climb("A3 C4 E4") + climb("D4 F4 A4") + climb("E4 G#4 B4") + climb("G#4 B4 D5 E5"))
    for i, pitch in enumerate(rising):
        vol = 12 + (60 - 12) * i / (len(rising) - 1)
        echoed(INTRO + i, 1, pitch, pluck, vol, short=0.25, behind=i >= BAR)
    for bar in (2, 3):
        at = INTRO + bar * BAR
        for row in range(0, BAR, 2):
            play(lows, at + row, 2, pitch_of("E3"), bass, 18 + 2 * bar + row, short=0.5)
            hats.note(at + row, 1, 60, tick, 10 + 4 * bar)
    kicks.note(INTRO + 3 * BAR, 1, 60, drum, 40)
    kicks.note(INTRO + 3 * BAR + 8, 1, 60, drum, 44)
    roll(INTRO + 3 * BAR + 8, 8, 14, 44)

    # A, B and A again.
    for start, chords, lines, under in ((A1, PART_A, LEAD_A + [CLOSE], stabbed),
                                        (B, PART_B, LEAD_B, stabbed),
                                        (A2, PART_A, LEAD_A + [PICKUP], rolling)):
        for bar, name in enumerate(chords):
            at = start + bar * BAR
            low(at, name)
            under(at, name)
            # A fill before each part's end, but B's, which has its
            # own, and the last, which runs into A's first bar.
            drums(at, fill=bar == 7 and start == A1, four=not (bar == 7 and start == B))
            if bar < len(lines):
                phrase(at, lines[bar])
        if under is rolling:
            # The stabs stay, once a bar, to mark the chord.
            for bar, name in enumerate(chords):
                chord(start + bar * BAR, 3, name, 24, octave=5)

    # B's last bar: the introduction's climb again, over the dominant,
    # and its roll.
    at = B + 7 * BAR
    for i, pitch in enumerate(climb("E5 G#5 B5")):
        echoed(at + i, 1, pitch, lead, 40 + 14 * i / 15, short=0.25)
    roll(at + 8, 8, 16, 42)

    m.length(END)
    assert m.rows == END, m.rows
    m.loop_from(A1)
    return m
