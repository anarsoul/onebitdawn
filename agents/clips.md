# Clip tooling reference

The pack maker that makes clips and packs, the beeper converter and
the build-time variables of the sound path, with the settings the
default pack's clips were made with and who chose them by ear. The
players are in `agents/modules.md`; the buzzer stage is in
`agents/hardware.md`.

## The pack maker (`tools/pack`, `onebitdawn-pack`)

```
cargo pack -o packs/mine.bin [--sound-output buzzer|speaker] file[:option,…][=Label] …
cargo pack --clip clips/tune.xmc tune.xm        # one file's clip and no pack
cargo pack --help                               # every kind's options
cargo pack-test                                 # its tests
cargo pack-wasm && cp target/wasm32-unknown-unknown/tool/onebitdawn_pack.wasm docs/pack/
(cd docs && python3 -m http.server)             # the page, at /pack/
```

One library, twice: the command (`src/main.rs`) and, as WebAssembly
with a C ABI written by hand (`src/wasm.rs` over `src/web.rs`; no
binding generator, no npm), the page `docs/pack/index.html`, which the
release workflow deploys beside the installer with the `.wasm` it
builds (the file is ignored by git). The page previews a clip through
`core`'s players (`web::Session::preview`) and gives the pack as a
download; it does not post to a clock, which `http::admit` would
refuse from a foreign `Origin`.

A clip is made for a firmware, `Firmware` (`convert_for`; `convert`
is the latest's; the command's `--firmware 0.5|0.6`; `convert_for`
takes a `Target`): for `V0_5` a module's 16-bit
samples are cut to 8 and a sound file is IMA ADPCM, an option that
asks for 16 bits (`samples=16|packed`, `format=packed|pcm16`) is
refused, and so is a clip made already of a version v0.5 does not
play (`Firmware::plays`, by the magic). The page starts at Speaker (the user, 2026-11-24: who comes to it
more likely has one), the command at the buzzer. The page asks which firmware, "v0.5"
or "v0.6 or later" (the default), and `obd_convert` takes the minor
version. A new version of a clip is a new `Firmware` and an option of the
page's box.

It was ported from the Python converters the clips of 2026 were made
with (`scripts/xm2clip.py`, `mod2xm.py`, `s3m2xm.py`, `pt32clip.py`,
`sid2clip.py`, `adpcm.py`, `mkpack.py`, gone 2026-11-19; they are in
the history), byte for byte: its tests hold it to `clips/` and to
`tools/pack/tests/data/`, the Python's output for three small files,
to the Python 6502's state after four thousand random programs and to
the Python ADPCM encoder's codes. Keep them passing: a clip a clock
has must come out the same.

- A file's kind is by its bytes, then its extension (`kind.rs`); a
  clip made already goes in as it is. A clip's label is the tune's
  title where its file has one (XM, MOD, S3M, SID, PT3; `title` and
  `natural` in `lib.rs`), cleaned to what a page shows, and its name
  the title's letters; with no title, or one that makes no name, the
  file's name as `ALARM_CLIP`'s. `name=` and `=Label` give others
  (the user asked for the titles, 2026-11-19).
- `--sound-output` chooses each kind's defaults, `Shape::new` in its
  module: `buzzer` is what the default pack's clips were made with,
  `speaker` (or `line`) what `scripts/speaker-clips.sh` made, so that
  script is now the default pack's sources through it.
- Every pack is read back by `core::pack::Pack::parse` before it is
  given out, and the limits are `core`'s constants.

### Sound files (`wav.rs`, `adpcm.rs`)

- The steps are those of the sox line the PCM clips were made with
  (`scripts/wav2pcm.sh`, gone with the rest): mono, the cut in time,
  the high-pass, the new rate (a Kaiser-windowed sinc), the peak to
  -0.5 dB, the drive into a limiter with no memory (sox's `gain -l`,
  `g x / (1 + (g - 1)|x|)`, measured against sox to seven places), the
  peak again, a fade of 1.5 s if it was cut. Not sox's bytes: against
  sox's output for the same file the difference is 28 to 34 dB under
  the signal at 8 bits (2026-11-19), dither and the resampler's band
  edge. Not yet heard on a clock.
- The buzzer's defaults are 900 Hz twice and 20 dB of drive; a
  speaker's 100 Hz once and 10 dB (`CLIP_HIGHPASS=100 CLIP_DRIVE=10`
  as it was).
- 16 bits predicted by default (`ADP2`, `format=packed`: a byte a
  sample and a sixteenth more, 17 KB a second at 16 kHz, with a table
  of predictors fitted to the sound; `adpcm::predicted`), since
  2026-11-22. `format=pcm16` is the 16 bits as they are (`PCM2`),
  `format=adpcm` IMA ADPCM (`ADP1`, four bits a sample, 8 KB a
  second, encoded with the four best paths kept) and `format=pcm` 8
  bits with dither, behind a `PCM1` header, or bare at 8 kHz. The
  first two are clips a firmware up to v0.5 refuses with its pack.
  Measured on the host (`PLAN.md`): on sixteen recordings, signal
  over the coding's noise is 49 to 82 dB predicted and 30 to 44 at 8
  bits; one table a clip, a table a segment of it gained nothing. The
  user's ear on white-clock, 2026-11-16, of ADPCM against PCM: "not
  bad", a difference heard, "way better than PCM 8k", which is the
  same flash; and of the clock's own decoder on a speaker: "audible,
  but acceptable". Nobody has heard a predicted clip on a clock.

### Tracker modules (`xmc.rs`, `modxm.rs`, `s3mxm.rs`)

- A FastTracker 2 module for the clock's own player, of up to twenty
  channels; the notes list what the module uses that the player
  lacks. An effect R (retrigger with a volume change) that changes
  no volume is written as E9, which the player has: x of 8, or of 0
  on a channel with no other R to remember a volume from, and y not
  0. It is the one place a pattern's bytes are not the module's. A ProTracker module (31 samples, any channel count the tag
  names) becomes an XM first: the pitch a PAL Amiga's, the slides
  (1, 2, 3, 5, E1, E2) converted from periods to the linear table at
  the period each is played at, E0 (the filter) left out and named. A
  ScreamTracker 3 module the same: samples, notes, volumes and the
  effects A, B, C, D, O and T, any other named; no panning.
- A module's 16-bit samples are kept predicted (`XMC4`: 8 bits a
  sample and a sixteenth more, behind a table of predictors fitted
  to each sample; `xmc::predicted`), the default since 2026-11-22.
  `samples=16` keeps them as they are (`XMC3`, twice the bytes, each
  sample at an even offset in the clip and the clip at an even
  offset in a pack, a byte of 0 before either where it takes one: the
  clock reads them as 16-bit words; a sample too short to repay its
  predictors is kept so in an `XMC3` clip whatever was asked) and
  `samples=8` cuts them to 8, rounded, which is the clip there always
  was (`XMC2`). The two tunes made here, their mix before the
  limiter against the mix of the 16 bits: 67 and 69 dB predicted, 44
  and 47 at 8 bits (`PLAN.md`). A predicted sample has seek points, the
  decoder's state every 256 samples (2 % more), so that a note a 9xx
  starts far into it does not decode all that is before it, and a
  channel playing on with no level is not decoded until it is heard
  (`xm::pass`); every one since 2026-11-23, only an instrument
  played with 9xx before. The
  first two are clips a firmware up to v0.5 refuses with its pack.
  A sample too short for its table to pay for itself (a wave of one
  period: 64 samples are 128 bytes at 16 bits and 166 predicted) is
  kept at 16 bits, the smaller and exact.
  A sample of 16 bits whose low bytes are all the same is one of 8
  and is written so, that byte dropped, whatever is asked: Keygen's
  lead is one (its low bytes nought), and its clip is what it was.
- The bass cut, an instrument's octave (`raise=`) and the limiter are
  the clip's. `knee` is the level the limiter starts at, in 1024s of
  the mix where one voice is 32. The default is 20 dB of drive, for
  the buzzer; a knee above the tune's peak is the limiter off.
  `knee=auto` is the lowest knee
  that has the limiter flatten no more than `tops` samples a minute
  at full scale, 60 unless given (`xmc::fit_knee`, `xmc::TOPS`): the
  tune is played on the host with the limiter off, the clip's own
  release decides which samples would be flat, and the knee goes over
  all but the allowed of them, as the drive has them: a `drive` over
  127 has samples under the knee at full scale too, and where the knee
  would pass 255 the note says the budget is not kept. It gives the two piano tunes the knees
  the ear chose (46 and 14 against 48 and 12); the other modules it
  leaves 5 to 7 dB quieter than the speaker's default, whose
  flattening is clicks too, heard on headphones and covered by the
  tune's level (the user, 2026-11-24); so it is not the default. A
  speaker's `Limiter` is the same for every module that gives no knee
  (`Target`'s third part beside the output and the firmware; the
  command's `--limiter`, the page's box beside Speaker, the wasm's
  place in `Limiter::ALL`): `loudest` the knee as it is,
  `loud` 1650 a minute (the user's ear, 2026-11-24), `cleaner` 600,
  `cleanest` 60, the default of the command and the page (the user,
  2026-11-24), so `--sound-output speaker` alone no longer makes what
  the speaker's pack has: `scripts/speaker-clips.sh` says `loudest`. A buzzer's clips are not touched (`PLAN.md`).
  A PT3 module has the same, option and `Limiter` (`pt3c::fit_knee`):
  its knee is one of fifteen steps, so the steps are tried from the
  highest down, and a fitted knee comes with the player's own drive
  where the speaker's default is `knee=37,drive=150`, unless the
  options give a drive. A SID tune has no knee and no such option.
- The rate: the clip is at `clip::FAST_HZ`, 31 250 samples a second,
  here and in the two kinds below, which the chip's converter at 256
  bits a sample sends at 8 Mbit/s exactly. It is the default since
  2026-11-13, heard on the breadboard stage and on white-clock. 32 kHz
  is not among the converter's whole dividers, and at
  `CLIP_PDM_BITS=384` such a clip is refused. `rate=16000` is the rate
  before it, half a module's share of the processor, and 8000 a
  module's third.
- A speaker's defaults are a 100 Hz high-pass and the limiter's
  release at 128 ms (`release=10`); heard and kept by the user
  2026-11-15, after a pack with the limiter off was too quiet.

### SID tunes (`sidc.rs`)

- A Commodore 64 tune (PSID) for the clock's own SID. It runs the
  tune's 6502 code on the host and keeps each frame's register
  writes, to the point where the tune loops (the machine's state is
  hashed after every call), and says what the tune does that the
  clock will not play.
- `mute=` keeps the voices' gates closed for the first frames, for a
  routine that clicks as it starts. `system=` plays it as the other
  machine would, whatever the file says: a PAL tune on an NTSC C64 is
  59.83 calls a second for 50.12, the pitch 3.8 % up.
- `flat`, for a speaker: no limiter, `drive` then a plain gain with
  the three voices at their top as full scale (flags bit 1; a firmware
  before 2026-11-15 plays such a clip limited).
- The record is written packed (`SID2`: writes and waits the tune has
  had before are references to them, `packrefs.rs`, a fifth of the
  bytes for a long tune); `plain` for `SID1`.
- A speaker's defaults are `flat` and the high-pass at 150 Hz twice
  (`highpass=150,passes=2`): the user's ear on white-clock's speaker,
  2026-11-19. A module's (the limiter with `release=10`, 100 Hz once)
  were tried first and heard as clipping, then flat with 100 Hz once
  as "almost good, but a bit bass-heavy". `limiter` asks for the
  limiter all the same.

### PT3 modules (`pt3c.rs`)

- A ZX Spectrum tune (a Pro Tracker 3 or Vortex Tracker II module)
  for the clock's own player and AY-3-8910. The clip is the module as
  it is behind a header with the machine's clock and frame rate (by
  the module's tone table unless told), the length and where it goes
  round. `restart` is for a module whose loop is only its ending.
- The defaults are the default pack's Synchronization: the limiter,
  the noise at 16, a 100 Hz high-pass once, `release=10`, since
  2026-11-16 (the user, after hearing the two on a speaker: "beeper
  should not be flat for sure"). Not heard on a buzzer yet. A
  speaker's add a gentler limiter, `knee=37` and `drive=150` (the
  user's ear, 2026-11-16, on test-clock's speaker; needs that day's
  firmware, an older one limits hard).
- `flat` is the chip as it is but for the high-pass, at a plain gain
  3 dB over three channels at full scale (`drive` 180): the user's
  choice of 2026-11-13, on headphones and on white-clock, over the
  limiter, and the converter's default until the pack's clip changed.
- `knee` is kept in the flags' top four bits as one of fifteen steps
  2 dB apart (`core::pt3::KNEES`).

## Modules made here (`scripts/mkxm.py`)

```
scripts/mkxm.py mods/src/morning.py mods/grieg-morning-bright.xm bright
scripts/mkxm.py mods/src/mountain_king.py mods/grieg-mountain-king-chip.xm chip
```

- Writes an XM from a tune written down as Python (`mods/src/`): a
  `tune()` that makes a `Module`, its instruments from recorded notes
  and its notes on a grid of rows (`Voices.phrase` takes them as
  `B5/2 G#5/2 …`, `read_midi` gives a MIDI file's). Needs numpy and
  scipy. Only what `xm.rs` plays is written.
- `xm.rs` starts a note with no ramp, so a note on a channel that is
  still sounding is a click. `Voices`, a set of channels, gives each
  note one that has gone quiet (a held note's release and a struck
  one's ring counted) and the script says how many notes it had to
  cut all the same. Give a part channels enough.
- The recorded notes are VSCO 2 Community Edition's (CC0), fetched by
  name at a pinned commit into the ignored `xm/vsco/` and cut down:
  mono, 8 bits, a new rate, a crossfaded loop for a held note. The
  pitch is measured, and a file more than 60 cents from the note it
  is said to hold is refused.
- A tune may have kits of instruments, named by a third argument:
  the two Grieg tunes have `bright` (the default) and `chip`. Besides
  a VSCO file a sample can be one of a SoundFont's by its name
  (`Sample.soundfont`), a period of a wave made there (`Sample.wave`
  over `pulse`, `saw`, `triangle`), a wave struck and dying
  (`Sample.pluck`) or noise (`Sample.noise`); `bright=` lifts what is
  over 2 kHz.
- It reports a note that found no channel; there should be none.
- A piano is a struck string: `Sample(struck=)` takes the
  recording's own fall out of the sample and into the instrument's
  volume envelope, which has no sustain point and goes on down after
  the loop's start as the recording does, and `Instrument(damp=)`
  makes a key-off the damper, the fade. `read_midi(path, pedal=True)`
  gives a note let go under the sustain pedal its length to where the
  pedal comes up, and with `grid=` puts a played file's notes on
  the grid. A recording may be a file anywhere by its URL (kept in
  `xm/` under its host's name), a WAV or an AIFF file, one `channel`
  of it.
- "Suspended Seconds" (`mods/src/suspended_seconds.py`,
  `mods/suspended-seconds.xm`, `clips/suspended_seconds.xmc`) is the
  user's own piano tune from his MIDI file, velocities and pedal
  kept, quantized to eighths, the default pack's sixth clip. Its kit
  is `upright` (VSCO's upright at mf; the user's ear, 2026-11-19),
  its samples of 16 bits since 2026-11-22 and kept predicted, so the
  clip is one for a firmware after v0.5;
  the tune's file has six others that were tried (VSCO's at pp, the
  University of Iowa's Steinway B, lifted ones). A piano's attacks
  click in the limiter, so its clip is made with
  `knee=12,release=10` for the buzzer and `knee=12,release=13` for a
  speaker (the user's ear again), a limiter that lets go over a
  second and rides a passage's level. The tune is the user's and not
  under the project's licence: it may be copied and distributed,
  unmodified or as converted from its module by onebitdawn's tools,
  as part of onebitdawn clip packs, including commercially; all other
  rights reserved (`clips/README.md`).
- "Starlight & Aurora" (`mods/src/starlight_aurora.py`,
  `mods/starlight-aurora.xm`) is the user's second tune, four parts
  from his MIDI file as they are there. It is the speaker's pack's
  ninth clip and in no other: on a buzzer it does not sound (the
  user, 2026-11-20), so `clips/` has no clip of it and
  `scripts/speaker-clips.sh` makes it from the module. Its label is
  the module's title, "Starlight and Aurora" (a label may not have
  `&`). Its piano is Surge XT's "Acoustic Grand", rendered a note at
  a time by `scripts/surgenotes.py` (needs the plugin and
  `pedalboard`) into the ignored `xm/surge/acoustic-grand/`, which
  the tune's script reads; the bell (a guess, by measurement, at a
  synthesizer that is not here) and the two sines are made in the
  script, the bass with harmonics. Its limiter is
  `knee=64,release=13`, which is the limiter off: at the other tune's
  knee of 12 the four parts are always over the knee and each new peak
  is flattened at full scale, heard as clicks on a speaker. Its samples are of 16 bits,
  kept predicted (a clip for a firmware after v0.5). The hiss under its
  quiet piano on a speaker in the buzzer's place is the transistor's
  stage and not the clip: the line output has none (`PLAN.md`). The terms are those of "Suspended Seconds".
- All their samples are of 16 bits since 2026-11-22, the recorded
  ones and the waves made in the scripts alike
  (`Sample.noise` and `Sample.pluck` take `bits=` for that); the
  mixes before the limiter against the 16 bits' are 65 dB (Morning,
  `bright`) and 64 (the Mountain King, `bright`) predicted, 46 and 41
  at 8 bits. Morning's `chip` kit is seven waves of one period, kept
  at 16 bits and exact.
- "Serial Dawn" (`mods/src/serial_dawn.py`, `mods/serial-dawn.xm`)
  is a tune in the key generators' manner, written here, of waves and
  noise made in its script: pulses put about their mean, a swept sine
  for the kick. It is the default pack's eighth clip
  (`clips/serial_dawn.xmc`, the pack maker's defaults; a speaker's
  defaults in the speaker's pack), and the project's, under its
  licence. Played round it starts over after its introduction:
  `Module.loop_from(row)` is the XM's restart position, which the
  alarm follows. Its chords are effect 0 on one channel, written into
  the cells by the script, since `Line` has no call for it.
- "Lera" (`mods/nq-lera.xm`, `clips/lera.xmc`) is n1k-o's "lerka",
  an XM of 18 channels as it was found, the default pack's third
  clip by his permission (`clips/README.md`), at the pack maker's
  defaults in both packs. Its rolls are R02, written as E92.
- The other two there are Grieg's Morning (shortened; read off a scan) and
  In the Hall of the Mountain King (Mutopia's MIDI file). Morning in
  both kits and the Mountain King in `bright` are in the default
  pack; `clips/README.md` has the commands and the sources.

## Beeper (`scripts/zxbeeper.py`)

```
~/.local/pipx/venvs/skoolkit/bin/python scripts/zxbeeper.py game.tap tap/tune.bpr --seconds 60 [--stretch 4] [--settle 20 --start 64000]
scripts/zxbeeper.py old.bpr new.bpr
```

- The first is what a ZX Spectrum program plays on its beeper, as a
  clip. It needs SkoolKit; `--start` is for a loader that waits for a
  key. The clip is written packed (`BPR3`, about half the bytes, the
  same runs); `--plain` for `BPR2`.
- The second packs a beeper clip again, or with `--plain` unpacks it;
  no SkoolKit.

## Packs

```
CLIP_PACK_FILE=packs/mine.bin cargo host-test the_pack_file_parses -- --nocapture
API_TOKEN='…' scripts/clips.sh <ip> [pack]
espflash write-bin 0x2e0000 packs/mine.bin && espflash reset
scripts/speaker-clips.sh
```

- `the_pack_file_parses` puts a pack through the parser the clock
  reads it with, as the pack maker does itself.
- `scripts/clips.sh` posts the pack to `/clips` (the default one, made
  of `clips/`, if none is named) and the clock restarts; with no
  arguments it just makes `packs/clips-default.bin`.
- `espflash write-bin` is the pack over USB.
- `scripts/speaker-clips.sh` makes the default pack for a clock with a
  speaker in the buzzer's place (`speaker-mod/`),
  `packs/clips-default-speaker.bin` (or the path it is given), under
  the default pack's names. The CI makes and checks it, and a release
  has it as `onebitdawn-clips-speaker-<tag>.bin`. With `amplifier`
  after the path it makes the pack for a speaker on an amplifier at
  the line output, `packs/clips-default-amplifier.bin`: the same with
  `--limiter cleanest`, which moves the knee of the five modules that
  give none (the two piano tunes keep theirs) and Synchronization's,
  with its drive, and nothing else; a
  release's `onebitdawn-clips-amplifier-<tag>.bin`.

## Build variables

```
ALARM_CLIP=clips/keygen.xmc,wav/tune.pcm cargo build --release
CLIP_PDM_BITS=128 cargo build --release
```

- `ALARM_CLIP` is the older way, clips built into the firmware, for a
  clock whose partition table has no `clips` partition; a pack takes
  their place where there is one. Any kind (PCM, ADPCM, beeper,
  tracker module, SID tune or PT3 module), as the last melodies, named
  after their files; `ALARM_CLIP_LABEL='A,B'` for other labels. A
  ringing alarm loops a beeper clip, a module, a SID tune or a PT3
  module with no gap and repeats PCM after a pause, unless the file is
  given as `file:once` or `file:loop`.
- `CLIP_PDM_BITS=128` is the chip's PDM converter, which every PCM
  clip, module and tune goes through (the software modulators a build
  could choose, `CLIP_MODULATOR`, `CLIP_KBPS` and `CLIP_CARRIER_KHZ`,
  were taken out on 2026-11-24), at 128 bits a sample, 2.048
  Mbit/s for a 16 kHz clip, as the clocks' buzzers have heard it. 256
  (4.096 Mbit/s) is the default and 384 the third. More bits are less
  of the converter's own noise and shorter pulses for the transistor.
  The variable is the line output's: a clip on the buzzer's pin gets
  128 at most (`clip::BUZZER_CONVERTER_BITS`, `agents/hardware.md`).
