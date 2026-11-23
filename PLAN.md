# onebitdawn — plan

What is not done: the list for the first public release, the work
planned and not built, the checks a built change still owes, and the
open findings of the reviews. A new piece of work is designed here
before it is started, with what was measured and learned added as it
goes; when it is built and has had its check, its section moves to the
end of `HISTORY.md` as it stands, and what it still owes stays here as
a line. `HISTORY.md` is the record of every milestone so far, the
original design included; `ARCHITECTURE.md` is the firmware as it is.

## To do before the first public release — a list, 2026-11-17

The user's, and what the readiness check of the same day left open.

- [x] Screenshots of the web interface (the user's): six in
  `screenshots/`, taken from white-clock with headless Chromium at
  420 px and twice the scale, the clock page light and dark and the
  others dark, the network's name and the broker blurred in the page
  before the capture (the user's choice; the version line reads
  `v0.4-67-gfb0e18f`). Three in the README under the feature list,
  all six in the manual beside what they show; the manual said four
  pages and has `/clips` in its list now.
  Retaken 2026-11-20 from white-clock on `v0.5-rc3` (the user: its
  pages are master's), the same way, with the broker's user blurred
  as well; the settings page has the sound output now.
- [x] A picture of the board with the buzzer mod, beside the README's
  description of it (the user's; a photograph, so the user's to take).
  Two, taken by the user 2026-11-17, in `pics/`: the component side
  (RBE beside Q1, R5, the BAT42 under the adapter) and the solder side
  (the flyback diode across the buzzer), under the mod's four changes
  in the README with a caption that says where each part is.
  Retaken by the user the same day without RBE and the BAT42, which
  the scope showed change nothing (`agents/hardware.md`, the buzzer's
  mod): the mod is two changes, the diode and R5.
- [ ] `CHANGELOG.md`: `[Unreleased]` made the version's section, the
  compare links under it.
- [x] An install of an image with no token from the web installer or
  the CI's artifact: the access point's setup, then a token set with
  the PIN. Done on test-clock 2026-11-17 with a local build of the
  CI's configuration written by espflash ("An install from nothing",
  below); the web installer itself is the last item's.
- [ ] The default pack heard on a stock buzzer circuit (no diode, R5
  at 10 kΩ), which is what a builder has; the README's wording on the
  mod by what is heard.
  Heard so far, 2026-11-17 (the user): the buzzer variant on a modded
  buzzer and the speaker variant on a speaker, both good; the stock
  circuit not tried.
- [x] The installer's list no longer offers v0.1 to v0.4 (the old
  partition table, the `esp32clock` token): `docs/index.html` leaves
  them out by tag (`OLD`); their files stay on `gh-pages` and their
  GitHub releases as they are. The page is deployed by the release
  workflow, so this is live with the next tag. Checked under Node
  against GitHub's list as it is today, which leaves nothing, the
  page's fallback to `latest/`.
- [x] A pack maker a user can use, one command and one web page
  (the user, 2026-11-19), before the project is announced: built and
  tried by the user on white-clock the same day ("A pack maker for
  users" in `HISTORY.md`). Its release builds run with the tag.
- [ ] The annotated tag; then the release's three files and one
  install from the live page, since the workflow has not run with the
  pack and the new names.

## Planned, not built

### Dim in the dark (M11b, second half) — planned 2026-11-01, not built

The rest of M11 in `HISTORY.md`: the sensors are read and the
temperature face is on the panel, and the light index is on the page
and in `/api`. The paragraph below is the design as M11 had it; the
record has no `dim_in_dark` field yet.

**Light on the brightness.** One boolean setting, `dim_in_dark`,
labelled "Dim in the dark" on the page (key `dim`). When it is on
and the room is dark, the panel runs at the lowest brightness step;
otherwise it runs at the brightness the user chose. Dark is the
10 s light average below `DARK_BELOW` and it ends when the average
rises above `DARK_ABOVE`, two constants in `config.rs` a comfortable
gap apart (a room light switching on has to clear the gap, so a
candle or a passing car does not flip it), with the first numbers
guessed and tuned on the panel from the heartbeat millivolts. The
user's brightness step is untouched by all this: the bottom short
press still cycles it, the record stores it, and the display task
substitutes the lowest step at draw time while dark is set. A user
who has chosen the lowest step sees no difference. The current step
`IS` still comes from the step actually drawn, cap and all.

### A station on a weak access point never looks for a better one — seen 2026-11-24, not designed

From the rc6 soak (its section below): the near access point went
away for a couple of minutes, the clock rejoined the house's far one
at −82 within 10 s, and stayed there when the near one came back at
−40. It left only because the far link got bad enough for the
self-heal's ten probes lost in a row, 19 minutes later; a link that
answers one probe in four would hold it for good, with MQTT dropping
and the pages mostly unreachable. A scan now and then while the RSSI
is poor, and a reconnect when a much stronger BSSID of the same
network is seen, is the obvious shape; nothing is designed. A
feature, so for `next`, not for master before v0.5.

### A line output on the adapter for an amplifier — an idea of 2026-11-15; the setting and the pin built 2026-11-18, not flashed, nothing heard or measured

The user's, of the speaker mod: with a proper amplifier in place of
Q1 as the output stage, the adapter can carry the filter, and the
amplifier's input is soldered to a pad on the adapter. The amplifier
then has the converter's bits from the pin and no saturating
transistor between. Everything below is reasoned, not measured.

**The design, the user's of 2026-11-15: a pin of its own and a
setting that chooses the output.** "That would be cleaner: you
cannot mess up your board by using wrong firmware version." The
line output is a second GPIO, push-pull, and GPIO5 stays what it is,
open-drain to Q1. One firmware and one release image serve both
kinds of clock, and a wrong setting is silence and never a buzzer
held on: `line` on a stock clock drives a pin nothing is soldered
to, `buzzer` on a clock with an amplifier plays into Q1 as it always
did.

- The pin: GPIO20, the user's yes of 2026-11-15. The firmware uses 0, 1 and 3 to
  9. GPIO10, the first thought, has the module's own LED on it on
  both the SuperMini and the Zero (the user, 2026-11-15), which the
  converter's bits would light. GPIO2 is a strapping pin that must
  read high at reset, and the filter's capacitor and divider would
  hold it low. GPIO21 is UART0's TX and carries the ROM's boot text
  into the amplifier at every reset. GPIO18 and 19 are the USB.
  GPIO20 is UART0's RX, which nothing uses (the console is the USB
  one): an input with a pull-up while the ROM runs, so the filter
  sees one step to 3.3 V at a reset and no data. Where it comes
  out on each adapter's layout is for the board's drawing.
- The parts, from the pin to the pad: two RC stages, 1 kΩ / 10 nF and
  then 10 kΩ / 1 nF (the second at ten times the impedance, so it
  does not load the first; −3 dB near 10 kHz, 8.2 nF / 820 pF if that
  is dull); a resistor to ground as a divider, or a trimmer's
  footprint, since a full-scale clip is about 1 V rms off a 3.3 V pin
  and a module of the PAM8302's kind (24 dB) clips near 0.1 V rms
  (about 1.2 kΩ under the 11 kΩ for a tenth; the amplifier's input
  impedance is in parallel with it, so the level is to be checked on
  the module); a series capacitor, about 1 µF (many modules have
  their own).
- The setting: `output`, `buzzer` (the default, and what every older
  record reads as) or `line`; a new record version, on `/settings`
  and in `/api`, not in `mqtt::SET_KEYS`. As first written it was
  read at boot and a change reset the clock. Built otherwise
  (2026-11-18, the user's "can it not be changed at run time?"): it
  is read at every tone and at every clip, since both build their
  pin's output anew each time anyway, so a change takes effect at the
  next sound and the two outputs can be compared on the bench without
  a reset between.
- On `line` the clip's I2S signal goes to that pin through the GPIO
  matrix without the inverter, and the beeps' LEDC to the same pin,
  both push-pull; GPIO5 stays released. The pin
  rests driven low, so the rest is the ground and the rail's noise
  is not in it, and the clip's ramps start from and return to the
  bottom rail as they do from the released line now. Whether 30 ms
  of ramp is a thump through the capacitor is to be heard.
- The thermometer's hold (`sensors::Hold`) is for Q1's heat and does
  not apply on `line`.
- Still true: 3V3 is the converter's reference while a clip plays,
  so the Wi-Fi's bursts on that rail are in the audio; if they are
  heard, a single-gate buffer from an RC-filtered supply is the fix,
  a footprint at most. The amplifier's shutdown pin on one more GPIO
  is a thought for the idle current, not a plan. (A plan since
  2026-11-18, for the Wi-Fi's clicks: "The amplifier shut down
  between sounds", at the end of this section.)

**A first try that needs no firmware: a pull-up on GPIO5.** For the
bench, before the setting exists, and not for the board. A 470 Ω
pull-up from the GPIO5 net to 3V3 (never to 5 V), then 10 kΩ /
680 pF twice (−3 dB near 9 kHz), about 2.2 kΩ to ground for a tenth,
the capacitor. The line stays open-drain, the beeps work as they
are, the released line is the top rail, which the ramps already
start from, and the audio is inverted, which nobody hears. The
filter charges through the pull-up and R1 and discharges through R1
alone: a second harmonic of about (Rp/R1)·a/2, some 1 % at full
scale; 7 mA while the pin is low. A high line turns Q1 on through
R5, so the buzzer must be off the board, as the speaker mod has it;
Q1 may stay.

**Built 2026-11-18** (record version 13, `settings::AudioOut`,
`buzzer::Pins`; the build, clippy and the host tests pass; not
flashed: the bench board is in its 24 h soak until 2026-11-19 12:09).

- `buzzer::Pins` holds GPIO5 and GPIO20 as `AnyPin`s and is the one
  place that knows each pin's drive and rest: the buzzer's open-drain
  and released, the line's push-pull and low. The buzzer task has it
  and the clip task a second handle, as with the one pin before.
- A tone: `settings::get().output` at its start. LEDC stays connected
  to the last tone's pin at the silent duty (100 % on the buzzer,
  0 % on the line), so when a tone goes to the other pin the task
  first takes the old one back to its rest, or both would sound.
- A clip: the setting at the moment the line is connected. The
  inverter only for the buzzer; `clip::converting`, which holds the
  thermometer, only for the buzzer.
- GPIO20 is not touched at boot unless the stored setting is `line`,
  in which case it is driven low before the tasks start. A clock
  switched to `line` later has it driven at its first sound: one step
  from the ROM's pull-up to the bottom rail, a thump to be heard once.
- The filter's values and the amplifier's level were worked out again
  from the datasheets for the CJMCU-832 and a PAM8403 module:
  `bench/pdm-amp.md`, with the drawings beside it. They replace the
  1 kΩ / 10 nF and 10 kΩ / 1 nF above, whose stages load each other
  more than was allowed for, and the "clips near 0.1 V rms", which is
  0.2 V rms (0.62 V peak to peak at 5 V).

Flashed to test-clock over USB the same day. The user's scope on the
filter's output, no amplifier yet (2026-11-18): "looks good", the
clips an audio wave, the beeps a square wave.

Heard the same day (the user, 2026-11-18), on a breadboard: the
CJMCU-832's filter of `bench/pdm-amp.md` (2.2 kΩ / 4.7 nF, 10 kΩ /
4.7 nF, 3.3 kΩ to ground) into a PAM8403 module with a 50 kΩ pot, the
PAM8302s not having arrived, and the same 8 Ω 0.25 W speaker as the
speaker mod has. "Subjectively the quality is noticeably better."
There is a slight background noise, which the user takes for pickup
from the Wi-Fi on the breadboard; where it comes in (the 3.3 V rail
through the pin, the ground between the boards, the amplifier's
supply) is not yet told apart. The PAM8302, whose input is
differential, is still to be heard.

An A/B the same evening (the user): the same clips, the output
switched at run time between the S8550's stage and the PAM8403, each
into its own speaker of the same kind, the loudness matched by the
amplifier's pot. The amplifier is "noticeably cleaner", "as if the
dynamic range it passes is wider". By ear, nothing measured. A reason
offered and not tested: the transistor is a non-linear element ahead
of any filter, so the converter's shaped noise folds into the audio
band there, where the RC filter removes it before the amplifier.

One more thing for the amplifier, the user's of 2026-11-18: its
trimmer sets the loudness, which the S8550's stage has no way to do.
A larger RS there would in theory, but with a PAM8302 as cheap as it
is that is pointless.

A click at Stop and none at Play (the user, 2026-11-18): a clip that
ends by itself ramps down, one cut short had its pin taken to the
bottom rail from the middle of the scale. Built the same evening: on
the line output a converter clip cut short ends on a ramp, one chunk
long, written two chunks ahead of the DMA from the sample before it
(`clip::fade`, under test), and the pin is taken when it has gone
out, so Stop is about 0.1 s later there at 16 kHz. The buzzer's cut
and a bit stream's are as they were. Heard by the user the same
evening on test-clock, the PAM8403 and the breadboard: no click at
Stop any more, on a quiet clip (PPK) either.

A third value, `split`, the user's of 2026-11-18: "note melodies on
buzzer, clips on speaker", for a clock that has the amplifier and
still has its buzzer on Q1. Built the same day (the build, clippy and
the host tests pass) and flashed to white-clock over the air; the
user, the same evening: "works as expected".

- `AudioOut::Split` is record byte 2, with no new record version:
  widening a field's values is what the record allows, and an image
  that knows two values reads the third as `buzzer` (the test that
  stood for "an output of a later image" now writes a 3).
- The setting no longer names a pin. `settings::SoundPin` does
  (`Buzzer`, `Line`), and the setting answers one for each kind of
  sound: `AudioOut::tones()` and `AudioOut::clips()`. `buzzer::Pins`,
  the tone and the clip take a `SoundPin`, so there is still no way
  to ask for GPIO5 push-pull.
- A tone reads `output.tones()`, a clip `output.clips()`; everything
  that hung on the pin follows by itself: the inverter and the
  thermometer's hold on the buzzer's pin only, the ramp at a cut on
  the line's only. LEDC stays on the buzzer's pin, silent and
  released, while a clip plays on the other, which is what it does
  during a clip on `buzzer` too.
- GPIO20 is driven low at boot when the stored setting's clips go
  there, `line` or `split`.
- On `/settings` the third option reads "Notes: buzzer, clips: amp"
  (first "Notes on the buzzer, clips on the amplifier", too long for
  the user, then "Buzzer + amplifier", "confusing"); `/api` takes
  `output=split`.

To check for `split`: a note melody's preview on the buzzer with
GPIO20 quiet, a clip's on GPIO20 with the buzzer silent, the alarm
ringing with each, and the thermometer not held during a converter
clip.

To check, on a board with the filter and the amplifier (or a scope on
the filter's output, `bench/pdm-amp.md`):

1. `output=buzzer`, a stock clock: tones and clips as before, and a
   scope on GPIO20 shows nothing.
2. `POST /api` with `output=line`, no reset: the next preview is on
   GPIO20 and the buzzer is silent, for a note melody and for a clip
   of each kind (a converter clip and a beeper clip).
3. Back to `buzzer` and to `line` again between previews, and in the
   middle of a note melody: never both at once, never a buzzer held
   on, GPIO20 low at rest.
4. The clip's start and end on the scope: the ramp from the bottom
   rail, and whether it is a thump in the speaker.
5. A clip on `line`: the thermometer does not hold (`sensors` in the
   heartbeat).
6. The record: an update from version 12 keeps every setting with the
   output at `buzzer`; a rollback from 13 to the release before keeps
   them too.

To try on the bench board, in this order: the pull-up and the filter
on a breadboard with the firmware as it is, the level at the
amplifier's input, the hiss with a silent clip and with Wi-Fi busy;
then the setting and the new pin, and the two side by side.

**The amplifier shut down between sounds — the user's of 2026-11-18,
the firmware built the same day, not flashed, nothing soldered or
heard.** "Ideally, amplifier needs to be shut down when
not used, since it picks up interference from WiFi (very quiet clicks
when WiFi is sending / receiving). PAM8302 should use shutdown pin,
e.g. GPIO10. LED on esp32c3 zero will be unusable, but it is a
sacrifice I am willing to make." So the shutdown pin, a thought for
the idle current above, is a plan for the clicks, and the idle current
comes with it. Everything below is reasoned; nothing is built, heard
or looked up in the datasheet again.

- The pin: GPIO10, the one free pin left that is neither a strapping
  pin, the USB nor UART0's TX (the list above). It was refused for the
  audio because the module's LED is on it; as a level that changes
  twice a sound it only costs the LED, which the firmware never used.
  On the Zero the LED is an addressable one and GPIO10 is its data
  input: whether one edge with no bit train behind it ever lights it
  is to be seen.
- The level: high is on, low is shut down, which is the PAM8302A's SD
  (active low) wired straight to the pin. Push-pull. The datasheet
  (Diodes, PAM8302A, read 2026-11-18): SD high from 1.2 V, low to
  0.4 V, with no word of the supply, so the pin's 3.3 V is enough at
  5 V; under 1 µA shut down; no pull on SD inside the chip is
  mentioned, so what a floating SD does is the module's resistor's, if
  it has one; SD is to rise 1 to 100 ms after the supply (10 ms
  typical), and "for the best power on/off pop performance" the
  amplifier is shut down before the supply comes or goes, which a
  pull-down on the adapter gives. No turn-on time from SD is given.
- Through a reset the pin is an input and floats, and the module's
  own pull-up on SD turns the amplifier on until the firmware is up.
  On Adafruit's board it is R3, 10 kΩ from SD to the amplifier's
  supply (its Eagle schematic, github.com/adafruit/
  Adafruit-PAM8302-Mono-Amplifier-PCB, read 2026-11-18; the copies are
  to be checked with a meter). With the amplifier at 5 V that is one
  more 5 V source on a GPIO (the user: "Ugh"): some 0.15 mA through
  the pad's clamp while the pin floats, 0.17 mA into the driver while
  it is high, and 0.5 mA through R3 all the time the amplifier is shut
  down, against the chip's 1 µA. A pull-down that wins against 10 kΩ
  would be under 870 Ω and cost 5 mA from the pin while the amplifier
  is on. So: R3 comes off the module (an 0805), and the adapter has a
  pull-down on GPIO10, 10 kΩ to 100 kΩ. Then nothing on SD is above
  3.3 V, the amplifier is off through the ROM and the boot, GPIO20's
  step to 3.3 V there with it, and it is shut down before its supply
  comes or goes, as the datasheet asks. A module with R3 left on and
  SD not wired plays as it does today. The amplifier on 3.3 V, with
  R3 left on, was the other way (2.0 to 5.5 V; some 0.55 W into 8 Ω at
  1 % THD, scaled from the datasheet's 0.65 W at 3.6 V): its current
  and the Wi-Fi's would share the module's regulator, and the rail's
  noise would be its supply. The user, 2026-11-18: stay on 5 V and
  take the pull-up off.
- No new setting. The pin is driven only when a sound goes to the
  line output (`output` at `line` or `split`), and at boot, low, when
  the stored setting is one of those, before GPIO20 is driven low, so
  that step is not amplified. On `buzzer` it is never touched. A clock
  with an amplifier and nothing on GPIO10 plays as it does now, its
  module's pull-up holding SD.
- `buzzer::Pins` gets the third pin and stays the one place that
  knows the pins: `amp(on)`. The buzzer task brackets a whole command
  with it, since it starts the tones and the clips alike
  (`clip::play` is called from it): on at the start of a `Preview` or
  a `Ring` whose sound goes to `SoundPin::Line`, a wait of `AMP_WAKE`
  for the amplifier to come up, the sound, and off when the command
  has ended, after the pin is back at its rest (a clip's ramp gone
  out, the tone's duty at 0 %). It stays on through the gaps between
  notes and between a ring's repeats, so a ring is one on and one off.
  A tone or a clip that finds the setting changed to the line under
  it turns the amplifier on itself, without the wait.
- `AMP_WAKE` is a guess until heard: the first note or the clip's
  ramp cut at its start means it is too short. The amplifier's own
  click at each turn-on and turn-off, if it has one, is what this
  trades the Wi-Fi's clicks for: to be heard before it is kept. The
  alarm starts later by the wait.
- The PAM8403 on the bench has its shutdown pin tied on the module;
  this is for the PAM8302s, when they arrive.
- When built, in the same change: the line output's bullet in
  `AGENTS.md`, "Line output" in `agents/hardware.md`, `Pins` in
  `agents/modules.md`, the pin in `bench/pdm-amp.md` and its drawings,
  the wiring in `MANUAL.md` and `speaker-mod/README.md`.

Built 2026-11-18 (the build, clippy and the host tests pass; not
flashed):

- `buzzer::Pins` has the third pin and `AMP_ON`, an atomic the two
  tasks write by turns: `amp_on` (and whether it had been off),
  `amp_off` (only if a sound turned it on, so a clock at `buzzer`
  never has the pin driven), `amp_shut` for the boot.
- The buzzer task: `amp_for` before a melody's first sound, on with
  `AMP_WAKE` (20 ms, the guess) if the melody's pin is the line, off
  if it is the buzzer; off at a `Stop` and at a preview's own end. A
  command that replaces another leaves the amplifier as it is, so one
  preview after another is not an off and an on.
- A tone and a clip that go to the line turn it on themselves if it
  is not, with no wait: for an `output` changed under a melody.
- `main.rs`: `amp_shut` and then GPIO20's rest, when the stored
  setting's clips go to the line.
- Written up: `bench/pdm-amp.md` ("The shutdown pin") and
  `pdm-pam8302.svg` with GPIO10, the pull-down (R4, 47 kΩ) and the
  pull-up taken off; `speaker-mod/README.md`'s section on the
  amplifier, which had fallen behind the line output; `MANUAL.md`,
  `ARCHITECTURE.md`, `AGENTS.md`, `agents/hardware.md` and
  `agents/modules.md`.

To check, on a PAM8302 with SD on GPIO10 (the pin alone can be seen
on a scope before the amplifiers arrive): the pin on the scope, high
from before a sound to after it and low otherwise, for a note melody,
a clip, a clip cut short and a ring with its gaps; low and untouched
on `buzzer`; the Wi-Fi's clicks gone at rest; the amplifier's own
click at the two edges; the start of a sound not cut; through a reset
with and without the pull-down; the Zero's LED.

**The adapters with the filter on them — drawn and ordered by the
user 2026-11-19, not built, nothing heard.** SuperMini v2 and Zero v4
(`adapter/`, `gerbers_clock_supermini_v2.zip` and
`gerbers_clock_zero_v4.zip`; the boards built so far are SuperMini v1
and Zero v3). Both carry the filter as heard on the breadboard, R4
2.2 kΩ and C1 4.7 nF, R5 10 kΩ and C2 2 nF, R6 3.3 kΩ to ground, from
GPIO20; R3, 10 kΩ from GPIO10 to ground, the shutdown's pull-down;
and J2, a five-pin header in the order of the PAM8302 board's own,
A+, A−, SD, 5 V, GND. R7 is a 0 Ω link between socket pins 16 and 14,
fitted only on an HU-058D, which has driver 1's clock on 14 (the
README's short wire). R4 is at the filter and not at the pin (the
user: the track before it is driven by the pin and picks up nothing;
after R4 it would be 2.2 kΩ beside the antenna). The boards are
6.5 mm (SuperMini) and 5.7 mm (Zero) wider than the old 40.5 mm, all
of it on the side away from the socket. Checked before the order:
KiCad's DRC and ERC with no errors, the two netlists the same but for
the module's footprint, and each archive's nine layers and two drill
files the same as its board file draws. With C2 at 2 nF the figures
for a PAM8302 board with its trimmer are a division of 0.152, −3 dB
at 15.7 kHz and −38 dB at 250 kHz. To check when they come: that each
fits its case, the filter's output on a scope, the amplifier on J2
with its pull-up off, and on a PAM8302 with no trimmer across its
input whether R6 has to be 2.7 kΩ (`bench/pdm-amp.md`).

### A watchdog — designed, built and run on test-clock 2026-11-17; a soak still owed

The backlog's "No watchdog" (second pass): `esp_hal::init` disables
every watchdog, so a hang that is not a panic leaves the panel dark, or
one row lit, until the power is cut, and a new image that hangs is
never rolled back.

What can hang, and what it takes with it:

| context | how | what else stops |
| --- | --- | --- |
| thread executor | a task spins without an `await` | every thread-mode task |
| thread executor | a task parked on an `await` that never completes | nothing else |
| interrupt executor (scan, clip) | a spin | all of thread mode |
| esp-radio's threads | a spin | the main thread; a block only stops Wi-Fi, which is `heal`'s |
| a critical section | waits for ever | everything, interrupts included |

Two layers.

**The RWDT** (RTC slow clock, independent of the main system),
enabled right after `esp_hal::init`: stage 0 `ResetCore` at
`watchdog::HARD_TIMEOUT`, 60 s, and `ResetSystem` one stage later.
`ResetCore` leaves the RTC domain alone, so the carry record should
survive (esp-hal zeroes persistent RAM on power-on only); a bench
check. 60 s and not a few seconds (the user, 2026-11-17): a clock in
a reset loop still has time to join the network and say something
each boot. It also means the boot needs no feeding by hand. The cost
is a row latched at full current for up to a minute in the worst hang.

It is fed from one place, `watchdog_task` on the thread executor, once
a second. That is the lowest context there is, so a feed proves the
interrupt executor, the radio's threads and the critical sections all
gave the CPU back. Never feed from an interrupt or the interrupt
executor.

**Check-ins**, for the task that waits for ever while the rest runs.
The feeder feeds only if every party that is expected has checked in
within `watchdog::DEADLINE`, 10 s:

- `display_task`, every pass of its loop (its wait is bounded at about
  a second);
- `clip_task`, only while a clip plays: expected from the start of a
  clip to its end, which is also the interrupt executor's sign of life.

Not watched: the tasks that wait on an event by design (buttons,
buzzer, the listeners, persist), the scan ring's position (decided
against, 2026-11-17), and the network, which is `heal`'s: a router
that is off must not become a reset loop.

On a missed check-in the feeder stops feeding, writes who it was into
the carry record, then tries `scan::halt`, `clock::carry_time` and a
software reset; if that hangs too the RWDT ends it within the minute.
No wait for a ringing alarm and no MQTT goodbye.

The parties, the deadline and the verdict are `core/src/watchdog.rs`,
under test; the RWDT and the task are `src/watchdog.rs`.

Across the reset:

- `carry::init` keeps the record on the RTC-watchdog reset reasons as
  on the software ones.
- One word appended to the carry record (a new layout, the low byte of
  `MARK`): watchdog resets since power-up and the last culprit (a
  party, or none for the hardware's own reset). In the boot log, the
  heartbeat and `/api` (a new status key); not on a page, not in MQTT.
- `ota_boots` survives a watchdog reset with the record, so an image
  that hangs is rolled back. `carry::init` and `ota::init` move ahead
  of `pack::init` and the settings load, or an image that hangs there
  is never counted. (`clock::restore_carried` before `ota::init` is
  the fourth pass's "stale time" item; same move.)
- After the hardware's reset the carried time is the last the display
  wrote, up to `HARD_TIMEOUT` old, and is shown as it is until SNTP:
  within a minute of right is good enough (the user, 2026-11-17).
- The heartbeat gains the longest gap between two feeds, so the
  figures above can be judged from a soak.

Decided against (the user, 2026-11-17): stage 0 as an interrupt that
stores the interrupted program counter in the carry record, so the
boot log would name where a spin was.

Built 2026-11-17 as designed: `core/src/watchdog.rs` (six tests) and
`src/watchdog.rs`; the carry record is layout 5 with the word
appended, read only under a mark that has it, since an older image
leaves the tenth word stale. `ota::init` runs before the settings
are loaded now, so its boot line no longer says whether a token is
set (the `boot:` line does). Builds, clippy clean, 179 host tests.

Not known until it is run: whether RTC RAM is as it was after the
`ResetCore`; whether espflash still flashes and its monitor still
resets a clock whose RTC watchdog is running (the timer is in the RTC
domain and outlives the USB reset; esptool turns it off over
USB-JTAG for that reason).

Run on test-clock 2026-11-17, with a build of the `bench-hang`
feature (`watchdog::bench`: `POST /hang/<kind>` with the token, and
`BENCH_HANG=boot` for an image that spins in `main` after `ota::init`),
the serial log read beside `/api`:

| hang | ended by | back, after the request | boot line |
| --- | --- | --- | --- |
| `thread`, a spin in a thread-mode task | the timer | 70 s | `CoreRtcWdt`, record kept, "by the hardware timer" |
| `irq`, a spin on the interrupt executor | the timer | 70 s | the same |
| `cs`, a spin in a critical section | the timer | 70 s | the same |
| `display`, the task parked | the feeder, 9.5 s after the park | 20 s | `CoreSw`, "by the display task" |
| `clip`, the task parked in a preview | the feeder, 10.2 s after | 20 s | `CoreSw`, "by the clip task" |

- RTC RAM is as it was after the `ResetCore` (the ROM prints it as
  `rst:0x9 (RTCWDT_SYS_RST)`, esp-hal reads `CoreRtcWdt`): the count,
  the time and `ota_boots` all came across, the time a minute old.
- The `BENCH_HANG=boot` image sent to `/ota`: boot 1 of 2 and boot 2
  of 2 each spun and were reset by the timer with `ota_boots` going
  1, 2, 3, the third boot switched back, and the clock answered from
  `ota_0` 129 s after the upload, the count up by two. The time it
  showed then was two minutes old until SNTP.
- espflash 4.7.0 flashes a clock whose timer is running (10 s, twice);
  the USB reset discards the record, so the count starts at 0.
- Seen by the way: with the board left in the ROM's download mode
  ("waiting for download", after espflash's monitor was attached with
  `--no-reset` to a freshly flashed board), the timer the firmware had
  started fired a minute later and booted the firmware. The RTC
  watchdog outlives a reset into the ROM; nothing here depends on it.
- The heartbeat after a boot with a Wi-Fi join and an MQTT session:
  longest feed gap 1261 ms. No clip, save or upload was in that
  minute.
- The plain build answers `POST /hang/thread` with a 404.

**A review's finding, fixed 2026-11-17: a clip that hangs the clip
task made a reset loop during the alarm.** The ring plays the clip,
the clip task misses its check-in, the feeder resets, the grace rings
the missed alarm with the same clip, and so on until the grace runs
out: a panel that flashes and no sound. Now a reset over the clip
task sets `Trips::clips_suspect` (bit 23 of the watchdog's word, the
count 23 bits), which later resets of any cause keep, and while it is
set the buzzer picks with `melody::pick_notes`: a clip's melody is the
beeps, `random` draws among the note melodies. The review asked for
it to clear "once a clip has played to its end or at power-up"; with
every clip replaced by notes none can play to its end, so it is
cleared by what can have mended the clip, a firmware update or a clip
pack upload (`watchdog::trust_clips`, before their reboot), and by a
power cut with the record. Two host tests (which causes set it and
what keeps and clears it; the notes-only pick with and without a
pack), 181 in all.

Run on test-clock the same day, bench-hang build, melody `keygen` (a
clip), the alarm set a minute ahead and `POST /hang/clip` before it:
`ring keygen`, the clip task parked, the feeder's reset 10.3 s later,
and at 0.6 s of the next boot "keygen is a clip and the clip task hung
in one" and `ring beeps`; no second reset, still ringing 90 s on, the
heartbeat with `clips not played`. The plain build sent to `/ota`
logged "clips to be played again" and a preview of `keygen` played the
clip. Not run: a preview while under suspicion (the ring was still on
and a preview is ignored then; it is the same `resolve`), a pack
upload's clearing. The second ring was heard as beeps by the user
(test-clock has a speaker connected).

**A second review's addition, 2026-11-17: the pause is shown.** A user
whose alarm beeps, and whose Play on the chosen clip beeps too, would
take the pack or the firmware for broken. `form::Status::clips_paused`
(from `watchdog::clips_suspect`): a line on the clock page under the
next alarm and above the alarm sound, armed or not, and
`"clips_paused"` in `/api` beside `watchdog`, so in the MQTT state as
well. The Home Assistant entity, a diagnostic binary sensor of the
`problem` class, took the discovery payload with a full pack to 4771
bytes, past `DISCOVERY_MAX`'s 4608, so the limit is 5120 now (the
user: "Raise the limit, grow a buffer"); `.stack` went from 41960 to
41448 bytes, the 512 of the MQTT task's buffer. Twenty entities. One host test (the line with the flag and
without, the JSON both ways), 182 in all; the clock page at its worst
is 6101 bytes of 8032.

Run on test-clock the same day: a preview of `keygen` with
`/hang/clip` posted first, the feeder's reset, and then
`"clips_paused":true` in `/api` and the line on the clock page between
the skip checkbox and "Alarm sound" (no alarm armed); the plain build
sent to `/ota` put both back (`false`, no line).
With the entity in, test-clock's discovery message was read off the
broker after a `/reload`: 4450 bytes, twenty components, `clips_paused`
among them as written; the session stayed connected. Not looked at in
Home Assistant itself.

Still owed: a soak with no reset, and the longest gap read after a
clip, a save, an upload and a clip pack upload.

The checks as they were listed: the record kept across a `ResetCore`; a spin in
thread mode, a spin on the interrupt executor and a critical section
that never ends each reset the clock in a minute with the count up; a
parked display task is named; an OTA of an image that hangs is rolled
back; a soak with no reset and the longest gap in the heartbeat.

## An install from nothing on the bench, access point included — run 2026-11-17

The release list's install, and the access point's checks the session
could not make while the host's only Wi-Fi was its LAN: a USB dongle
(`wlp0s20f0u1`) joins the clock's network now, with a NetworkManager
profile that takes no default route and no DNS from it, so the host
stays on the LAN. No code changed. The image was `3eccec4` built as
the CI builds it (no network, no token, `TZ_RULE=<-07>7`), the merged
image, the OTA image and the default pack made with the release
workflow's commands; test-clock's flash was erased whole
(`espflash erase-flash`), the merged image written at 0 and the pack
at 0x2E0000. The pages were driven in headless Chromium, the PINs
read from the serial log.

What was seen, all as designed unless it says otherwise:

- **First boot.** `clips: pack of 4 clips`, the access point
  `clock-f044` on channel 1, the first scan in the log. The dongle
  joined in 2 s and had 192.168.4.2 from `dhcpd`, with the router and
  the DNS server 192.168.4.1.
- **Captive.** The DNS answers any name's A query with 192.168.4.1
  and an AAAA with no records; `/generate_204`,
  `/hotspot-detect.html`, `/settings`, `/ha` and an unknown path are
  a 302 to `http://192.168.4.1/`, a foreign `Host` too; `/`,
  `/system`, `/clips`, `/tz.js` and the favicon answer.
  `scripts/verify-csrf.sh 192.168.4.1` passed whole.
- **The setup form.** A listed network hides and disables the typed
  name; Show the password, the browser-zone button (`PDT7`,
  America/Vancouver) and Rescan work, Rescan keeping the pick and the
  typed password; two `POST /scan` at once were each answered after
  the one scan (2.4 s). A typed password under eight characters is
  stopped by the browser. Refused by the clock with nothing saved: a
  secured network with no password, a short password, a bad hostname,
  a bad timezone, each with its notice naming the field.
- **A wrong password.** Saved for `esptest`: the clock reset into the
  station, logged `wrong password?` at each of ten attempts 14.6 s
  apart, self-healed after 131 s and began again, and never became an
  access point. It was brought back by erasing `nvs` over USB.
- **The right one.** `vasilykh-wrt`, hostname `test-clock`, 11 dBm
  and the browser's zone in one save: the saved page, the reset, the
  address by DHCP 5.3 s after boot, the time synced.
- **The PIN.** A wrong PIN on `/token` is a 403 and a 9 s lockout
  that the next click reports; then the token set from the System
  page with the PIN, the firmware uploaded from the page with a PIN
  (907 KB in 17 s, "Back up … in ota_1" 7 s later, confirmed good at
  its first boot), Reboot with a PIN. With the token:
  `scripts/reload.sh`, `scripts/clips.sh`, a wrong Bearer refused.
- **Back to the access point**, from the page with a PIN and by
  `POST /ap` with the token: the setup form has the stored network
  picked and the stored hostname, zone and power; the System page
  there has no Access point button and says a token is set. Save
  untouched resets into the station at once; the timezone alone with
  an empty password saves, resets and joins with the stored password.

Found (fixed the same day, in the section after this one):

- A request head and body over `REQUEST_BUF` is an empty reply and no
  log line: 1530 bytes were answered, 1590 were not. The open item
  under "Nits", now measured on the air.
- `POST /api password=…` on the unprovisioned access point saved (a
  password set, no network) and did not reset: the open backlog item,
  as it describes.
- A refused setup form comes back as a fresh one: the picked network,
  the hostname and the rest are gone, and on a new clock the pick is
  "Other, typed below" again, so pressing Save a second time is "no
  network was named".
- The System page's line after an upload or a reboot reads
  "…; rebooting Waiting for the clock to come back.", two messages
  joined with no full stop (`back()` in the page's script).
- `agents/behaviour.md` did not list `/clips` among the pages the
  access point serves; it does now.

Not done: the buttons (the 10 s hold, the IP display), the panel
itself (`AP`, the PIN's digits, the blank station), a phone's sign-in
sheet, and the web installer in a browser. test-clock was left as it
was found (lime, alarm 01:00 disarmed, the broker, the local pack and
`futurama`, the session's token), on this image in `ota_1`.

## The install run's findings fixed — built 2026-11-17, run on test-clock

The four things the run above found, no new setting and nothing in
the record.

- **A request too long is answered.** `read_request` gave
  `NoRequest::Close` for a head with no end in `REQUEST_BUF`, a body
  over `BODY_MAX` and a head that does not parse. Now a `431`, a
  `413` and a `400`. An answer written with bytes unread is lost to
  the reset, so the first two read what is coming and throw it away
  first, the request buffer as scratch: the 413 the rest of the
  body by its length (`discard`), the 431, which cannot know a
  length, until the client has been quiet 100 ms or has sent eight
  buffers (`discard_until_quiet`). All of it is inside the
  listener's `HEAD_TIMEOUT`, so a body that takes longer than 2 s
  is still a close.
- **`/api` refuses a password with no network**, a `400`, when the
  request is what made it so (`Credentials::password_alone`, under
  test); a clock already in that state takes its other keys.
- **A refused setup form comes back filled in.** The refusal is a
  redirect, so the page is a new one. Its script keeps the pick, the
  typed name, the timezone, the hostname and the power in
  `sessionStorage` at the submit and puts them back when the query
  has `error=`; never the password. A pick no longer in the list
  falls back to "Other". A test ties the script's list to the
  form's fields. The page is 4761 bytes of 8032 at its worst.
- **The System page's line** has a full stop between the clock's
  answer and "Waiting for the clock to come back."

On test-clock, over the air, then with `nvs` erased for the access
point's half (a build as the CI makes it):

- on the station a 1.7 KB and a 5 KB cookie are a `431`, a 900-byte
  body to `/api` a `413`, `HELLO` a `400`, each with its line in the
  log, and the next request is served; on the access point the setup
  form with a 1.7 KB cookie is a `431`. A 909 KB body to `/api` is
  still an empty reply, the 2 s being over first;
- `POST /api password=…` on the unprovisioned access point is the
  `400` and `password_set` stays false; `color=red` after it is
  taken;
- in headless Chromium the form refused for a secured network with
  no password came back with the pick, `PDT7`, `test-clock` and
  11 dBm; a visit without `error=` shows the clock's own values; a
  typed "Other" name came back typed; the password field is empty
  each time; with the password typed it saved and joined;
- the Reboot line reads "rebooting with the time carried over.
  Waiting for the clock to come back."

One thing not explained: on the access point one `curl` straight
after three others was refused its connection (`Failed to connect`),
once; 60 posts in a row on the station, with and without a `400`
between them, were all answered. test-clock is on this build in
`ota_1` (the CI's configuration), its settings as before.

Not seen: the forms from a phone, whose head is what the 1536 bytes
are tight for.

## A preview never interrupts the alarm — built 2026-11-17, run on test-clock, not yet heard

The user's report: `/play` interrupts a ringing alarm; while the alarm
sounds, `/play` and `/stop` should be ignored.

**Cause.** The buzzer task takes the latest command whatever it is
doing, so a `Preview` during a ring replaced the loop with one pass of
the previewed melody and then silence, the panel still flashing; and
`POST /stop` was written to end a ring as a press does. The MQTT
`play` command went the same way as `/play`.

**Design.** The rule is the buzzer's, not each caller's:
`buzzer::preview` and a new `buzzer::stop_preview` do nothing while
`display::ringing()`, and say so in the log; `buzzer::ring` and
`buzzer::stop` stay the display task's, unconditional. `/play` and the
MQTT `play` call the first, `/stop` the second and no longer
`display::silence`. The flag goes up before the display task signals
the ring, and the display, HTTP, MQTT and buzzer tasks are all on the
thread executor, so a preview cannot slip in between the two; one
signalled just before is replaced or cut by the ring. Both routes
answer as before (the redirect), so the page shows nothing new. The
MQTT `silence` command is unchanged: ending a ring from Home Assistant
is what it is for, and a press still ends one.

**The bus's Stop** (the user, the same day: Home Assistant has a
button to preview and none to stop it). `silence` does both: the MQTT
task calls `display::silence`, which acts only while ringing, and
`buzzer::stop_preview`, which acts only while not. No new command,
entity or discovery byte, and nothing an older automation sent means
anything else.

**Built:** `src/buzzer.rs`, `src/http.rs`, `src/mqtt.rs`; the
manual, `agents/behaviour.md`, `agents/modules.md`, the changelog.
No pure logic in it, so no new test.

**Run on test-clock, 2026-11-17** (by OTA to `ota_1`, the monitor
attached, the alarm set a minute ahead; the board has no buzzer, so
this is the log and `/api`, not the ear). The ring was `keygen`, one
clip from start to end:

| at, ms of uptime | sent | log | `ringing` after |
| --- | --- | --- | --- |
| 71169 | | `buzzer: ring keygen` | true |
| 76614 | `POST /play`, `melody=halftime` (303) | `buzzer: preview ignored, the alarm is ringing` | true |
| 80699 | `POST /stop` (303) | `buzzer: stop ignored, the alarm is ringing` | true |
| 85183 | MQTT `cmd` `play` | `mqtt: play`, then the same `preview ignored` | true |
| 89247 | MQTT `cmd` `silence` | `mqtt: silence`, `clip: cut short, 0 underruns, 0 stray positions` | false |

One `clip:` line for the whole ring, at the silence, 18 s after it
began: nothing cut or restarted the clip before. With no alarm
after it, `/play` logged `buzzer: preview halftime` and `/stop` cut
it 4 s later (`clip: cut short`). The alarm was put back to 01:00,
disarmed.

**The bus's Stop run on test-clock, 2026-11-17** (a second OTA, to
`ota_0`): the MQTT `play` (`buzzer: preview keygen`) was cut by
`silence` 4 s later (`clip: cut short`), and so was a preview from
`/play` (`halftime`); with the alarm ringing, `silence` ended it 7 s
in as before (`ringing` false), the log saying `buzzer: stop ignored,
the alarm is ringing` beside it, which is `stop_preview` standing
aside for `display::silence`. The alarm was put back again.

**Not checked:** the ear, and the page's own buttons (the posts were
curl's, the same routes); a press ending the ring (nobody at the
buttons; the MQTT `silence` goes the same way through `ui.handle`).

## The state JSON's worst case — a review's finding, fixed 2026-11-17

The MQTT task formatted `form::write_json` into a 1024-byte buffer
that nothing tested. At its worst the JSON is 1467 bytes: over the
buffer the state would stop updating in Home Assistant with only
"state JSON does not fit" in the log. On a clock it is about 640
bytes, so no clock was near it; a 32-byte SSID of control characters
(six bytes each in JSON) with long names elsewhere would have been.

The limit is `form::JSON_MAX` in core now, 1536, and the MQTT task's
buffer is sized from it. `the_json_fits_the_state_buffer` builds the
worst case: a full pack's longest clip name, every choice at its
longest name, a status of extremes (the longest version and slot,
`u64::MAX` of uptime, the counters full, a heal stamp), and the tz,
the SSID and the broker user full of control characters whether or
not a rule lets them in (today only the SSID's does), so a rule
relaxed later cannot pass it unseen. `/api` answers from the response
buffer, a page's 8 KB: a `const` assert holds `JSON_MAX` under
`PAGE_MAX`, and another the state and its topic under the socket's
send buffer. `.stack` went from 41448 to 40936 bytes. 183 host tests.

Run on test-clock: the session connected and the state published
after the update. The worst case itself is the host test's.

## A 24 h soak of `v0.4-93-g43d96d9` on the bare bench board — started 2026-11-18

The watchdog's owed soak, and the first long run of the image with the
watchdog, the clip pack and the 16-bit transmitter fix. The board is
`another-bench` (10.1.1.65, on USB at `/dev/ttyACM0`): no panel, no
speaker, no sensors (the NTC reads -19 °C, the LDR dark; neither is a
finding), so nothing is seen or heard and the session may set it as it
likes (the user, 2026-11-17). Image as the user flashed it over USB,
`ota_0`, the home network and broker from `.env`.

Before the clock starts: the default pack by `scripts/clips.sh` (one
reset), then `espflash monitor` attached, whose reset is the start of
the run. Everything is host side and lives in `target/soak/`:
`serial.log` (the monitor), `watch.log` (the driver), `driver.sh`,
`summary.sh`.

Started 2026-11-17 23:57:00 MST (the boot in `serial.log`, reset
reason `CoreUsbUart`), to end 2026-11-18 23:57. `espflash monitor`
hung at "Connecting…" and left the chip in the ROM's download mode,
off the network, for five minutes, the watchdog not bringing it back
(it is not running there: the attach's reset is a whole-chip one); an
`espflash reset` booted it and the log is a plain `cat` of the port.
First heartbeat: heap free 54080, longest feed gap 1266 ms.

Load, heavier than a clock's day on purpose:

| every | what |
| --- | --- |
| 30 s | a 498-byte ping and `GET /api`; uptime, heals, watchdog, mqtt, rssi kept |
| 5 min | the five pages fetched, status and size kept |
| 10 min | `POST /play` of the next of the pack's four clips and two note melodies, `/stop` 30 s later |
| 1 h | a colour set over `/api`: a save, so a flash write |
| 6 h, first at 20 min | the alarm set two minutes ahead and armed, the melody the next clip; it rings its five minutes out, previews posted meanwhile |

Passes if, over 24 h: one boot in the serial log and the uptime never
goes back; `heals` and `watchdog` stay 0; no `WARN`, `ERROR` or panic;
heap free flat but for one-minute dips; the scan's descriptor index
moves between heartbeats; every `clip:` line without underruns or
stray positions and at the clip's own length (the half-rate fault
would double it); the longest feed gap well under `DEADLINE` after
clips, saves and rings; one MQTT session; the page's time equal to the
host's. A watcher ends the wait early on a reset, a heal, a watchdog
count, a warning or five `/api` failures running.

After the 24 h, since both reset the clock: an OTA of the same image
and the pack again, the feed gap read in the last heartbeat before
each, which pays the watchdog section's "after an upload and a clip
pack upload".

Not covered: a panel (the ring runs into no drivers), a speaker, the
buttons, the access point, the sensors.

## The soak again, 24 h of `v0.4-95-gb7cb573` on the bare bench board — started 2026-11-18

The run above was stopped at 9 h 13 min and found Trantor's stray
positions; this is the same soak, same board, load and pass criteria,
on the image with the I2S divider written in the IDF's sequence. It
still owes the watchdog's 24 h and the feed gap after the two uploads.

Differences from the first run: the image went in over the air
(`ota_1`, confirmed at 5 s), and that reboot is the start, reset
reason `CoreSw`, the serial log a `cat` of the port attached before
it; the default pack was already in flash (4 clips, 105156 bytes,
checked in 73 ms at boot), so no pack upload before the start. The
update's one `WARN` (`boot 1 of 2, rolled back if not confirmed`) is
expected and not counted. The first run's logs are in
`target/soak/run-2026-11-17/`; `watcher.sh` there is the watcher.

Started 2026-11-18 12:09:25 MST, to end 2026-11-19 12:09. The feed
gap before the upload, at 1140 s of the build before: 1184 ms.

The serial capture's `cat` was gone at 15:50 (uptime 13250 s; not
the clock: no reset, `/api` answering throughout, the port free and
not re-enumerated). Another session, working on test-clock on
`/dev/ttyACM1`, stopped its own capture with a `pkill` that matched
every `cat` and took this one too. Restarted appending at 15:51 with no reset;
the log lacks about a minute, the heartbeat of 13260 s among it.

The heap rested at 54080 bytes for 9 h 24 min, took one 188-byte step
to 53892 at the heartbeat of 33901 s (21:34) with nothing but the
`/api` tick and the page fetches in the minute before, and has held
there since (at 21 h). This is the step of the soak of 2026-11-08
(`HISTORY.md`: 54 032 to 53 844 at 21:35, given back at the first
rejoin 92 h later), so it is the association's, not a leak; that both
fell at the same minute of the evening points at the access point.

**Passed, 2026-11-19 12:09**: 24 h on one boot (last heartbeat 86403 s,
`/api` uptime 86419 s), heals and watchdog 0, no `WARN`, `ERROR` or
panic past the update's own. Heap 54080 or 53892 in all but seven of
1441 heartbeats, those one-minute dips (lowest 50672). Longest feed
gap 1180 ms from the first heartbeat to the last, through 96 clips,
the hourly saves and four rings (Trantor 12:32, Halftime 18:33, Keygen 00:34,
Synchronization 06:36). All 96 `clip:` lines with 0 underruns and 0
stray positions. One MQTT session (1470 published); the descriptor
index moving; 2870 ticks with no `/api` failure (slowest 141 ms) and
five lost pings, each alone; every page 200 and the page's time the
host's 287 times of 287; one BSSID and channel throughout.

The two uploads after it, the capture still attached. The image is
`b7cb573` built again (the soak's file had been overwritten by a
later build; the same 912480 bytes, not compared byte for byte):

| upload | last heartbeat before | result | first heartbeat after |
| --- | --- | --- | --- |
| OTA, 912480 bytes, 9.0 s, into `ota_0` | 86463 s, feed gap 1180 ms | `CoreSw`, confirmed at 5 s, pack checked in 73 ms | heap 54080, feed gap 1262 ms |
| the default pack, 105156 bytes, 1.1 s | 120 s, feed gap 1262 ms | `CoreSw`, 4 clips checked in 73 ms | heap 54080, feed gap 1341 ms |

Neither upload tripped the watchdog (count 0 in the carry record both
times). The gap during an upload itself is not in these figures: the
clock resets before the next heartbeat, so what is read is the boot's.
The heap was back at 54080 after the rejoin, as in the earlier soak.
The driver left the alarm armed at 06:36 on Synchronization.

## Twenty channels for the module player, and Crush tried in the default pack — built and measured on test-clock 2026-11-18; Crush heard and dropped the same day

The user asked for `xm::MAX_CHANNELS` at 20, after sixteen the same
day (`HISTORY.md`, "Sixteen channels for the module player"). It is
20 in `core/src/xm.rs` and in `scripts/xm2clip.py`, which refuses a
module with more; nothing else is sized by it. The mixer's sum is an
`i32` and a channel at full level is 32768 of it, the clip's channel
count a byte, the format the same `XMC2`: a clip of sixteen channels
or fewer is made and played as before. A firmware older than this
takes the first sixteen cells of a row of a wider clip for the row
and plays nonsense, with no harm, as an eight-channel one did of
sixteen; none with sixteen is released.

- The stack: 40 264 → 39 960 bytes (39.3 → 39.0 KB), 304 bytes.
- The build, clippy and `cargo host-test` pass; no test has a module
  of more than sixteen channels.

**Crush.** What the twenty were for: "Crush" by Drozerix, eighteen
channels, a MilkyTracker module the user took from The Mod Archive,
where it is module 179581, "Licensed under the Public Domain
license", by Drozerix, its MD5 that of the file in `mods/` (the
page read 2026-11-18). `mods/drozerix-crush.xm` is
the module, `clips/crush.xmc` the converter's defaults of it (31 250,
16 bits, 121 820 bytes, 124.7 s, nothing the player lacks named), and
it is the default pack's fifth clip, after the four so those keep
their places: `scripts/clips.sh`, `scripts/speaker-clips.sh` (with
the speaker's high-pass and release, as the other modules), the CI's
and the release's lists. `clips/README.md` credits Drozerix, the
changelog and `AGENTS.md` name the tune. The default pack is 227 040
bytes, 19 % of the partition; the parser's test reads it and the
players' tests pass with the five built in.

A released firmware (v0.4 and before) has no clip packs, so no
released player of eight channels meets this pack; a build of this
tree from before today does, and plays Crush as nonsense.

**On test-clock, 2026-11-18** (flashed over USB, `v0.4-100-gcb0f77f-dirty`;
`packs/clips-default-speaker.bin` written to the partition with
`espflash write-bin 0x2e0000`, in the local speaker pack's place;
`output` on the line; `/play` with `melody=crush`, the serial log a
`cat` of the port, which did not reset the clock):

| clip | filling | share |
| --- | --- | --- |
| Crush, eighteen channels, 31 250, played through | 71 569 of 124 770 ms | 57 % |
| Crush, a preview before it that this one cut short | 43 119 of 76 907 ms | 56 % |

7615 chunks, no underruns, no stray positions in either. The
watchdog's longest feed gap stayed at 1424 ms, the heap at 54 080
bytes, no gateway probe failed, and `/` and `/api` were answered
while it played. So the dearest clip there is: Children's sixteen
channels were 49 %.

**Dropped, 2026-11-18.** The user heard it on test-clock's line
output before and after the player's two fixes (the next section):
"It sounds a bit weird", then "It doesn't sound good. Keep the fixes,
but drop Crush from default pack." The default pack is the four clips
again, everywhere it was five; `clips/crush.xmc` went to the ignored
`xm/`, the speaker's stays in `packs/speaker/`, and the module went
there too, as `xm/drozerix-crush.xm` (the user, later the same day).
test-clock held the pack of five until Elysium's replaced it. The
twenty channels stay.

## The module player against openmpt123: vibrato's direction, a memory for each portamento — fixed 2026-11-18, heard on test-clock with Crush only

The user, of Crush (eighteen channels, the section above) previewed
on test-clock: "It sounds a bit weird", and asked
whether every instrument and effect of the module is played. The
converter names nothing missing: the module's effects are 1 2 3 4 8 A
B C D F, its volume column sets, slides, fine slides and panning (C,
D, E: 2294 cells of it), one sample an instrument, all of 8 bits, one
ping-pong loop. Panning is the one thing dropped, the clock being
mono.

Checked on the host against openmpt123 (mono, 31 250 Hz), the whole
mix and then each of the eighteen channels alone (the module split
into eighteen, the other channels' cells emptied but for B, D, F and
G; `CLIP_RAW`):

- The whole mix: every order's level within 0.5 dB of the
  reference's, and nine bands from 40 Hz to 15.6 kHz within 0.3 dB
  but the top two, 0.9 and 2.5 dB under (the averaging down).
- **Vibrato went the wrong way**: up in pitch first, where
  FastTracker's (and the reference's) goes down first. The lead of
  orders 43 and 44 was 13 to 27 cents off over an order for it.
  Fixed in `xm::Tune::vibrato`; the instrument's own vibrato does go
  up first and is as it was.
- **Portamento up and down shared one memory**; FastTracker keeps
  one each (a module of one cell 110 and then 200s does not move in
  openmpt123). Crush has 721 cells of 100 and 442 of 200. Fixed:
  `Channel::porta_down`.
- Both change Halftime and Keygen as well, which have 4xx and 6xx by
  the hundred (Keygen 1xx and 2xx too, each with its own parameter).
  The clips are the same files; the player reads them differently.
- After the fixes, per channel and order: pitch within 6 cents
  everywhere a channel is above -42 dBFS but the tails of three
  sweeps (channels 15, 16 and 17, at -32 to -37 dBFS, where a slide
  runs into the period's limit, which is not FastTracker's), and
  levels within 0.7 dB but channel 11, 0.7 to 1 dB under throughout,
  and those tails. Not chased.
- The fixes change 11 of the 48 orders' mix. So they are not likely
  to be all of "weird". What else differs from the module on a
  stereo player, and was not tried: the sides, 7 to 12 dB under the middle in most
  orders, are gone in mono; and the mix is 5 dB hotter than
  Halftime's or Children's (0.4 dB over one voice at full level, RMS)
  into the same limiter, which takes its gain down as far and holds
  it there more of the time.

The stack with the fixes: 39 880 bytes (39 960 before: eighty
bytes, four a channel for the new memory and its padding).

Not heard since the fixes: Halftime and Keygen, which they change.

## A MOD converter, and Elysium the default pack's fifth clip — built 2026-11-18, not yet heard

The user asked for a script that makes a clip of a `.mod`, and for
`mods/elysium.mod` in the default pack: "Elysium" by Jester of Sanity
(Volker Tripp, 1992; the module is signed in its sample names), four
channels, from The Mod Archive, module 40475. The page (read with
curl, 2026-11-18): `ELYSIUM.MOD`, MOD, 4 channels, "Licensed under
the Attribution Non-commercial Share Alike license", linked to
CC BY-NC-SA 4.0, "Registered Artist(s): Jester", and the MD5
`45b205dd41fe63c2de0e77ae749322de`, which is that of the file in
`mods/`.

**The converter.** `scripts/mod2xm.py` makes an XM, as `s3m2xm.py`
does, and `xm2clip.py` the clip; the player is untouched. A period is
the nearest note with 428 as C-4, and every instrument's finetune is
20 of 128 lower, a PAL Amiga's 8287 Hz at that period for FT2's 8363.
The XM is on the linear table, the only one `xm.rs` plays, so a slide
in periods (1, 2, 3, 5, E1, E2) is converted at the period the channel
has when it is played, the song followed once in order. Elysium uses
1, A, B, C, F and one E0 (the Amiga's filter, left out): four
portamentos, `101` each, become `101`.

**Checked on the host.** openmpt123's renders of the MOD and of the
XM, one channel at a time: the envelopes correlate at 0.999, 0.999,
0.986 and 0.998, no drift over the 222 s, the XM 2.5 dB lower on
every channel alike (openmpt's level for the format). The samples do
not line up wave for wave: the Amiga's periods are whole numbers, a
few cents off the notes they stand for. The clock's own mix
(`CLIP_RAW` dump) against openmpt's MOD: envelope 0.980.

**The clip.** `clips/elysium.xmc`, the converter's defaults: 126 666
bytes, 222.7 s, nothing the player lacks named. Fifth in the pack,
after the four so those keep their places: `scripts/clips.sh`,
`scripts/speaker-clips.sh`, the CI's and the release's lists,
`core/build.rs`'s comment; `clips/README.md` credits Jester and
states the licence, the changelog, the manual, the readme and
`AGENTS.md` say five. The default pack is 231 886 bytes, 19 % of the
partition; the parser's test reads it and the 183 host tests pass
with the five built in.

**The licence's cost.** CC BY-NC-SA is the first clip here with a
condition on use: the default pack, and a release's
`onebitdawn-clips-<tag>.bin`, may no longer be used commercially
(someone selling clocks with it flashed, say), and the clip is under
the same licence as the module. The firmware's own licence is
untouched; the clip is data beside it.

Owed: the user's ear on a clock (`melody=elysium`), the processor's
share while it plays (four channels: under Halftime's, expected).
The speaker pack of five is on test-clock since 2026-11-18.

**Withdrawn from the pack, 2026-11-19.** The user took Elysium out of
the default pack and out of the repository the next day: the module
and its clip are in the ignored `xm/` (`xm/elysium.mod`,
`xm/elysium.xmc`), the pack is the four again, and with the one clip
that had a condition on use gone the pack has none; the lists, the
counts, `clips/README.md`, the release notes and the installer's page
are as they were before it. `scripts/mod2xm.py` stays. What is owed of
this section is the converter heard on a clock, with any module.

## Two modules of our own: Grieg's Morning and In the Hall of the Mountain King — built 2026-11-19, heard in the page's player, reworked, three of them in the default pack; not yet heard on a clock

The user asked for XMs made here, of two movements of the first Peer
Gynt suite, Op. 46 (Grieg died in 1907; the music is in the public
domain): the notes found online, the instruments public domain or
under a permissive licence.

**The notes.**

- *In the Hall of the Mountain King* (No. 4): the Mutopia Project's
  piano score, piece 1888, typeset by Coyau from The University
  Society's edition of 1918 and placed in the public domain by the
  typesetter (`copyright = "Public Domain"` in the `.ly`). Its MIDI
  file is what the tune's file reads, `mods/src/mountain_king.mid`,
  with the `.ly` beside it; both as Mutopia gives them.
- *Morning* (No. 1): no score a program can read was found under a
  free licence (Mutopia has Nos. 2, 3 and 4 only; IMSLP has scans; the
  MIDI files about say nothing of their terms). So it was read off
  the composer's own arrangement for piano four hands, Edition Peters
  2432 (plate 7191), the scan the Sibley Music Library gives out
  (`hdl.handle.net/1802/16617`), by eye from the pages rendered at
  400 dpi, with a throwaway script that finds note heads and measures
  their height against the staff where the print is dense. The script
  caught one bar read a third too low; there may be others.
  **Shortened**: bars 1 to 31 and then 56 to 87. The middle (32 to
  55, two pages of chromatic arpeggios) is left out, the join is the
  G sharp major of the sunrise straight into the E major of the
  horn's return.

**The instruments.** Versilian Studios' VSCO 2 Community Edition
(`github.com/sgossner/VSCO-2-CE`, commit `4403009`), CC0. Morning:
flute, oboe, horn, the violins bowed twice (two registers), the
cellos twice. Mountain King: bassoon, oboe, trumpet, trombone, horn
and the violins held, cello, bass and violin pizzicato, a kettledrum,
cymbals. A note an instrument, mono, 8 bits (the player's samples are
8 bits: `xmc.rs` cuts 16 to 8), the high ones at 22 or 32 kHz, the
low at 11 or 16; a held one has a loop of 0.25 to 0.7 s found by
correlation and crossfaded. The files' names are an octave off as
often as not; the pitch is measured.

**The tool.** `scripts/mkxm.py` (numpy and scipy): a tune's file
defines `tune()`, which returns the module, and the script writes the
XM. It fetches the VSCO files a tune names into the ignored
`xm/vsco/`. The tunes are `mods/src/morning.py` and
`mods/src/mountain_king.py`; the modules `mods/grieg-morning.xm`
("Morning Mood": `morning` is a note melody's name already) and
`mods/grieg-mountain-king.xm`. Only what `xm.rs` plays is written:
notes, the volume column's set, the volume envelope with its sustain
point, an instrument's vibrato, 3FF for a slur, ED for a note late in
its row, K for a key-off late in its row, F for the tempo. `Voices`
is a set of channels that share notes so that none is cut while it
sounds.

```
scripts/mkxm.py mods/src/morning.py mods/grieg-morning.xm
scripts/mkxm.py mods/src/mountain_king.py mods/grieg-mountain-king.xm
```

**The arrangements.** Morning: the tune between flute and oboe over
held strings, the violins in octaves with everyone from bar 21, the
horn with it under flute ripples when it returns, the trills and the
bird calls, the cellos last. Mountain King: bassoons with the cellos
and basses plucked, then plucked violins and oboes an octave or two
up, then bowed violins and trumpets with trombones, kettledrum and
cymbals, the tempo from 104 to 208 where the MIDI file has 138 and
154; the left hand is an octave up throughout, the clock having no
bass.

**The user's ear, 2026-11-19**, on the first build in the pack
maker's page: "Mountain King sounds like there is no sustain on notes
at all. I don't think an actual musician would play it like this";
"an audible click between some notes"; "instrument sample rate is
rather low"; "Morning Mood also has abrupt notes ending". What was
wrong, and what was done:

- The first Mountain King was all staccato recordings cut to 0.3 to
  0.45 s. The score does say staccato on every note (Mutopia's `.ly`:
  `e sempre staccato`, an accent on the third beat) and the orchestra
  is cellos and basses plucked with bassoons at first; but a bassoon's
  staccato quaver has its tone and a plucked string rings on. Now the
  winds, brass and bowed violins are held recordings, a quaver two
  thirds of its length and a longer note nearly whole, and a plucked
  note is the recording for 0.7 to 1 s.
- **The player has no ramp** (`xm.rs`, `mix` and `trigger`): a note
  that starts on a channel replaces what was sounding there within a
  sample, and a level changes once a tick. FastTracker ramps both.
  The first build reused a channel while its note was still sounding
  or in its release, hundreds of times: the clicks, and the abrupt
  ends (a chord's release was cut by the next chord). Now
  `mkxm.Voices` gives a note a channel that has gone quiet, slurs a
  chord's note on from one of its instrument that ends there, and
  counts what it still has to cut: 4 in Morning, 46 in the Mountain
  King, 23 of them the kettledrum's roll. Morning has twelve ticks a
  row, so a release of 140 to 280 ms is ten to twenty steps.
  A ramp in the player is the whole cure, for every module: the user
  asked for it the same day, below.
- A bug of the script's own, found by the two players disagreeing by
  1.5 dB over twenty seconds: a note that ended in the row it began
  on was never let go, so Morning's bird calls hung. Fixed, and tried
  in both players row by row.
- The samples were at 8 to 16 kHz to keep the clips small. They are
  at 22 or 32 kHz where the instrument is high; the modules are 1.6
  and 2 times the size for it.

**Measured on the host, 2026-11-19**, the second build.

| | Morning Mood | Mountain King |
| --- | --- | --- |
| the module | 139 964 bytes | 197 685 bytes |
| the clip (`cargo pack --clip`, defaults) | 138 067 bytes | 194 884 bytes |
| length | 129.5 s | 139.6 s |
| channels with the tempo's | 20 | 18 |
| the player lacks | nothing named | nothing named |
| the clock's mix against openmpt123's | 2.4 to 2.7 dB over, 8 s at a time | 2.2 to 2.5 dB over, 10 s at a time |

- Each instrument alone in openmpt123's render is within 11 cents of
  the note asked for (the horn and the trumpet the furthest; the
  kettledrum and the cymbals have no pitch to measure).
- The loudest pitch a quaver in Morning's first four bars and bars 9
  to 12 is the flute's line as written (the first build).
- No note is left without a channel.
- With the buzzer's defaults the limiter flattens both: the shaped
  output is at -5 to -8 dBFS throughout, the crescendos gone (the mix
  before it runs from -29 to -14 dB in Morning and -25 to -18 in the
  Mountain King). That is the buzzer's 20 dB of drive; a speaker's
  defaults were not measured.
- `packs/grieg.bin` and `packs/grieg-speaker.bin`: the default four
  and these two, 438 235 bytes, 37 % of the partition.
- Morning is at the player's twenty channels. The processor's share
  is not known: Crush's eighteen were 57 %.

**A ramp in the player, 2026-11-19** (the user: "add the ramp to the
player"). In `xm.rs`, two things, neither of which softens a note's
own attack:

- A channel's gain is no longer the tick's level but moves to it over
  a 512th of a second (`RAMP_PER_SECOND`; 62 mixed samples at 32 kHz,
  122 at 62.5), a step a sample; a note that has just begun starts at
  its level. A key-off with no envelope, EC, the volume column and an
  envelope's steps all go through it.
- When a note, or E9, starts on a channel that is sounding, what the
  channel was adding to the mix at that sample goes into `Tune::cut`,
  one sum for the tune, which loses a 64th of itself a mixed sample
  (`CUT_DECAY`; half in 0.7 ms at 62.5 kHz, gone in about 5). The new
  note's first sample is not taken out of it: a square wave still
  starts with its edge.
- The cost: a test and at most an add for each sounding voice a mixed
  sample, and one for the tune. `Channel` is 16 bytes longer; `.stack`
  is 39 208 bytes (39 880 before).
- It changes every module's sound, Halftime and Keygen among them:
  their note cuts and volume steps are rounded off, their attacks are
  not. The clips are the same files.
- Checked: the new test `a_level_is_ramped_to_and_a_cut_note_dies_away`
  (a level is reached in steps over the ramp, a fresh note is at its
  level in its first sample, a cut note falls a 64th a sample to
  nothing); the 186 host tests, both clippys, the pack maker's tests
  and the release build pass. Against openmpt123, which ramps too, the
  two tunes are as before, within 0.3 dB.
- The pack maker's page was rebuilt with it (`cargo pack-wasm`) for
  the user to hear.

Owed for the ramp: the user's ear in the page and on a clock, Halftime
and Keygen included, and the processor's share with it (Crush's 57 %
was without).

**The instruments again, 2026-11-19.** The user, with the ramp in:
"The clicks are gone. I still don't like instruments you chose for
Grieg. They sound dull compared e.g. to dualtrax tunes. Is there
anything better under permissive license? But arrangements are good,
especially Morning Mood."

- Measured, openmpt123's renders, the share of the power over 2 kHz:
  Halftime 5.3 %, Morning 5.0 % (0.7 % over 4 kHz, Halftime 2.6 %),
  the Mountain King 8.4 % (1.7 % over 4 kHz). Not so far apart in
  sum; what Halftime has and these lack is over 4 kHz, where a square
  wave still has partials and a flute, a horn or a bassoon has next
  to none. The first choice made it worse: the softest recorded
  layers, in a room.
- MuseScore General (FluidR3's descendant, MIT; fetched into the
  ignored `xm/sf2/`) was looked at as the other permissive source:
  its flute, bassoon, horns and oboe have 0 to 5 % over 2 kHz, no
  brighter. `mkxm.py` reads a SoundFont now (`Sample.soundfont`); no
  kit was made of it. Its licence asks for its notices in a
  derivative work.
- Two kits beside the first, chosen by a third argument to the script
  (`scripts/mkxm.py tune.py out.xm bright`), for the user's ear, in
  the ignored `xm/ab/`:

| kit | what | Morning: module, over 2 kHz, over 4 kHz | Mountain King |
| --- | --- | --- | --- |
| (none) | VSCO, soft layers | 139 964 bytes, 5.0 %, 0.7 % | 197 685 bytes, 8.4 %, 1.7 % |
| `bright` | VSCO's loud layers where it has them, 10 dB more of what is over 2 kHz, the violins at 32 kHz | 150 242, 7.9 %, 1.4 % | 206 266, 13.5 %, 3.1 % |
| `chip` | waves made in the script: square, narrow pulse, saws, a triangle drum, noise for the cymbals | 23 321, 7.2 %, 2.6 % | 83 623, 8.9 %, 3.0 % |

  Every instrument of each kit is within 12 cents in openmpt123's
  render, and the converter names nothing missing in any.

**Decided, 2026-11-19** (the user, having heard the kits in the page:
"Keep bright and chiptune variants. Default pack should have Morning
bright and chiptune, and Mountain king bright."):

- The soft kit is gone from the tunes' files and its two modules from
  `mods/`. The kits are `bright` (the default) and `chip`; the modules
  `mods/grieg-morning-bright.xm` ("Morning Mood"),
  `grieg-morning-chip.xm` ("Morning Mood Chip"),
  `grieg-mountain-king-bright.xm` ("Mountain King") and
  `grieg-mountain-king-chip.xm` ("Mountain King Chip", in no pack).
- `clips/morning_mood.xmc` (144 787 bytes), `morning_mood_chip.xmc`
  (17 866) and `mountain_king.xmc` (203 465), the pack maker's
  defaults, are the default pack's fifth to seventh, after the four so
  those keep their places: `scripts/clips.sh`,
  `scripts/speaker-clips.sh` (from the modules), the CI's and the
  release's lists, `core/build.rs`'s comment, the pack maker's tests
  (the clips byte for byte from the modules, the pack's names and
  labels). `clips/README.md` has their sources and licences, the
  changelog, the manual, the readme and `AGENTS.md` say seven.
- The default pack is 471 466 bytes, 39 % of the partition, for a
  buzzer and for a speaker; the parser's test reads it, and the 186
  host tests pass with the seven built in.

**Morning ends at its rest, 2026-11-19.** The user, of the pack's
Morning: "Morning mood has a long rest at ~1:42-1:43, let's cut it
short there - I mean the tune should end there. The rest likely
sounds good with orchestra, not so good in mod". That is the score's
silent bar 76. The tune is bars 1 to 31 and 56 to 76 now, 104.0 s,
ending on the trill over the dominant with the silent bar before a
ringing alarm starts it over; the flute's last statement, the cellos'
and the closing chords (77 to 87) are gone from `mods/src/morning.py`.
The figures above for the two Morning clips and the pack are after
the cut; the tables further up are from before it. `mkxm.py` writes a
short last pattern now and no longer pads the tune to whole ones.

**On the bare bench board, 2026-11-19** (`another-bench`, on
`/dev/ttyACM0`; no panel, speaker or sensors, so this is the log and
not the ear. Updated over the LAN from `v0.4-95` to this tree,
`v0.4-111-g0d83913-dirty` in `ota_1`, in 9 s;
`packs/clips-default-speaker.bin` posted to `/clips`, 471 466 bytes
in 5 s, checked at boot in 325 ms; `output` on the line; each clip by
`POST /play` to its end; the serial log a `cat` of the port):

| clip | channels | filling | share |
| --- | --- | --- | --- |
| Morning Mood | 20 | 68 997 of 104 040 ms | 66 % |
| Morning Mood Chip | 20 | 66 313 of 104 045 ms | 64 % |
| Mountain King | 18 | 67 128 of 139 599 ms | 48 % |
| Halftime | | 10 462 of 43 571 ms | 24 % |
| Keygen | | 13 318 of 57 639 ms | 23 % |

- No underruns and no stray positions in any of the five; the
  watchdog's longest feed gap 1259 ms, no reset; the heap at 54 080
  bytes throughout; no gateway probe failed; `/` in 98 ms and `/api`
  in 24 ms while Keygen played.
- Morning is the dearest clip there has been (Crush's eighteen
  channels were 57 % before the ramp). What the ramp costs is in
  these figures and not apart from them: none of the five was
  measured on this board without it.
- The board was left on this build with the new pack, `output` back
  on the buzzer, its other settings as found.

Owed: the three heard on a clock, buzzer and speaker (with the
buzzer's defaults the limiter flattens their crescendos, measured on
the first kit); Halftime and Keygen heard with the ramp.

## The speaker's pack in a release — built 2026-11-19, the workflow not yet run

v0.5-rc2 had the buzzer's pack only (the user). The release workflow
now runs `scripts/speaker-clips.sh` (which takes the pack's path) and
attaches `onebitdawn-clips-speaker-<tag>.bin`, read back by
`the_pack_file_parses`; the CI makes and checks the same pack on every
push, so a tag is not the first time it is built. The web installer
still writes the buzzer's pack: a choice there is a second manifest a
tag and a control on the page, not built.

Owed: the next tag's release having the file.

## A 24 h soak of the v0.5-rc3 release, installed from nothing through the access point — passed 2026-11-20

The release candidate's own files, not a local build, on the bare
bench board (`another-bench`, `/dev/ttyACM0`; no panel, speaker or
sensors, so the alarm may ring: the user, 2026-11-19). The install is
the manual's espflash path: `onebitdawn-v0.5-rc3.bin` and
`onebitdawn-clips-v0.5-rc3.bin` fetched from the GitHub release, the
flash erased whole, the image at 0, the pack at 0x2E0000, a reset.
Then the setup as a new clock's owner makes it: the host's dongle
(`wlp0s20f0u1`, a profile with no default route) on the clock's
access point, the setup form in headless Chromium (the home network,
the hostname, the zone), and on the LAN the token set with a PIN read
from the serial log and the broker on `/ha`.

The soak is the one of 2026-11-18 (the section above of that date):
the same driver, watcher, load and pass criteria, in `target/soak/`
(the earlier run's logs moved to `run-2026-11-18/`). The previews go
round the pack's eight clips and two note melodies, and the alarm
rings every 6 h, first at 20 min, on the next clip. The start is the
boot after the last of the setup's resets.

The install, 2026-11-19, as planned (the release's pack is byte for
byte `packs/clips-default.bin`): first boot `firmware v0.5-rc3`,
`ota_0`, `pack of 8 clips, 727891 … bytes … checked in 501 ms`,
nothing stored, the access point `clock-1bf4` on channel 1. The
dongle had 192.168.4.2 and the host kept its default route;
`/generate_204` a 302 to the form. The form listed five networks; the
home one picked (the typed name hidden and disabled), the password,
"Use this browser's zone" (`PDT7`), the hostname `another-bench`, 16
dBm; "Saved", the reset into the station 2 s later, 10.1.1.65 by DHCP
at 5.6 s and the time synced at 5.7 s. On the LAN, from the pages:
the token set on `/system` with the PIN from the serial log, no
reset; the broker on `/ha`, connected 4 s after the save as
`clock-6a1bf4`. Seen on the way: after "token set." the System page
still reads "(none is set)" beside the field until it is loaded
again.

A mistake of the session's, not the clock's: the serial `cat` was
attached before `espflash reset`, which then fought it for the port
and left the chip in download mode with the capture dead; the reset
first and the `cat` after, as the soak of 2026-11-18 has it, worked.
That first boot's log is `target/soak/rc3/first-attempt-serial.log`.

Started 2026-11-19 16:18:07 MST (the station boot, `CoreSw`, the
setup's own reset), to end 2026-11-20 16:18. The watcher reads the
serial log from that boot on and does not count the setup's `API
token set` warning. First heartbeat: heap free 54080, longest feed
gap 1186 ms. Rings due at about 16:41, 22:41, 04:41 and 10:41, on
Morning Mood, Mountain King, Suspended Seconds and the chip Morning
Mood.

The broker went down at 16:22:58 (uptime 291 s) and is to stay down
for a while: the user is upgrading the host's services (2026-11-19).
So this run does not hold "one MQTT session"; it has instead the
session lost and found again, to be read from the log: the backoff,
the heap across it, the reconnect and the discovery published again.
The watcher was restarted not counting the `mqtt:` warnings and the
receive's `NotConnected` error.

Back at 16:31:14 (uptime 787 s), 8 min 15 s later: ten refused
connects on the backoff 5, 10, 20, 40 s and then every 60 s, the
eleventh connected to `nettop.lan` by its name (the user: its address
may have changed; 10.1.1.28 now). Heap free 54080 after it as before,
`2 connects`, no heal, no watchdog count, `/api` answering
throughout.

The first ring, Morning Mood at 16:41: its five minutes out (299987
ms, no underruns, no stray positions), the preview and the stop
posted meanwhile each logged as ignored. `/play` and `/stop` answer
303 in this release where the earlier soak's image answered 200; the
summary counts both as good.

The watcher ended at 17:50 on the run's first warning of the clock's
own: one gateway probe lost at uptime 5499 s (17:49:46),
`DestinationHostUnreachable (1 in a row)`, back at the next probe 30
s later with 14 ms; the host's ping and `/api` either side of it
answered (one `/api` took 1061 ms at 17:50:29), same BSSID and
channel, no heal. The soak of 2026-11-18 had no such line in 24 h.
Not counted as a fault: the watcher was restarted letting a lone
lost probe pass and still ending on two in a row; how many there
are goes in the result.

Five lone losses by 18 h 40 min (uptimes 5499, 5621, 8684, 60900
and 67114 s: three in the first evening, none overnight for 14 h,
two the next morning), each answered at the next probe. The user,
2026-11-20 about 11:00: there is 2.4 GHz traffic in the room,
Bluetooth headphones and a 2.4 GHz keyboard and mouse, and either
can get in the way of this board with its ceramic antenna. The
losses fall where someone is at the desk and stop overnight, which
fits; the room's traffic the first evening was not noted, so that
part is an inference.

Kicked off the access point by the user, on purpose, at 12:56:22
(uptime 74300 s; one association until then, 74294 s): `wifi:
disconnected … AssociationTooMany`, the link down, the MQTT session
lost. Seven connects 8.5 s apart refused by the access point with
`AuthenticationFailed` (the attempts counted "of 10", the seconds
"of 180", so the self-heal was three attempts away), the eighth
joined at 74368 s, 68 s after the kick: the same address by DHCP
0.5 s later, MQTT connected 1.2 s after that (the third session). No
reset, no heal, no watchdog count. A preview of Halftime had begun
6 s before the kick and played on through the scans and the
connects, 43565 ms with 0 underruns and 0 stray positions, ending
by itself as the driver's `/stop` could not arrive. The host lost
two ticks' pings and one `/api`. The watcher was restarted with the
kick's ten warnings as its baseline.

`target/` was removed under the run at about 13:37 (a clean build in
the tree), and `target/soak/` with it. The clock, the capture and
the driver ran on, writing to files with no name; both logs were
copied whole out of the processes' open files at 15:38 into
`soak-rc3/` (untracked), with the driver and the watcher, and
`refresh.sh` there copies them again before each reading. The
watcher saw nothing from 13:37 to 15:38; the logs for those two
hours were read afterwards and have no warning in them. Gone for
good: the release files as downloaded, the first attempt's serial
log, the access point's page as saved, and the logs of the soaks of
2026-11-17 and 2026-11-18, whose figures are in their sections
above. A soak's files belong outside `target/`.

**Passed, 2026-11-20 16:18**: 24 h on one boot (last heartbeat 86404
s, `/api` uptime 86419 s), heals and watchdog 0, no panic, and no
warning but the broker's outage, the user's kick and five lone lost
probes, each accounted for above. Against the criteria:

- Heap free 54080 until 21:34, then 53892 (the same 188-byte step at
  the same minute as the soaks of 2026-11-08 and 2026-11-18), 54032
  after the rejoin; eleven one-minute dips in 1440 heartbeats, lowest
  48780. No trend.
- Longest feed gap 1186 ms from the first heartbeat to the last,
  through 116 clips, the hourly saves, four rings and the rejoin.
- All 116 `clip:` lines with 0 underruns and 0 stray positions; the
  four rings (Morning Mood 16:41, Mountain King 22:42, Suspended
  Seconds 04:44, the chip Morning Mood 10:45) each five minutes.
- The descriptor index moving; 84366 images published, no stop.
- The gateway probe: 5 lost of 2878, none two in a row.
- The host: 2868 ticks; outside the kick's minute one lost ping and
  no `/api` failure, the slowest 1061 ms; every page 200 and the
  page's time the host's 287 times of 287.
- Not held, both by an outside hand: "one MQTT session" (three: the
  broker taken down, the kick) and one association (two).

The driver was stopped at the end and left the alarm armed at 10:45
on the chip Morning Mood, the colour blue. The uploads the earlier
soak ended with (an OTA of the same image, the pack again) were not
made. The logs are in `soak-rc3/`.

## The licences of what a release is built from — built 2026-11-20, run on the host; the workflows not yet run

A release hands out binaries (the images, the pack maker's commands,
its WebAssembly on the page) built from other people's crates under
MIT, Apache-2.0 and the like, whose notices are to go with a copy;
until now a release had none of them (the user asked whether the web
interface wants a copyright line: it does not, this is where the
notices belong). Release process only, no firmware change, so it is
within the freeze.

Design:

- `cargo about` (pinned in the workflows, a prebuilt binary as
  espflash is) over the workspace, every member and every target a
  release builds for: the chip, the three systems of the pack maker,
  wasm. One list for all of them, a superset for each.
- `about.toml` names the targets and the licences accepted; a
  dependency under one not in the list fails the run, so a new
  licence is seen when it arrives, not at a tag.
- `about.hbs` is a plain-text template: a preamble for what is not a
  crate (the project's own licence, the second-stage bootloader
  espflash puts in the merged image, the radio's binary libraries,
  the clips with a pointer to `clips/README.md`), then each licence's
  text once with the crates under it.
- `scripts/licenses.sh [out]` makes the file; the CI runs it on every
  push, the release attaches `onebitdawn-licenses-<tag>.txt` and puts
  it on the pages as `licenses.txt`, which both pages' footers link.
- `README.md`'s License section and `agents/behaviour.md`'s
  "Releases" say so; `CHANGELOG.md` gets a line.

Built as designed, with cargo-about 0.9.3. What the run showed:

- 171 crates: 162 taken under MIT (of an "MIT OR Apache-2.0" the
  first accepted is chosen), ryu under Apache-2.0, the four riscv
  crates ISC, smoltcp, managed and xarxa-driver 0BSD, unicode-ident
  with Unicode-3.0 for its tables; the last three were added to
  `accepted`. The file is 106 KB.
- Some fifty crates, esp-hal's and embassy's among them, are
  published with no licence file, and get the licence's plain text
  with `<year> <copyright holders>`; the preamble says so. Nothing
  better is in the package.
- The radio's binaries (`libnet80211.a`, `libphy.a` and the rest) are
  in `esp-wifi-sys-esp32c3`, not in `esp-phy`. Checked the same day
  (the user asked): the `LICENSE` of `esp-idf`, `esp32-wifi-lib`,
  `esp-phy-lib` and `esp-coex-lib` is Apache-2.0 in each. But one of
  the binaries is `libwpa_supplicant.a`, ESP-IDF's build of Jouni
  Malinen's wpa_supplicant and hostapd, BSD with three clauses, and
  the firmware links it (some 150 `wpa_`, `hostap` and `sm_` symbols
  in the ELF); a binary is to reproduce that notice, so the preamble
  has it in full, from the component's `COPYING` and `README`.
- The workspace's own crates are left out (`private`), for which the
  root manifest got the `publish = false` the other two had.
- The binary needs `--features cli` from `cargo install`; the
  workflows take the prebuilt archive, whose name and layout were
  checked by downloading it.

Owed: a push's CI run with the new step; the next tag's release
having the file and `licenses.txt` on the pages (until then the
footers' link has nothing behind it). Not looked for: other
third-party code inside Espressif's binaries or the bootloader beyond
the supplicant.

## `UTC+3` in the timezone field is east — built 2026-11-20, run on white-clock

The user typed `GMT+3` into the timezone field and the clock went to
UTC-3, and `GMT-3` gave UTC+3 (2026-11-20). The parser was right by
POSIX, where the offset after a name counts west (`TZ=GMT+3 date`
does the same), and the label says "POSIX rule"; but nobody who types
`GMT+3` means three hours west, the user included. A fix, so on
`master` in the freeze.

The design: a value that is all of `UTC` or `GMT` (either case), a
sign, and `h[h][:mm]` is the offset as people write it, east
positive (`tz::shorthand`), and `form::apply_one` stores `tz::fixed`
of it, `<+03>-3`, as it already does for the old key's plain minutes;
past `TZ_LIMIT_MINUTES` it is a bad value. Normalised at input and not
in the parser, so the record holds a rule every image reads the same
way and a rollback keeps the zone. The sign is required: `GMT0` and
`GMT0BST,M3.5.0/1,M10.5.0` are rules as before, and so is every other
name (`MSK-3`). The build's `TZ_RULE` stays POSIX alone, checked by
the compiler. Refusing the form as ambiguous was the other choice,
and the stricter; the user took this one. The label is now "Timezone,
POSIX rule or UTC±N".

What it costs: a clock that has `GMT+3` stored in POSIX's sense (three
hours west, under that name) keeps it until its settings form is next
saved, which sends the field back and turns it over. `<-03>3` is how
to say that now.

Tested on the host: `tz::the_shorthand_counts_east` (the forms taken
and those left to the parser) and the cases in
`form::a_refused_form_names_its_field`; the page sizes still fit.

On white-clock, 2026-11-20, over `/api` with the token (the user had
tried the page by hand first): `GMT+3` stored `<+03>-3` with
`utc_offset` 180, `GMT-3` `<-03>3` and -180, `utc+5:45`
`<+0545>-5:45`, `UTC-03:30` `<-0330>3:30`, `UTC+0` `<+00>0`; `MSK-3`
and `GMT0BST,M3.5.0/1,M10.5.0` stored as sent; `UTC+15` a 400. The
run found one hole: the first `shorthand` had a grammar of its own
(two digits of minutes), so `GMT+3:5` was not it, went on to the
parser, which takes one digit, and was stored as a rule, 185 minutes
*west*. `shorthand` is now the parser's own reading of such a text
turned over, so there is no offset one takes and the other does not;
odd seconds, which `fixed` cannot write, are a bad value.

The second image, the same day and the same way: the five above as
before, `GMT+3:5` `<+0305>-3:05` with `utc_offset` 185,
`UTC-03:30:00` `<-0330>3:30`; `UTC-03:30:15`, `UTC+15` and `GMT+24`
each a 400 with the zone left alone; the rules as sent; the old
key's `180` and `-420` as fixed rules. The clock's own `PDT7` put
back after each run.

## A pack's clip this firmware does not play is skipped — built 2026-11-22, on the host only

The user, 2026-11-22: "if we have an unsupported clip in the pack,
can we skip the clip instead of rejecting the whole pack?"; and of
where it goes, "while it is not a bug fix, it is a
forward-compatibility fix which might be worth getting into the first
public release". So it is a change of its own, written to apply to
`master` as it is to `next`, and it is the user's to say which.

Until now a pack with a clip of a version the firmware does not play
was refused whole, at an upload after the write and at boot, and the
clock ran on the note melodies. Every new version of a clip's format
after the first public release would be that for every clock not yet
updated, and for one rolled back. A firmware that skips can be given
a newer pack and plays what it can of it.

- `Pack::parse` skips a clip `clip::is_unsupported` says is of a
  known kind's other version or has its header cut short, and keeps
  the pack. `Pack` then is the clips that play: `count` and `clip`
  number those alone, so the melody table, the pages, `/api` and the
  discovery payload have nothing to learn. `skipped` and
  `skipped_clip` are for saying so.
- A pack none of whose clips plays is refused (`PackError::Unplayable`),
  and so is one with a clip that plays and is shorter than a second
  (`PackError::Clip`), as before: that is a pack made wrong, not a
  newer one. Names are still held to the rules for every entry,
  skipped or not.
- What says so: the boot's log line and the upload's answer give the
  skipped clips' number, and the Clips page says how many the pack
  has that this firmware does not play.
- An alarm whose sound is a skipped clip is the beeps, by the rule
  for a name that is not there, and gets its sound back with a
  firmware that plays it.
- What does not change: a firmware up to v0.5 refuses such a pack
  whole, and nothing done now can change that; a clip of a new *kind*
  is still PCM with no header to a firmware that does not know its
  magic, so such a pack must still set a reserved bit or the version.
  The pack maker still refuses a clip made already that the players
  do not read.

Host: the tests of `pack.rs` and the Clips page's; the page is 3978
bytes of 8032 at its worst. To check on a clock: a pack with a clip
of an unknown version uploaded, the answer, the boot's log line, the
Clips page and the list of sounds.

## "Starlight & Aurora" clicks on a speaker: the limiter off, its knee from 12 to 64; its hiss is the transistor's stage — built 2026-11-20, heard on test-clock and white-clock

The user, with the speaker's pack on test-clock: "есть щелчки".

**What was measured.** The clip rendered on the host by the firmware's
player (`CLIP_PCM_DUMP`, `CLIP_WIDE`; `CLIP_RAW` for the mix before the
shaper), 111 s at 31 250 samples a second.

- The mix before the shaper has no step and no burst, and is as
  `openmpt123` renders the module. The player is not the cause.
- With `knee=12,release=13` the shaped clip is at full scale (±32512)
  for 1254 samples, in runs of up to 12, at 181 places, most of them
  between 48 s and 96 s. The mix peaks near 62 000 and a knee of 12 is
  12 288, so the limiter is never off; its attack is instant, and every
  sample over the envelope as it has decayed goes to full scale, a
  flat top on each new peak. "Suspended Seconds" with the same options
  is there for 51 samples at 11 places.
- The knee, `release=13` throughout (on the bare bench board the clip
  as it was played through with 0 underruns, 0 stray positions, 70 of
  111 s filling):

| knee | samples at full scale | RMS, dBFS |
| --- | --- | --- |
| 12 to 32 | 1254 to 1211 | -10.6 |
| 40 | 394 | -11.8 |
| 48 | 63 | -13.2 |
| 56 | 5 | -14.5 |
| 64, the limiter off | 0 | -15.7 |

**What was heard.** A pack of the clip at 12, 48 and 64
(`packs/starlight-aurora-knee-ab-speaker.bin`) on test-clock's speaker.
The user: "С 48 хорошо, но в тишине слышно шипение". So
`scripts/speaker-clips.sh` makes it with `knee=48,release=13`.

**The hiss.** A clip of zeros (ADPCM codes of 0, made by hand: the WAV
converter refuses silence, and brings sox's dither of one step up to
full scale, which was the first, wrong, "silence") is cleaner than the
tune's quiet notes (the user), so it is not the converter or the
amplifier. It is the samples' 8 bits:

- The render has a flat floor over 5.6 kHz, where a piano sampled at
  11 025 Hz has nothing: 37 to 40 dB under the tune at any level,
  whatever the knee.
- Each sample against what it was rounded from (`scripts/mkxm.py`,
  measured with the float kept): the piano's notes 40 to 44 dB over
  their rounding, and their loops, which are what a held note is,
  35 dB (D4, F4) to 44; the bells 41 to 44; the two waves 46 and 49.
- A part alone, its share of the band over 5.6 kHz against the whole
  mix: the bell -42 to -47 dB, the piano -52 to -59, the lead and the
  bass under -61.

**What was tried for the hiss, 2026-11-20, and dropped.** The user
asked for the samples nearer full scale and for the rounding's error
shaped. Measured as the 8-bit module's render less that of the same
module with 16-bit samples (`openmpt123`, linear interpolation):

- Shaping the error: not built. At a sample rate of 11 or 12 kHz the
  whole band is where the ear is keen, and the best filter there is
  gains under 1 dB, A-weighted.
- Nearer full scale: the piano's `DEPTH` from 12 to 18 dB (its worst
  loops, D4 and F4, from 35 to 41 dB; 24 gives the mix no more), and
  the bell's level moved into an envelope of its instrument with a
  loop of whole periods (43.7 to 48 dB, and under a held bell the
  rounding is the note's harmonics). In the mix: the rounding 47.2 dB
  under the tune before and 48.1 after, 44.2 and 45.8 in the quiet
  stretches.
- The band over 5.6 kHz did not move with either, so it is not the
  rounding: it is the images of linear interpolation, 42 dB under the
  tune over 9 kHz, most of it the bell's. With the samples at 22 050
  and 24 000 Hz it is 9 to 14 dB lower, in a clip of 428 KB.
- Heard on test-clock, the three as `sa_a`, `sa_b`, `sa_c`
  (`packs/starlight-aurora-hiss-ab-speaker.bin`). The user: "шипение
  больше всего слышно у пианино в самом начале. Я не слышу разницы в
  шипении между 3мя вариантами". So neither was kept, and the module
  is as it was.
- The start, where the piano is alone: the shaper adds nothing (the
  mix before it and the clip after have the same floor), and the floor
  between the partials is the 8 bits' (4 to 5.5 kHz: -56.7 dB of the
  total with 8-bit samples, -70.9 with 16-bit; 5.5 to 9 kHz -60 and
  -69). A piano note's attack is 43 to 45 dB over its rounding
  whatever the envelope takes, since the hammer's peak is what full
  scale is.

Not fixed, and not to be had from 8 bits: a few dB is not heard. What
would do it is samples of 16 bits, which the player and the clip
format do not have: a feature, for after v0.5. The piano's alone are
about 140 KB more (the pack 1090 KB of 1152); with the bells' too it
does not fit.

**The knee again, and what the hiss is, 2026-11-20 (later).** The
user, after the clip at 48 and at 64 on both clocks: "Лимитер выключай
вообще, knee=64". At 64 no sample is at full scale and the clip is
-15.7 dBFS, as loud as "Suspended Seconds"; `scripts/speaker-clips.sh`
makes it so.

The hiss is not the samples', whatever is said above. A firmware that
plays 16-bit samples was built on `next` and the piano given them: on
test-clock, whose speaker is on the buzzer's transistor (the S8550,
the speaker mod), the hiss was as before, and on white-clock's
amplifier the same clip was clean ("sa_a хорошо звучит на усилителе -
прямо как надо :) а на S8550 (speaker mod) есть шипение"). Every
hearing of the hiss above was on the transistor. On it a pure sine of
250 Hz or 1 kHz, a looped 16-bit sample with nothing over 1.5 kHz
within 54 dB, hisses too, and a clip of zeros much less ("Синусы
шипят, тишина шипит заметно меньше"). So it is that stage under any
signal, and "Suspended Seconds" covers it better. On the amplifier 16
bits against 8: "16 бит лучше, но 8 бит тоже приемлемо", so the tune
goes out at 8 bits in v0.5 and the 16-bit samples stay on `next`.

**The transistor's stage and the converter's bit rate, 2026-11-20.**
On test-clock's speaker (the S8550, the speaker mod) the user: "Синусы
шипят, тишина шипит заметно меньше", of sines of 250 Hz and 1 kHz
that are clean in the clip. So the stage hisses under any signal. Then
the same clips through the converter at 256 bits a sample (8 Mbit/s,
the default) and at 128 (4 Mbit/s), by a bench switch in `src/clip.rs`
that is not committed, each run through with no underrun: "128 бит
звучит лучше на speaker mod - меньше шипения". Half the edges, less
hiss: what one would expect of a transistor that turns off more slowly
than it turns on, which makes the line's level depend on the bits'
pattern and so brings the converter's noise, all of it far over the
audio band, down into it; with zeros in, the pattern is regular and
there is nothing to bring down. Not measured on a scope. 64 bits a
sample the driver refuses for this converter (`Pdm(InvalidClock)`: the
up-sampling's `fs` would be 960, over its 480), and at 16 kHz it was
white noise (2026-11-13).

**128 bits a sample on the buzzer's pin — built 2026-11-20 on `next`,
heard there, on `master` since 2026-11-21 (the user: "Работает", then
"это всё-таки фикс").**
`clip::converter_bits` gives a clip on the buzzer's pin
`BUZZER_CONVERTER_BITS`, 128, and one on the line output the build's
`CLIP_PDM_BITS`; `src/clip.rs` reads the `output` setting once a clip
and takes its pin and its bits from that. The bench switch is gone.
Heard on test-clock's speaker (the transistor) and on white-clock,
its amplifier and, with `output` at `buzzer`, its buzzer, which is
not a stock one: a stock buzzer at 128 bits and 31 250 Hz nobody has
heard.

## A 24 h soak of master, `v0.5-rc4-3-g9039ef5`, on the bare bench board — started 2026-11-21

Master as it stands, three commits past `v0.5-rc4`, a local build of
a clean tree; the first long run of the buzzer's pin at 128 bits a
sample (`9039ef5`) and of the timezone commits. The board is
`another-bench` (10.1.1.65, `/dev/ttyACM0`; no panel, speaker or
sensors; alarms and previews allowed: the user, 2026-11-21), found on
`v0.5-rc3` in `ota_0` with the record the rc3 soak left: the home
network, the broker, the `.env` token, the output `buzzer`, the alarm
armed at 10:45.

The soak is the one of 2026-11-18 and 2026-11-20 (their sections
above): the same driver, load and pass criteria, the files in
`soak-master/` (untracked, outside `target/`). In order: master's
default pack by `scripts/clips.sh` (one reset), the serial `cat`
attached with no reset, then the image over the air by
`scripts/ota.sh`; that reboot (`CoreSw`, into `ota_1`) is the start,
and the update's `boot 1 of 2` warning is not counted. The previews
go round the pack's eight clips and two note melodies, the alarm
rings every 6 h, first at 20 min. The watcher reads the serial log
from the start's boot on, lets a lone lost gateway probe pass as the
rc3 run settled, and ends on anything else: a second boot, a warning,
a heal, a watchdog count, a clip with underruns or stray positions,
five `/api` failures running.

Started 2026-11-21 00:35:26 MST, to end 2026-11-22 00:35. The pack
`scripts/clips.sh` made is byte for byte the one of 2026-11-20
(727891 bytes), written in 8.0 s; the image 916944 bytes, 9.1 s into
`ota_1`, `CoreSw`, the pack checked in 501 ms, the address at 5.8 s,
confirmed at 5.9 s, MQTT at 7.0 s. The feed gap before the upload, at
180 s of rc3: 1180 ms. First heartbeat: heap free 54080, longest feed
gap 1178 ms. Rings due at about 00:57, 06:57, 12:57 and 18:57, on
Morning Mood, Mountain King, Suspended Seconds and the chip Morning
Mood. `start-line` there is the line of `serial.log` the run begins
at; `summary.sh` gives the figures so far.

Seen on the way: `stty -F /dev/ttyACM0 raw -echo -hupcl` hung for two
minutes, the port having echo on and the clock's log echoed back at
it, so the set waited for output nobody reads. Setting the port from
Python with `TCSANOW` after a `tcflush` did it at once, no reset.

**Passed, 2026-11-22 00:35**: 24 h on one boot (last heartbeat 86404
s, `/api` uptime 86415 s), heals and watchdog 0, no panic, and no
warning but the update's own and three lone lost gateway probes.
Against the criteria:

- Heap free 54080 until 21:34, then 53892 (the 188-byte step at the
  heartbeat of 75544 s, the same minute as the soaks of 2026-11-08,
  2026-11-18 and 2026-11-20); three one-minute dips in 1440
  heartbeats, lowest 50672. No trend.
- Longest feed gap 1178 ms in every heartbeat, through 116 clips, 24
  saves and four rings.
- All 116 `clip:` lines with 0 underruns and 0 stray positions, on
  the buzzer's pin at 128 bits a sample; the four rings (Morning Mood
  00:58, Mountain King 06:59, Suspended Seconds 13:00, the chip
  Morning Mood 19:01) each five minutes, 299939 to 299991 ms.
- The descriptor index moving; 84351 images published, no stop. The
  index read 2 once in 1440 heartbeats where the others read 118 to
  146 times each; the rc3 run's log has it twice, so it is not this
  image's, and what makes descriptor 2 rare at the heartbeat's moment
  is not looked into.
- The gateway probe: 3 lost of 2879 (uptimes 27705, 27767 and 29840
  s, 08:17 to 08:53), none two in a row.
- One MQTT session (1468 published), one association, one BSSID and
  channel.
- The host: 2870 ticks, no `/api` failure, the slowest 295 ms; four
  lost pings, each alone (the first at 33 s of the boot); every page
  200 and the page's time the host's 287 times of 287.

The driver was stopped at the end and left the alarm armed at 19:01
on the chip Morning Mood, the colour blue; the clock runs on in
`ota_1` with the serial `cat` still attached. The two uploads the
soak of 2026-11-18 ended with were not made. The logs are in
`soak-master/`.

Left running after the soak with no load but the daily alarm, the
capture attached, and read again 2026-11-23 07:59 at 55 h 23 min
(last heartbeat 199388 s, `/api` uptime 199433 s): still the one
boot, heals and watchdog 0, no panic. Heap free 53892 in every
heartbeat since the step of the 21st but nine one-minute dips (52188
or 50484), and no second step at 21:34 on the 22nd. Longest feed gap
1178 ms in all 3323 heartbeats. One association and one MQTT session
(3353 published) throughout; 190716 images published, no stop. One
more lone lost gateway probe, at 198182 s (07:38 on the 23rd): 4 of
6645 in all. The alarm rang by itself at 19:01 on the 22nd, the chip
Morning Mood, its five minutes with 0 underruns and 0 stray
positions. The serial `cat` was stopped then (the user asked), with
no reset; the clock runs on, the alarm still armed daily at 19:01.

## The installer asks: buzzer or speaker — built 2026-11-21, deployed by `v0.5-rc5`, no install from it yet

The web installer wrote the buzzer's pack whatever the clock had (the
section of 2026-11-19 above left the choice unbuilt); the user,
2026-11-21: the page should ask which clips to install. A feature, on
a `master` in feature freeze: the user asked for it as a last touch
before v0.5.

Design. ESP Web Tools takes one manifest, so a release gets two:
`manifest.json` as before, with the buzzer's pack, and
`manifest-speaker.json`, the same four parts and
`onebitdawn-clips-speaker-<tag>.bin` at 0x2E0000 (under `latest/`,
`onebitdawn-clips-speaker.bin`). The page has a step between the
release and the install, "Choose the sound", two radio buttons with
the buzzer checked, and the install button's manifest follows the
release and the choice. The choice is the pack only: the `output`
setting is the clock's, in `nvs`, which the installer does not write,
and the page says so for a clock with an amplifier on the line
output. No firmware change.

The page and the manifests are deployed by the same run, so `latest/`
always has both. A tag deployed before this has `manifest.json` only;
all of those are pre-releases or the old names, which the list does
not offer.

Checked on the host: the page's script in headless Chromium, with
the list of releases stubbed, the manifest attribute following both
controls (`v0.6/manifest.json`, `v0.6/manifest-speaker.json` after
the speaker, `v0.5/manifest-speaker.json` after the release,
`v0.5/manifest.json` after the buzzer); the workflow file parses as
YAML. Its `manifest` function is the old one called twice more and
was not run: the host has no `jq`.

`v0.5-rc5` (2026-11-21) is the first tag with it: the run passed, and
the page has the step, `manifest-speaker.json` and the speaker's pack
(949750 bytes) under `v0.5-rc5/` and `latest/`, the manifest's fifth
part naming that pack at 3014656.

After the tag (the user, 2026-11-21): the line about the `output`
setting was easy to miss in the step's small print, so it is a notice
of its own, shown only while the speaker is chosen. Not deployed
until the next tag.

Owed: an install of each choice from the page onto a clock.

## A 24 h soak of the v0.5-rc6 release, installed from nothing through the access point — passed 2026-11-25, with a Wi-Fi outage and a self-heal on the way

The release's own files on the bare bench board (`another-bench`,
`/dev/ttyACM0`), as the rc3 soak of 2026-11-19 was run (its section
above): `onebitdawn-v0.5-rc6.bin` and the buzzer's pack
`onebitdawn-clips-v0.5-rc6.bin` fetched from the GitHub release, the
flash erased whole, the image at 0, the pack at 0x2E0000, a reset and
only then the serial `cat`. The setup as a new owner makes it: the
dongle (`wlp0s20f0u1`, a profile with no default route) on the
clock's access point, the setup form in headless Chromium (the home
network, the hostname, the browser's zone, 16 dBm), and on the LAN
the token set on `/system` with a PIN from the serial log and the
broker on `/ha`.

The driver, the watcher, the load and the pass criteria are those of
the soaks of 2026-11-18, 2026-11-20 and 2026-11-21; the files are in
`soak-rc6/` (untracked). The start is the station boot the setup's
save makes; the watcher does not count the setup's `API token set`
warning and lets a lone lost gateway probe pass. The board was found
on a local build, `v0.5-rc5-20-g722ce63-dirty` in `ota_0`, with the
output `line`; the erase leaves the output at its default, `buzzer`,
as the earlier soaks had it.

The install, 2026-11-24, as planned (the release's pack is 727891
bytes, as rc3's was): the access point `clock-1bf4` on channel 1,
`/api` there `v0.5-rc6` in `ota_0`, `clips: pack`, nothing stored,
the output `buzzer`. The dongle had 192.168.4.2 and the host kept its
default route; `/generate_204` a 302 to the form. The form listed
five networks; the home one picked (the typed name hidden and
disabled), the password, "Use this browser's zone" (`PDT7`), the
hostname `another-bench`, 16 dBm; "Saved", the reset into the station
2 s later, 10.1.1.65 by DHCP at 5.6 s and the time synced at 5.7 s.
That boot's log: `firmware v0.5-rc6`, `pack of 8 clips and 0 of a
version this firmware does not play … checked in 501 ms`. On the LAN,
from the pages: the token set on `/system` with the PIN from the
serial log, no reset; the broker on `/ha`, connected 3 s after the
save as `clock-6a1bf4`. The very first boot's log was not captured:
setting the port raw from Python reset the board (the access point's
uptime was 33 s a minute and a half after `espflash reset`), and the
capture begins 29 s into that second boot.

Started 2026-11-24 11:39:23 MST (the station boot, `CoreSw`, the
setup's own reset), to end 2026-11-25 11:39. First heartbeat: heap
free 54080, longest feed gap 1185 ms, one MQTT connect. Rings due at
about 12:02, 18:02, 00:02 and 06:02, on Morning Mood, Mountain King,
Suspended Seconds and the chip Morning Mood. `start-line` in
`soak-rc6/` is the line of `serial.log` the run begins at;
`summary.sh` there gives the figures so far.

The first ring, Morning Mood at 12:02: 299943 ms, no underruns, no
stray positions.

The watcher ended at 16:43:47, 5 h 4 min in, on the home access
point going away, not on the clock: at 16:43:04 the host lost it too,
on both its interfaces at once (`CTRL-EVENT-BEACON-LOSS` on the
dongle on 80:69:1a:1f:25:cc, `Connection to AP 80:69:1a:1f:25:cd
lost` on the 5 GHz one), and the dongle came back on the house's
other access point, 28:d1:27:06:3c:de on channel 3. The clock, at
uptime 18225 s: one gateway probe lost, `esp_wifi_internal_tx
returned error: 12294`, `disconnected … after 18224 s: BeaconTimeout,
rssi -50`, the link down, the MQTT session lost; 6.8 s later the scan
found only the far access point at −82, joined it in 2.7 s, the same
address by DHCP 0.6 s after that (10.1 s from the disconnect), MQTT
connected 4.3 s later (the second session), the probe back at its
next turn. No reset, no heal, no watchdog count; heap free 54032 at
the next heartbeat. Two `http: write: ConnectionReset` warnings
followed, the host's requests cut by the outage; the host lost one
tick's ping and `/api` and one round of pages. Why the access point
dropped is not known (the user was not asked yet).

The clock stays on the far access point, at −80 to −82 where it had
−39 to −52: it does not roam back, and nothing short of a reset moves
it. The run goes on there, so from 16:43 it is a soak on a weak link,
and its lost probes and the host's failures are to be read with that.
The watcher was given a baseline (`watcher.sh <end> <count>`) and
restarted not counting the outage's seven warnings.

It ended again three minutes later, at 16:46:31, as the weak link
said it would: gateway probes lost at uptimes 18317, 18379 and 18411
s, the last two in a row (4 failed of 614), and one `/api` in 1232
ms. The criteria of a −40 link do not fit this one. Restarted
(`watcher.sh <end> 5 'probe: gateway\|http: L[0-9] write'`) letting
every probe warning and every cut write pass, and still ending on a
second boot (a self-heal after ten probes lost in a row would be one,
and would put the clock back on the near access point), a panic, a
watchdog count, a clip's underruns, a disconnect or five `/api`
failures running.

That one ended at 16:49:48 on five `/api` failures running; the clock
was up, the link was not: ten probes lost of 623 by 16:51, in runs of
two to four, one answered in 1134 ms. The probe comes back before ten
in a row each time, so the self-heal does not fire.

The outage was the user's own, a couple of minutes of the home
Wi-Fi, and the near access point was back by 16:51 (the host's dongle
on it again); the clock stays on the far one. The user, 2026-11-24:
no reset, "let's see how it survives on marginal link". So from
16:43 the run is that: the same boot, the same load and the same end,
2026-11-25 11:39, and what is asked of it is no longer the −40 link's
criteria but that the clock itself holds: one boot or a self-heal
that is the design's (ten probes lost in a row), no panic, no
watchdog count, the feed gap and the heap as before, every clip that
plays with 0 underruns, and the link's losses counted. The watcher is
`watcher-marginal.sh`, which ends on a second boot, a panic, a
watchdog count in a heartbeat, a clip's underruns or strays, or the
serial log silent for five minutes, and leaves the link's warnings
to the summary.

Seen and to go in the backlog whatever the run says: a station that
rejoined a much weaker access point during an outage never looks for
the better one again, and a link that answers one probe in four keeps
the self-heal from firing for as long as it does.

**The marginal link, 16:43 to 17:02: 19 minutes, ended by the
self-heal as designed.** The far access point's signal fell from −82
to −86…−90; 28 probes lost of 644, the runs growing (1, 1, 2, 4, 3,
then 10); the MQTT session lost twice more and not found again in
the last five minutes; the host reached the clock at 9 ticks of 30
(21 pings and 19 `/api` lost). A preview posted at 16:58 played, and
ran on to 40 s with its `/stop` not arriving, 0 underruns and 0
stray positions like the 26 before it. At uptime 19391 s (17:02:33)
`heal: gateway 10.1.1.1 unreachable for 10 probes (300 s) with the
link up; self-heal reset 1 since power-up: blanking the panel and
rebooting`. The boot after it, `CoreSw`: `heals: 1` and the time
carried, the pack checked in 501 ms, the scan saw both access points
(−87 and −40) and took the near one, the address at 5.6 s, the time
synced at 5.7 s, MQTT at 6.8 s. Throughout the 19 minutes: heap free
54080 (53840 once), the feed gap 1185 ms, no watchdog count, no
panic.

The first stretch, then, is 5 h 23 min on one boot (19391 s), clean
for the 5 h 4 min before the outage: 27 clips and one ring with no
underruns, one probe lost of 606 (the outage's), heap 54080 with
three one-minute dips, the feed gap 1185 ms in all 323 heartbeats.

**The 24 h begins again at the self-heal's boot**, 2026-11-24
17:02:33 MST, to end 2026-11-25 17:02, on the near access point with
the first run's criteria and its watcher (`watcher.sh`, now expecting
`heals:1` in `/api`); the driver ran on through it all, so its rings
stay where they were due, about 18:02, 00:02, 06:02 and 12:02.
`start-line` is this boot's line of `serial.log` (`start-line-1` and
`end-epoch-1` the first stretch's).

**Passed, 2026-11-25 17:02**: 24 h on one boot from the self-heal's
(last heartbeat 86404 s, `/api` uptime 86425 s), heals still 1 (the
outage's), watchdog 0, no panic, and no warning but two lone lost
gateway probes. Against the criteria:

- Heap free 54080 until 16:50 of the second day, then 53892 to the
  end (the 188-byte step of the earlier soaks, here at the heartbeat
  of 85684 s and not at 21:34: on 2026-11-24 at 21:34 this boot, four
  and a half hours old, showed nothing); nine one-minute dips in 1440
  heartbeats, lowest 50672. No trend.
- Longest feed gap 1180 ms in every heartbeat (1185 ms in the first
  stretch's), through 115 clips, 28 saves and four rings.
- All 115 `clip:` lines with 0 underruns and 0 stray positions; the
  four rings (Mountain King 18:08, Suspended Seconds 00:10, the chip
  Morning Mood 06:11, Trantor 12:12) each five minutes, 299966 to
  300032 ms.
- The descriptor index moving; 84351 images published, no stop. The
  index read 2 once in 1440 heartbeats, as in the master soak.
- The gateway probe: 2 lost of 2880 (uptimes 14142 and 52764 s), none
  two in a row.
- One MQTT session (1468 published), one association, RSSI −36 to
  −40.
- The host: 2870 ticks, no `/api` failure, the slowest 387 ms; three
  lost pings, each alone; every page 200, all 314 actions answered,
  and the page's time the host's 286 times of 287 (the one, at
  21:33:59, the host's 21:33 against the page's 21:34: the minute
  turned between the two reads).

With the first stretch that is 29 h 23 min of rc6 on this board, and
the only reset in it the self-heal the outage led to. The driver was
stopped at the end and left the alarm armed at 12:12 on Trantor, the
colour lime; the clock runs on in `ota_0` with the serial `cat` still
attached. The uploads the soak of 2026-11-18 ended with were not
made. The logs are in `soak-rc6/`; `summary.sh` there counts
`watch.log` whole, the far access point's 19 minutes included.

## A keygen-style tune of our own for the alarm, "Serial Dawn" — built 2026-11-25, in the default packs, on white-clock in the speaker's pack

The user asked for a module in the key generators' manner, of
samples made in the script and none recorded, for the speaker and the
buzzer alike: a rising arpeggio to open, tonic, subdominant, dominant,
tonic in the middle, little bass, and its end going round to what
follows the introduction. It went into the default packs in
Trantor's room, on master: a change of the melodies with no code
changed is not a feature (the user).

The design:

- `mods/src/serial_dawn.py` for `scripts/mkxm.py`, `mods/serial-dawn.xm`.
  A minor, 150 beats a minute, a row a sixteenth (speed 6), a pattern
  four bars, 28 bars: 44.8 s the first time, 38.4 s a round.
- The introduction, four bars, order 0: sixteenths climbing through
  Am, Dm, E and E7, from one quiet voice to full level, an echo three
  rows behind from the second bar, bass and hats from the third, a
  snare roll in the last beat.
- A, eight bars: Am, Dm, E, Am twice, a pulse lead with its echo,
  chord stabs as effect 0 arpeggios, an octave bass, drums.
- B, eight bars: F, G, Em, Am, F, G, E, E, ending on the
  introduction's climb over the dominant.
- A again with rolling sixteenths under the lead; its last bar is the
  pickup into A, and the module's restart position is order 1.
- Instruments: pulses of three widths (lead, stabs and bass, echo), a
  swept sine for the kick, noise for the snare and the hat. The bass
  is a pulse an octave up from where a bass would sit (A2 to G3) and
  under the lead's level.
- `mkxm.py` gets `Module.loop_from(row)`, the restart position, which
  it wrote as 0; `xmc.rs` and `xm.rs` have it already.

Built as designed: 21982 bytes of module, 19880 of clip, eight
instruments in 6888 bytes of samples, fourteen channels and the
tempo's, no note without a channel. The XM's restart position and the
clip's byte 28 are 1. Two things learned:

- A narrow pulse has a mean, and a note of it starting or stopping is
  a step. The pulses are put about their mean in the tune's script
  (`mkxm.py`'s `pulse` is as it was, for the Grieg kits).
- openmpt123's render against Keygen's (the module, 48 kHz, no
  limiter): 20 to 150 Hz is 14 dB under the whole here and 4 dB under
  there; 1 to 3 kHz 3 dB under here, 8 there. Its RMS is 9 dB under
  Keygen's before the clip's limiter.

The notes were written without being heard, so the tune, its levels
and the limiter's settings were the user's ear's to judge. `packs/serial-dawn.bin` and
`packs/serial-dawn-speaker.bin` (ignored) hold it alone.

In the default packs, 2026-11-25: `clips/serial_dawn.xmc` (the pack maker's defaults, 19880
bytes) is the buzzer's eighth clip, after Suspended Seconds, and the
speaker's pack makes its own from the module before Starlight and
Aurora, which is the ninth again; 680 KB and 902 KB. Both scripts,
both workflows, the pack maker's tests (the clip from the module byte
for byte, the names and labels), `clips/README.md`, the manual, the
README and the changelog have it. It is the project's, under its
licence. The speaker's pack was written to white-clock (nine clips,
the line output) and the user had it committed after; what he heard
is not recorded here.

## Trantor out of the default pack — built 2026-11-25, on the host

The user: the right to distribute Trantor's title music is not to be
had, so it leaves the default pack and the repository's tree for the
local packs. A fix of what is distributed, so on master.

- `clips/trantor.bpr` is `tap/trantor-packed.bpr` (ignored), and
  `packs/local.sh` takes it from there under its old name and label,
  first in the three local packs as it was.
- The default pack is seven clips, 660 KB, and the speaker's eight,
  882 KB: `scripts/clips.sh`, `scripts/speaker-clips.sh`, both
  workflows (the pack, the built-in clips' test run, the pack maker's
  check), the pack maker's tests, `clips/README.md`, `MANUAL.md`,
  `README.md`, `CHANGELOG.md`.
- The melody is kept by name: a clock with Trantor chosen rings its
  default after the new pack. The beeper player and
  `scripts/zxbeeper.py` stay; no clip in the repository is a beeper
  clip now, so the CI's run with the clips built in no longer covers
  `the_beeper_clips_built_in_add_up` (the synthetic beeper tests
  remain).
- The file is still in the repository's history and in the packs of
  the releases up to v0.5-rc6; nothing was done about either.

Run on the host: `cargo pack-test`, `cargo host-test` with and
without the seven built in, both packs made, `packs/local.sh`.

## Halftime out of the default pack; Keygen is CC BY 3.0 — built 2026-11-26, on the host

The user: Dualtrax published Keygen on SoundCloud as CC BY (the
page's licence field is `cc-by`; 3.0 Unported by the user's word),
and no licence is to be found for Halftime, whose author does not
answer. So Keygen stays under that licence and Halftime leaves the
default pack and the tree for the local packs. A fix of what is
distributed, so on master.

- `clips/README.md` has Keygen's terms: the credit, the track's
  page, the licence's link, what was changed (converted, labelled).
- Halftime's module and clip are in the ignored `xm/`, and
  `packs/local.sh` converts the module from there under its old name
  and label, second as it was; the buzzer's local pack is the same
  786 623 bytes.
- The default pack is seven clips, 669 KB, and the speaker's eight,
  891 KB: the same places as for Trantor, above. The pack maker's
  check in both workflows takes `clips/serial_dawn.xmc` for the clip
  made already.
- The melody is kept by name: a clock with Halftime chosen rings its
  default after the new pack.
- The files are still in the repository's history and in the packs
  of the releases up to v0.5-rc6; nothing was done about either.
- Found on the way, not fixed: `packs/local.sh` gives the speaker's
  and the amplifier's packs `--limiter`, which master's pack maker
  has not, so only the buzzer's local pack is made here.

Run on the host: `cargo pack-test`, `cargo host-test` with and
without the seven built in, both default packs made, the buzzer's
local pack.

## n1k-o's "lerka" the default pack's third clip, "Lera"; the pack maker writes R as E9 — built 2026-11-26, heard on white-clock in the speaker's pack

The user has Oleg Nikitin's permission for `nq-lera.xm` ("nq/skrju -
lerka", 2012, for TSRh; both names are his). A clip of the default
pack, as the last three changes to it were, so on master; the
converter's change with it is what the clip needs to be played whole.

- `mods/nq-lera.xm` as found; `clips/lera.xmc` at the pack maker's
  defaults, 27 412 bytes, 18 channels, 115 s. Name `lera`, label
  "Lera", third, after Synchronization (the user's choices).
- The module has R02 26 times, a roll on one instrument, which the
  player has not. `xmc.rs` writes an R that changes no volume as E9
  (x of 8; x of 0 on a channel with no R that has a volume; y not 0),
  over the packed cells in place, so the roll plays on every firmware
  that plays a module. No other clip in `clips/` changed by a byte.
  The player's E9 retriggers on the ticks y divides, as R does.
- The default pack is eight clips, 697 KB, and the speaker's nine,
  919 KB (the module again at a speaker's defaults). Scripts, both
  workflows, the pack maker's tests and the documents say so.

Run on the host: `cargo pack-test`, `cargo host-test` with and
without the eight built in, the pack maker's clippy, both default
packs made and parsed.

On white-clock the same day, the speaker's pack over the build of
`next` it had (the firmware is not in this change): the user heard
the clip and passed it. Not looked at: the buzzer's pack on a clock,
and the `clip:` line for underruns at 18 channels.

## Samples of 16 bits in a module, for the piano of "Starlight & Aurora" — on `next`, a feature for after v0.5

The section above has why: under a quiet piano the user hears the
rounding of its 8-bit samples, 43 to 45 dB under a note's attack, and
nothing done within 8 bits is heard.

**Design.**

- The clip. A module with a 16-bit sample is `XMC3`; one with none is
  written as `XMC2`, byte for byte what it was, so the clips there are
  and their tests do not move. `XMC3` is `XMC2` with one of an
  instrument's three spare bytes, the first, as the sample's flags:
  bit 0 for samples of 16 bits, little-endian, signed. The sample's
  length and its loop stay in samples, so the player's positions and
  9xx do not change; only the bytes are two a sample.
- An older firmware takes `XMC3` for a module's clip of a version it
  does not play and refuses the pack (`clip::is_unsupported`): what
  the compatibility rules ask of a new version. A pack with such a
  clip is therefore for this firmware or later, and a clock rolled
  back under it has no pack until it is updated again.
- The player (`core/src/xm.rs`). A channel knows whether its sample
  is wide. The mixer's sample is `a + (b - a) * frac / 256` with `a`
  and `b` at 16 bits, an 8-bit one being its byte times 256: the same
  numbers as before for the modules there are. What a second byte a
  sample costs the mixer is to be measured on a clock (the `clip:`
  line's time filling; the tune was 70 of 111 s on the bare board).
- The converter (`tools/pack/src/xmc.rs`). A module's 16-bit samples
  are cut to 8 as they were, unless `samples=16` asks for them kept: a
  pack maker's page must go on making packs every firmware takes, and
  16 bits are twice the bytes.
- `scripts/mkxm.py`: `Sample(bits=16)` keeps what it was rounded
  from at 16 bits, and the module's writer says so in the sample's
  type.
- The tune: the piano's ten samples at 16 bits, about 140 KB more
  (the speaker's pack about 1090 KB of 1152); the bells stay at 8,
  with them it would not fit. `scripts/speaker-clips.sh` makes the
  clip with `samples=16`.

**Built, 2026-11-20.** As designed, with two things the design did
not have: a module's 16-bit samples cut to 8 are rounded and no
longer have their low byte dropped, and the limiter of the tune is
off, `knee=64` (the user: "Лимитер выключай вообще, knee=64").

- Host: the modules there are come out byte for byte (`cargo
  pack-test`); a 16-bit sample plays as the same at 8 bits times 256,
  to the sample (the test found the second place a sample is read,
  for a note that is cut). The tune's clip is 363 309 bytes (221 795
  at 8 bits) and the speaker's pack 1 091 264 of 1 179 648. At the
  start of the tune the floor between the partials is 15 dB lower at
  4 to 5.5 kHz and 9 lower at 5.5 to 9, and as `openmpt123` has it.
- test-clock: the tune through with 0 underruns, 0 stray positions,
  80 of 111 s filling (72 %; "Suspended Seconds" 23 of 48 s there).
  The same tune at 8 bits was not timed on that board, so what the
  second byte costs is not known; on the bare board at 8 bits it was
  70 of 111 s. One bounds check for the two bytes changed nothing.

**What the hiss is, by the user's ear, 2026-11-20.** Not the samples.
With the piano at 16 bits he still heard it on test-clock, whose
speaker is on the buzzer's transistor (the S8550, the speaker mod),
and "Suspended Seconds" has none there. On white-clock, the same
firmware and clip: "sa_a хорошо звучит на усилителе - прямо как
надо :) а на S8550 (speaker mod) есть шипение". So it is the
transistor's stage under this tune, the line output being clean, and
every hearing of the hiss before that was on that stage. What was
found on the way and is true but is not it:

- the 8 bits' rounding, 43 to 45 dB under a piano note's attack (the
  section above);
- the images of linear interpolation: the tune's first 12 s have
  -48 dB of the whole at 9 to 15 kHz, and the same notes mixed on the
  host from the recordings -75 (`sa_x`, a clip of one 16-bit sample);
  the piano's samples are at 11 025 Hz and its low partials come back
  round that, where the other tune's are at 16 000 and 22 050;
- a loop keeps the treble the note had when the loop starts, 6 to
  17 dB too much over 2 kHz two seconds into a held note
  (a low-pass of the loop was tried in `mkxm.py`; in the mix it changes nothing
  that is measured, the next notes' attacks being over it, and the
  tune does not use it).

Not known: whether 8 bits and 16 differ on the amplifier, which is
what says if this feature is wanted; and why the transistor's stage
hisses under this tune and not the other (a guess, not measured: the
converter's bits at 8 Mbit/s through a transistor that does not turn
off as fast as it turns on, which folds the converter's noise down
under a quiet, dark tune that covers nothing).

**Found on the way, 2026-11-20, not fixed.** Tones made to try the
transistor's stage with (a looped 16-bit sine, `packs/tones-speaker.bin`)
show two things about the shaper's high-pass (`core/src/shape.rs`):

- Every module's clip for a speaker has a constant offset of about
  -580 of 32512 (1.8 % of full scale) after the high-pass that is
  there to take one out: the section's `>> 14` rounds down, half a
  step each sample on average, and the feedback's gain at 0 Hz is
  1 / (1 + a1 + a2), about 2500 for 100 Hz at 31 250 (reckoned -614,
  measured -578 to -582, in a tone, in the tune, loud or soft). At
  the buzzer's 300 Hz it is a ninth of that. Not heard as such; it
  moves the converter off mid-scale by that much. A PCM or ADPCM clip
  has no such filter and no offset, the clip of zeros among them.
- `highpass=20` gives a clip stuck at full scale: the section's
  coefficients in Q14 with the corner that low.

**To check.** Host: the 8-bit modules' renders unchanged (the pack
maker's byte-for-byte tests, the players' tests over the clips); a
16-bit sample against the same at 8 bits; the tune's render against
the module's with `openmpt123`. On test-clock: the time filling, no
underruns, and the user's ear at the start of the tune.

## 16 bits in 8 by prediction, for a module's samples and for sound files — on `next`, a feature for after v0.5; built 2026-11-22, on the host only

The section above keeps a module's 16-bit sample at twice the bytes.
This keeps it at 8 bits a sample and a sixteenth more, as the
difference from what the samples before it predict: the SNES's sample
format (BRR) with codes of 8 bits for its 4. What follows is the day
in its order: the design as first built, with seven fixed predictors
and for modules only; then what was measured after it, and the
design as it now is, under "The format as it is".

**Why, measured on the host, 2026-11-22** (the samples `mkxm.py` has
before it rounds, each scheme against them; signal over the rounding's
noise, over all of a kit's samples):

| | 8 bits | µ-law | a shift a block of 16 | predicted |
| --- | --- | --- | --- | --- |
| "Suspended Seconds", `upright` | 39.4 dB | 37.9 | 46.1 | 67.4 |
| "Starlight & Aurora" | 43.2 dB | 37.9 | 45.2 | 55.9 |

Every piano sample is at 62 to 76 dB predicted; the four bells of
"Starlight & Aurora" at 48 to 61 (partials near half their 12 kHz
rate predict badly), which is what holds that tune's figure down. In
the mix before the limiter, against the mix of 16-bit samples:
"Suspended Seconds" 43.7 dB at 8 bits, 48.0 with the shift, 65.0
predicted; "Starlight & Aurora" 47.3, 48.6, 62.3. The user's ear on
the host, the tunes rendered through the player each way (`tmp/`):
of the shift, "I cannot hear the difference between 8bit and
block16", "16-bit is way better"; of the predicted, "sounds close
enough to 16bit. No more grit in the tails of piano".

**Design.**

- The sample. An 8-byte head, then blocks of 17 bytes for 16 samples
  each: a byte with the predictor in its high nibble and a shift in
  its low one, then sixteen signed residuals. A sample is
  `predict(the three before it) + (residual << shift)`, held to 16
  bits. The predictors are seven: nothing, the last sample, the line
  through the last two, the parabola through the last three (FLAC's
  fixed ones), and three of the SNES's that lean on the last two less
  (60/64; 122/64 and -60/64; 115/64 and -52/64). The last block is
  filled out with residuals of nothing. The head is what the loop
  needs: the three samples before the block the loop starts in, the
  newest first, and the sample at the loop's start, each an `i16`;
  nothing for a sample with no loop. The length and the loop stay in
  samples.
- The clip. `XMC4`: `XMC3` with bit 1 of an instrument's sample flags
  for a predicted sample. One with none is written as `XMC3` or
  `XMC2` as before, byte for byte. An older firmware refuses `XMC4`
  with its pack, as the compatibility rules ask.
- The player (`core/src/xm.rs`). A channel on a predicted sample
  keeps two decoded blocks, the one its position is in and the next,
  and the mixer reads its two samples from them as it reads the flash
  for the others; a block is decoded, in a loop of its own predictor,
  when the position enters a new one. The first try decoded a sample
  each time the position moved and cost 26 and 50 % of the player's
  time on the host for the two tunes; this costs nothing that the
  host can measure (-2 and -7 %), the mixer's loop having no branch
  on how far the position went. At the loop's end the sample after
  the last is the head's, and the wrap decodes the loop's block from
  the head's three. Any other move that is not to the next block
  (9xx, a retrigger, a step of more than a block) decodes forward
  from the nearest of where it is, the loop's block and the sample's
  start: exact, and slow for an offset far into a long sample, so
  the converter does not predict an instrument a module plays with
  9xx.
- RAM: 64 bytes of blocks and the three samples a channel, about
  1.5 KB over twenty channels in the tune's state. To check against
  the stack (`agents/sizes.md`).
- The converter (`tools/pack/src/xmc.rs`). `samples=packed` predicts
  a module's 16-bit samples; `samples=16` keeps them as they are and
  the default cuts them to 8, both as before. The encoder tries each
  predictor with the shift its residuals ask for and the ones beside
  it, a block at a time, with the decoder's own arithmetic on the
  samples as decoded, so a sample's error is its own residual's
  rounding and nothing adds up. An instrument played with 9xx
  anywhere in the module is kept at 16 bits.
- The tunes. "Starlight & Aurora" for the speaker's pack goes from
  `samples=16` to `samples=packed`. "Suspended Seconds" has its
  module's piano at 8 bits and stays so until the user says: at 16
  bits the module is twice the size, and the buzzer's clip must come
  out as it is.

**To check.** Host: the modules and clips there are, byte for byte
(`cargo pack-test`); a predicted sample plays as its decoded samples
kept at 16 bits, to the sample, through a loop, a 9xx and a
retrigger; the encoder against the measurements above. On a clock:
the `clip:` line's time filling and its underruns for "Starlight &
Aurora" (80 of 111 s at 16 bits on test-clock), and the user's ear.

**Built, 2026-11-22, on the host only; no clock has played it.** As
designed, with the encoder trying every shift for every predictor (63
pairs a block; the tune's clip is made in 0.2 s).

- The modules and clips there are come out byte for byte (`cargo
  pack-test`), and a predicted sample plays as its decoded samples at
  16 bits, to the sample: through loops that start and end off a
  block's edge and are shorter than a block, a sample's end, 9xx,
  E9x, a note cut by another, a pitch that steps over whole blocks,
  two channels on one sample, a clip cut short.
- "Starlight & Aurora" for a speaker: 230 794 bytes predicted,
  363 309 at 16 bits, 221 795 at 8; the speaker's pack 958 749 bytes
  of 1 179 648 (1 091 264 with the 16 bits). Its mix before the
  limiter, against the 16 bits': 70.6 dB predicted, 49.5 at 8 bits
  (the bells are at 8 bits in the module on `next`, so this is the
  piano's alone); after the limiter 49.3 and 36.6.
- The stack: 39 208 to 37 528 bytes, 84 bytes a channel of twenty,
  against `next`'s HEAD built in a worktree.
- Not done: "Suspended Seconds" (its module's piano is at 8 bits),
  the bells of "Starlight & Aurora" at 16 bits in the module, which
  packed would now fit.

**Against CD-XA's ADPCM, 2026-11-22** (the user asked). XA's Level A
is this scheme: 8-bit residuals, a filter and a range a sound unit.
It has 28 samples a unit and four filters, none of an order over
two: nothing, 60/64, 115/64 with -52/64, 98/64 with -55/64 (the
PlayStation's chip adds 122/64 with -60/64). The same samples through
the same encoder with each set of parameters, signal over noise:

| | "Suspended Seconds" | "Starlight & Aurora" | bytes a sample |
| --- | --- | --- | --- |
| ours: 7 predictors, block 16 | 67.4 dB | 55.9 | 1.0625 |
| XA Level A: 4 filters, unit 28 | 63.8 | 55.5 | 1.0357 (1.143 in XA's own sound groups, which carry each unit's byte twice or four times over) |
| XA's filters, block 16 | 64.6 | 55.9 | 1.0625 |
| our predictors, unit 28 | 66.5 | 55.5 | 1.0357 |
| XA and the PlayStation's fifth, unit 28 | 64.8 | 55.8 | 1.0357 |

So for the pianos XA's parameters are 3.6 dB worse, 2.7 of it the
predictors (the parabola through the last three samples is the one a
piano's blocks choose most, 60 % of them, and XA has nothing like
it) and 0.9 the longer unit; for the bells, which no predictor here
says much of, they are the same. XA's layout does not fit either: a
128-byte sound group for a CD sector, its headers repeated against
read errors, no loop (the PlayStation's samples loop on a block's
edge only, and a loop here must keep its length to the sample, which
is its pitch). A unit of 28 would save 2.7 % of the bytes for 0.9 dB
and make the channel's two blocks 112 bytes for 64. Nothing reads
XA's Level A from inside a module's clip, so its being a standard
buys nothing here. Kept as designed.

**More bits for the predictor, 2026-11-22** (the user asked). The
same coder with other predictors; signal over noise:

| | "Suspended Seconds" piano | "Starlight & Aurora" piano | its bells | Futurama, 8-bit residuals | 4-bit |
| --- | --- | --- | --- | --- | --- |
| the 7 fixed | 67.4 dB | 67.2 | 51.6 | 47.2 | 23.6 |
| 16 fitted to the sound, kept once | 71.1 | 70.2 | 59.6 | 49.1 | 25.5 |
| 64 fitted, a byte more a block | 71.4 | 70.3 | 59.5 | 49.2 | 25.6 |
| 256 fitted, a byte more a block | 71.4 | 70.4 | 59.5 | 49.2 | 25.6 |
| a block's own three, 3 bytes more a block | 68.9 | 68.3 | 58.1 | 48.8 | 25.1 |

So no more bits a block: a table of 16 fitted to the sound, which the
block's nibble already names, is the gain, 3 to 4 dB on a piano and 8
on the bells. Coefficients finer than 64ths gained nothing.

**On recordings, 2026-11-22** (the user: "Find a few 16-bit WAVs
online and use them for testing"). Futurama's theme as a speaker's
clip at 16 kHz: 41.8 dB at 8 bits, 47.2 predicted with the 7 fixed
(49.1 with a fitted table), 46.1 with CD-XA Level A's parameters;
IMA ADPCM by sox's encoder 15.3, the same predictors with residuals
of 4 bits 23.6. A dense mix pushed into a limiter is the worst case
there is for prediction. Nine recordings from Wikimedia Commons, the
Open Speech Repository and Espressif's samples (a carillon, a drum
loop, an electric guitar, a music track, an orchestra, an organ, a
piano, guitar chords, speech), 12 s of each, mono, the peak at
-0.5 dB: at 16 kHz 30.6 to 43.3 dB at 8 bits, 52.8 to 66.5 with
XA's parameters, 53.7 to 69.0 with the 7 fixed, 56.8 to 70.9 with a
fitted table; at 31 250 Hz 61.7 to 74.7, 64.3 to 80.3 and 66.4 to
81.7. Ours is over XA's on every one.

**A table a segment, 2026-11-22** (the user: "For WAVs we may want to
store filters per segment"; then "Try longer recordings of at least
60 seconds"). Nine recordings of 70 to 120 s at 16 kHz (a fiddle,
jazz, the music track, the orchestra, rock, a song, a symphony's
movement, a talk, and a medley of four of them, 20 s each), a table
for the whole against one every 30, 10, 4 and 1 s, residuals of 8
bits and of 4: every length within 0.4 dB of the one table, the
medley among them; a table a second is a little worse on the song,
having less to be fitted to. Sixteen predictors chosen anew every 16
samples already follow what a recording does. On the way: a table of
fitted predictors alone was 9 dB worse than the 7 fixed on the rock
and the medley at 8-bit residuals. At full scale a fitted predictor
can be further off than the largest residual reaches; with one entry
kept as "no prediction" it is not. The user: "Keeping one entry as
'no prediction' is important. Make sure it is not forgot. One table
per clip, no segments it is."

**The format as it is, built 2026-11-22** (the user: "Implement it as
a default format to convert to from 16-bit input files (or 16-bit
samples in case of MODs). Plain 16-bit PCM should be an option for
the convertor. Resample instruments for Suspended Seconds into
16-bit, same for Starlight & Aurora if it is not done yet").

- The blocks are as above, and the predictors are a table of 15
  written once (three coefficients each, `i16` in 64ths, 90 bytes),
  fitted to the sound by the converter. Predictor 0 is no prediction
  and is not in the table: the decoder has it whatever the table
  holds (`adpcm::coefficients`), so it cannot be fitted away or left
  out, and a test holds it. A coefficient is held to ±1023 as it is
  read, so three of them times a sample fit 32 bits.
- The fitting (`tools/pack/src/adpcm.rs`, `train`): each block's sums
  (its samples' products with the three before each), each block to
  the predictor that leaves the least of it, "no prediction" among
  them, each predictor fitted again to its blocks, 25 times, from the
  seven there were and blocks' own at even steps through the sound.
  Nothing random, plain `f64` arithmetic: one table for a sound, on
  the host and in the page. Then every predictor and shift is tried
  for each block, 144 pairs.
- A module (`XMC4`): a predicted sample is 8 bytes for the loop, the
  table, the blocks. It is what a 16-bit sample becomes unless told:
  `samples=16` keeps it as it is, `samples=8` cuts it.
- A sound file: `ADP2` is the magic, the rate, the number of
  samples, the table, the blocks; `PCM2` is the magic, the rate, the
  samples of 16 bits. Both are `adpcm::Decoder`'s, which the player
  and the page's preview already took their ADPCM from. `ADP2` is
  what a sound file becomes unless told (`format=packed`);
  `format=pcm16` is `PCM2`, `format=adpcm` and `format=pcm` what
  there was. The magics are of families a firmware up to v0.5 knows
  and versions it does not, so it refuses them with their pack.
- A sound file is therefore 17 KB of the pack a second where it was
  8, and a pack made with the defaults is one a clock at v0.5 or
  before refuses: the user's choice, and the manual and the page say
  what to give for such a clock.
- The tunes: "Suspended Seconds"' piano and all of "Starlight &
  Aurora"'s instruments are of 16 bits in their modules
  (`mods/src/`), and `clips/suspended_seconds.xmc` is made again:
  272 902 bytes for 256 361. Keygen's module has one 16-bit sample of
  169, which the default would predict; its clip is made with
  `samples=8` and is what it was, byte for byte.

Measured, host: the two tunes' mixes before the limiter against the
mix of 16-bit samples are 67.2 dB ("Suspended Seconds"; 43.7 at 8
bits) and 69.4 ("Starlight & Aurora"; 47.4); their clips for a
speaker 272 902 and 236 371 bytes (507 181 and 428 359 at 16 bits);
the buzzer's pack 744 432 bytes and the speaker's 980 867 of
1 179 648. Futurama as a speaker's clip: 469 948 bytes predicted,
884 408 at 16 bits, 221 108 as ADPCM, made in 1.1 s. The stack is
where the first build left it, 37 528. Every test, the firmware's
build and the lints pass; no clock has played any of it.

Not built: residuals of 4 bits, which with these predictors were 7
to 8 dB over IMA ADPCM by sox's encoder on Futurama at an eighth more
bytes. To weigh against the pack maker's own ADPCM encoder first.

**A 16-bit sample that is 8.** Keygen's module has one sample of 16
bits, its lead: 515 of the tune's 1445 notes, on 67 % of its rows.
Every one of its 169 samples is a multiple of 256, so it is an 8-bit
sample in a 16-bit one's place, and the clips made of it at 8 bits, at
16 and predicted are the same mix to the sample. The converter now
takes such a sample for what it is (`xmc.rs`: 16 bits with every low
byte nought are written as 8, whatever `samples=` says), so the
tune's clip is what it was with no option, and any module's like it
is one every firmware plays. And the same where the low bytes are all
one value that is not nought (the user: "The convertor should check
if the low byte is the same, not only zero. Dropping it would also
cost nothing"): that is an offset of under an 8-bit step, and it is
dropped, not rounded.

**The Grieg tunes at 16 bits, 2026-11-22** (the user: "Let's re-do
Grieg pieces with 16-bit samples"). Every sample of `morning.py` and
`mountain_king.py` is of 16 bits, both kits, the waves made in the
scripts among them, and the four modules and three clips are made
again. Mixes before the limiter against the mix of 16-bit samples,
and the clips' bytes for the buzzer:

| | at 8 bits | predicted | bytes at 8 bits | as made | at 16 bits |
| --- | --- | --- | --- | --- | --- |
| Morning, `bright` | 46.4 dB | 65.4 | 144 787 | 153 492 | 272 156 |
| Morning, `chip` | 46.5 | exact | 17 866 | 18 314 | 18 314 |
| the Mountain King, `bright` | 41.1 | 64.5 | 203 465 | 215 108 | 371 264 |
| the Mountain King, `chip` (in no pack) | 45.3 | 55.2 | 80 822 | 84 524 | 125 978 |

Morning's `chip` kit is seven waves of one period, 64 samples each:
predicted, each was 166 bytes for the 128 it is at 16 bits, the table
being 90 of them, and the clip was larger than the 16 bits' and not
exact (59.0 dB). So the converter keeps a sample at 16 bits where
predicting it would not be the smaller (`xmc.rs`), and that clip is
`XMC3`. The buzzer's pack is 765 228 bytes and the speaker's
1 001 663 of 1 179 648. Not heard.

**Seek points for 9xx, 2026-11-22.** A review's finding the user
brought: a 9xx on a predicted sample decodes from the sample's start
or its loop's block to where the note starts, 9FF being 65 280
samples, 4080 blocks, before the mixer has one sample: a burst far
over a chunk's budget. No clip the converter made could do it, an
instrument played with 9xx being kept at 16 bits; one from elsewhere
could. Of the local modules only `xm/children.xm` has 9xx, on three
8-bit samples. The user chose the variant that costs nothing where
no 9xx is: "Go with the second variant".

- A predicted sample of an instrument the module plays with 9xx has
  seek points (bit 2 of the sample's flags): after the table, for
  every 256 samples but the first, the three decoded samples before
  that block, each an `i16`, the newest first, the state the loop's
  head keeps for its own block. 6 bytes for 272 of blocks, 2.2 %.
- `refill` starts from the nearest of where the channel is, the
  loop's block, the seek point at or under the block asked for, and
  the sample's start. An offset is a multiple of 256 samples, so a
  9xx on such a sample decodes the two blocks any start does.
- The converter predicts such an instrument now, with its seek
  points, where it kept it at 16 bits: half the bytes. A 9xx on a
  cell that names no instrument still counts for every instrument.
- A predicted sample with no seek points that a clip from elsewhere
  hits with 9xx is decoded from its start, as before: slow, and
  right.

To check, host: a long predicted sample hit with 9FF, the blocks
`refill` decodes counted with the seek points and without, and the
mix against the same samples at 16 bits. On a clock: a module with
9xx on a 16-bit sample, the `clip:` line's time filling and its
underruns; there is none here to try.

Built the same day, host only. A sample of 70 000 hit with 9FF: the
note's first sample decodes 2 blocks with the seek points and 4082
without, the two-note tune 191 and 8335, and the mix is the 16 bits'
to the sample both ways. The six tunes of the player's test play the
same with seek points before the blocks as without. The stack is
37 440 bytes, 80 less, a channel's count of seek points. No clip in
the tree changed: none has a 9xx on a 16-bit sample.

**On test-clock, 2026-11-22** (`v0.5-rc5-9-gd982662`, flashed over USB
by the user, who heard both packs on it first: "На слух всё хорошо";
`/play`, each clip to its end, the serial log's `clip:` line, a `cat`
of the port. The speaker's pack with `output` on the line, then the
buzzer's, posted to `/clips`, on the buzzer):

| clip | speaker's pack, line | buzzer's pack, buzzer | share |
| --- | --- | --- | --- |
| Trantor | 1377 of 126 533 ms | 1379 of 126 535 | 1 % |
| Halftime | 11 122 of 43 571 | 11 122 of 43 569 | 26 % |
| Keygen | 14 177 of 57 646 | 14 170 of 57 641 | 25 % |
| Synchronization, cut short | 61 587 of 244 175 | 29 218 of 119 997 | 25 %, 24 % |
| Morning Mood | 76 033 of 104 039 | 76 134 of 104 043 | 73 % |
| Morning Mood Chip | 85 057 of 104 039 | 85 066 of 104 044 | 82 % |
| Mountain King | 75 518 of 139 591 | 75 507 of 139 593 | 54 % |
| Suspended Seconds | 22 453 of 48 044 | 22 425 of 48 041 | 47 % |
| Starlight & Aurora | 74 371 of 111 038 | | 67 % |

- No underruns and no stray positions in any, on either output; no
  warning in the log, the heap at 54 080 bytes, no gateway probe
  failed, the watchdog's count 0. The output costs nothing: the two
  columns are the same to a tenth of a percent.
- "Starlight & Aurora" predicted is cheaper than at 16 bits (80 of
  111 s on this board, 2026-11-20), and "Suspended Seconds" is what
  it was (23 of 48 s).
- Morning Mood Chip at 82 % is the dearest clip there has been. With
  it playing, `/api` asked every four seconds, 24 times: none failed,
  the median about 0.2 s, 22 under 0.41 s and two of 1.3 and 1.4 s;
  the watchdog's longest feed gap went from 1420 to 1500 ms
  (`DEADLINE_MS` is 10 s).
- What the prediction costs, from `packs/clips-default-speaker-8bit.bin`
  (made 2026-11-20, its modules' samples at 8 bits) on the same board
  and firmware, line output: Morning Mood 75 814 of 104 042 ms (73 %),
  Morning Mood Chip 72 996 of 104 044 (70 %), Mountain King 73 759 of
  139 598 (53 %), "Starlight & Aurora" 77 526 of 111 035 (70 %). So
  Morning and Mountain King cost nothing more predicted, and the chip
  Morning twelve points more. The chip Morning is not predicted: its
  seven instruments are waves of 64 samples, too short to repay a
  table, and are kept at 16 bits (`XMC3`; the other three modules are
  `XMC4` with every instrument predicted). So the twelve points are
  the 16-bit read's (`sample_at`: `word`, two byte reads a sample,
  two samples a mixed one for interpolation, twenty channels) and not
  a decoder's. A predicted sample is read from the channel's two
  blocks in RAM, one `i16`, which is why decoding costs no more than
  reading 8 bits from the flash did (the user's guess, 2026-11-22:
  reading the flash is dearer than decoding). How much of an 8-bit
  module's cost is cache misses is not measured.
- The chip Morning three ways from the one module, the same board,
  line output: `samples=8` 73 003 of 104 041 ms (70 %), `samples=16`
  85 026 of 104 046 (82 %), and with the converter's rule taken out
  for the try, so its 64-sample waves are predicted (`XMC4`, 18 580
  bytes, 266 more), 75 929 of 104 046 (73 %). No underruns in any. So
  the twelve points are the 16-bit read alone, and predicting a short
  sample too gives nine of them back for 38 bytes an instrument.
- Why the read costs so much: the mixer runs at twice the clip's
  rate, 62 500 times a second, over every sounding channel, so with
  twenty an instruction in the loop's body is some 0.8 % of the
  processor, and the 16-bit read is two more loads, two more bounds
  checks and the shifts for each of the two samples (the disassembly
  of `Player::level`, where `mix` is inlined). The 64-sample loop's
  wrap, a division, is some hundreds of times a second a channel and
  not it. One bounds check for the two bytes changed nothing for
  "Starlight & Aurora" on 2026-11-20, where few channels are wide.
- From Halftime's four channels and the chip Morning's twenty at 8
  bits, a channel is some 2.7 % of the processor, about 70 cycles a
  mixed sample, and what is not the channels' about 15 %.
- An aligned 16-bit load tried, 2026-11-23 (the user: is it the
  alignment, and "давай померяем lh в прошивке"). `sample_at` for a
  16-bit sample as one check and one `i16` read through a pointer
  (`unsafe`; the samples' offsets in the clip and the clips' in the
  pack are even, the partition's mapping too), which the compiler
  makes one `lh` where `word` is two `lbu`, two checks and the
  shifts: there was no unaligned load before, the two bytes were
  loaded apart. The trial over the air into `ota_1`, the same A/B
  pack, line output: `samples=16` 74 345 of 104 039 ms (71 %, from
  82 %), and `samples=8`, whose code did not change, 75 648 of
  104 043 (73 %, from 70 %). So the one load gives all twelve points
  back, a 16-bit sample then costing no more than an 8-bit one, and
  the same clip moves two or three points between two builds for no
  reason of its own. Not kept: the tree's `xm.rs` is as it was, the
  trial's diff is not in the tree.
- The flash's cache, from Espressif's documents (the IDF's "Memory
  Types" for the chip, its datasheet and memory map, read
  2026-11-23): what is mapped through the flash's MMU is read
  through a cache in internal SRAM, data as well as code, 16 KB,
  eight ways, read-only, and cached data is as fast as other internal
  memory. The pack's mapping is in the data bus's range, so the chip
  Morning's 896 bytes of samples are in it; what a large module's
  misses cost is still not measured.
- The cache counted, 2026-11-23 (the user: does a read fill a cache
  line each time, and are the two bytes of a sample pushed out
  between them). A trial build logs `EXTMEM`'s counters over the
  fills alone, with `CACHE_TRACE_ENA` set: accesses and misses of the
  data bus to the flash, misses of the instruction bus (its count of
  accesses stops at 2^32). The ROM gives the line as 32 bytes. Line
  output, each clip through, about 104 s:

  | clip | filling | data accesses | data misses | code misses |
  | --- | --- | --- | --- | --- |
  | chip Morning, 8 bits | 72 252 ms | 220.2 M | 35 742 | 121 114 |
  | chip Morning, 16 bits | 84 201 | 434.0 M | 35 431 | 105 365 |
  | chip Morning, predicted | 74 076 | 77.7 M | 50 933 | 184 469 |
  | Morning, 8 bits | 74 922 | 220.0 M | 1 216 493 | 439 662 |
  | Morning, predicted | 74 097 | 63.9 M | 1 397 580 | 628 522 |

  A line is filled on a miss and not on a read, and the misses are
  few: one read in 6000 for the chip Morning, one in 180 for Morning
  at 8 bits. The 16-bit read misses no more than the 8-bit one; it
  is 214 M more reads, as many again as the samples' at 8 bits
  (two a channel a mixed sample, some sixteen channels sounding), and
  12 s more: 56 ns, nine cycles, a read with its check and its shift.
  What the misses cost, from the two Mornings at 8 bits, the same
  count of reads: 1.5 M more misses, code and data, and 2.7 s more,
  under 2 us a miss if nothing else differs, so some 3 s of Morning's
  75. The code misses too, the more the larger the samples: it is in
  the flash and shares the 16 KB with them. The trial is not in the
  tree; the clock is back on `d982662` with the speaker's pack.
- The accessors, 2026-11-23 (the user: "Твои функции аксессоры
  по-сути и есть bottleneck, надо решать где-то там"). What was
  tried, on the bare bench board (`another-bench`, `/dev/ttyACM0`,
  no speaker; test-clock was unplugged), and what is kept.
  - Does the chip load unaligned (the user: "если данные не
    выровнены - проверяем работает ли без выравнивания")? `lhu` and
    `lw` in inline assembly against the bytes: every offset of 96
    bytes of RAM, of the flash's read-only data and of a clip, and
    the 32 bytes round an MMU page's edge, cache lines' edges among
    them: no load wrong, no exception. Over an empty loop's 50 ns:
    `lh` 6 to 12 ns at an even address, 12 to 15 at an odd one, two
    `lbu` and the shift 37 to 43; `lw` 6 aligned, 25 at an odd
    address. The clips of the pack that board had were at odd
    addresses.
  - With the compiler told so (`-Ctarget-feature=+unaligned-scalar-mem`,
    in one word, since riscv-rt's build script gives up on a
    `target-feature=` it takes for an ISA extension), a sample read
    from the channel's own bytes with `i16::from_le_bytes`, no
    `unsafe`: the chip Morning 65 to 66 % at 8 bits and at 16, at an
    even address and at an odd one, from 70 and 82 %. Not kept: the
    feature is unstable and the compiler warns it will be a hard
    error (rust-lang/rust#162235, a tracking issue: once
    `-Zllvm-target-feature` and build-std are stable; rustc knows
    the feature since #110884, nobody has asked for it to be stable
    in #150257, read 2026-11-23), and the predicted clips were no
    cheaper for it (the speaker's pack with the flag: Morning 72 %,
    Mountain King 53, "Starlight & Aurora" 66, "Suspended Seconds"
    46).
  - The two samples the mixer goes between with one load, tried
    with the flag (the user: "читай парами где надо"): 32 bits for a
    16-bit sample's two, 16 for an 8-bit one's. The four clips
    68 007, 67 371, 67 735 and 68 000 ms against 68 175, 67 554,
    68 226 and 68 176: half a second of 104 at most, under what two
    builds differ by. Not kept. The loads are a few cycles of a
    channel's hundred or so a mixed sample; what is left is the
    channel's state read and written each sample, the loop's and
    the end's tests, the ramp and the multiplies.
  - Kept (the user: align the samples and do without the feature,
    with no way round by bytes, 16-bit clips not being out yet). A
    channel keeps its sample, `held` for 8 bits and `words` for 16,
    and the mixer reads from that: one bounds check, against the
    sample's own end. `words` is `&[i16]` by `align_to`, the one
    `unsafe` of `core/`, its proof beside it; at an odd address it
    is empty and the sample silence. The converter puts a 16-bit
    sample at an even offset in its clip, the pack maker a clip with
    one at an even offset in the pack (a byte of 0 before it; the
    pack's version and layout are what they were, and a firmware of
    v0.5 reads the offset as it does any), `core/build.rs` aligns the
    clips built in, and `Pack::parse` skips a clip `xm::is_misplaced`
    names, as it does one of a version it does not play. A clip cut
    inside a 16-bit sample plays that half sample as 0, where it was
    its low byte.
  - Host: a tune of each width from an even address and an odd one
    (the 16 bits' silent at the odd one, the 8 bits' the same), one
    cut inside its last sample, a pack with the module after a clip
    of an odd length and the same with the module moved a byte; the
    196 tests of `core/`, with the clips built in as well, the pack
    maker's, both crates' clippy and the firmware's. The player's
    output before and after (`CLIP_PCM_DUMP`, `5ab7212` in a
    worktree) for the six module clips of `clips/`, the speaker's
    seven and the chip Morning at 8 and at 16 bits: the same, byte
    for byte, all fifteen. Both default packs come out byte for byte
    as they were: their 16-bit clip was at an even offset already.
    The main stack is 37 120 bytes, 320 less: the two slices.
  - On `another-bench`, each clip through, no underruns and no stray
    positions in any, heap 54 080, the watchdog's longest feed gap
    1262 ms. The chip Morning from one module, line output:

    | clip, its offset in the pack | `5ab7212` | kept |
    | --- | --- | --- |
    | 8 bits, even | 72 986 of 104 045 ms, 70 % | 67 692, 65 % |
    | 8 bits, odd | | 67 679, 65 % |
    | 16 bits, even | 85 017, 82 % | 66 400, 64 % |
    | 16 bits, odd | 85 035, 82 % | skipped: "4 clips … 1 more that this firmware does not play" |

    The default packs, as made by the new pack maker:

    | clip | speaker's pack, line | buzzer's pack, buzzer | share |
    | --- | --- | --- | --- |
    | Halftime | 10 597 of 43 567 ms | 10 596 of 43 565 | 24 % |
    | Keygen | 13 525 of 57 645 | 13 539 of 57 644 | 23 % |
    | Morning Mood | 75 418 of 104 043 | 75 465 of 104 043 | 72 % |
    | Morning Mood Chip | 66 417 of 104 040 | 66 432 of 104 042 | 64 % |
    | Mountain King | 74 763 of 139 595 | 74 735 of 139 597 | 54 % |
    | Suspended Seconds | 22 270 of 48 039 | 22 244 of 48 038 | 46 % |
    | Starlight & Aurora | 73 616 of 111 042 | | 66 % |
    | Synchronization, cut short | 29 482 of 120 000 | | 25 % |

    Morning Mood, predicted, is the dearest clip now.

    Before and after on that board, the speaker's pack on the line
    output (the same file both times), `5ab7212` flashed for the one
    and the commit of this for the other; no underruns either way:

    | clip | `5ab7212` | after | |
    | --- | --- | --- | --- |
    | Trantor | 1374 of 126 530 ms | 1399 of 126 530 | 1 % |
    | Halftime | 11 111 of 43 569 | 10 597 of 43 567 | 25.5 → 24.3 % |
    | Keygen | 14 158 of 57 641 | 13 525 of 57 645 | 24.6 → 23.5 % |
    | Morning Mood | 76 029 of 104 045 | 75 418 of 104 043 | 73.1 → 72.5 % |
    | Morning Mood Chip | 85 041 of 104 039 | 66 417 of 104 040 | 81.7 → 63.8 % |
    | Mountain King | 75 504 of 139 599 | 74 763 of 139 595 | 54.1 → 53.6 % |
    | Suspended Seconds | 22 450 of 48 039 | 22 270 of 48 039 | 46.7 → 46.4 % |
    | Starlight & Aurora | 74 362 of 111 039 | 73 616 of 111 042 | 67.0 → 66.3 % |
    | Synchronization, cut at 120 s | 29 174 of 120 001 | 29 482 of 120 000 | 24.3 → 24.6 % |

    The modules of 8-bit samples are a point cheaper, the predicted
    ones half a point, which is under what two builds differ by; the
    PT3 and the beeper clip are not this change's.
  - A predicted sample's coefficients kept on the channel (the user,
    2026-11-23), counted on the host and not built for the board: a
    block has the predictor of the one before it in 21 % of Morning
    Mood's blocks, 33 % of Mountain King's, 46 % of "Suspended
    Seconds"' and 50 % of "Starlight & Aurora"'s, and Morning decodes
    24 000 blocks a second, so the three coefficients read anew each
    block (three checks, six byte loads, three clamps) are under a
    percent of the processor by the code's count. Not done.
  - The player's code in RAM, tried 2026-11-23 (the user: "попробуем
    перенести код микшера в SRAM"), not kept yet: `.rwtext` by a
    `link_section` on `Player::level` (the mixer and the players are
    inlined in it, 7.5 KB), `fill_samples`, `fill_bits`,
    `xm::refill`, `xm::fill`, `adpcm::decode_block` and
    `Shaper::run`, 10 KB in all; the tick's code (a cell read, a note
    taken, the envelope) stays in the flash, and the data. The main
    stack is 27.6 KB with it, from 37.1. The speaker's pack on the
    line output, the same board, against the table above's "after":

    | clip | code in the flash | in RAM | |
    | --- | --- | --- | --- |
    | Halftime | 10 597 of 43 567 ms | 10 402 | 24.3 → 23.9 % |
    | Keygen | 13 525 of 57 645 | 13 262 | 23.5 → 23.0 % |
    | Morning Mood | 75 418 of 104 043 | 72 693 | 72.5 → 69.9 % |
    | Morning Mood Chip | 66 417 of 104 040 | 64 766 | 63.8 → 62.3 % |
    | Mountain King | 74 763 of 139 595 | 71 986 | 53.6 → 51.6 % |
    | Suspended Seconds | 22 270 of 48 039 | 21 463 | 46.4 → 44.7 % |
    | Starlight & Aurora | 73 616 of 111 042 | 71 587 | 66.3 → 64.5 % |

    No underruns, the watchdog's longest feed gap 1340 ms. Every clip
    is cheaper, the ones with the most samples the most, as the
    cache's counts said: about what the misses were reckoned to cost.
    How much of the main stack is used was not measured.

    The same with less of it in RAM (the user: SID and AY out of
    `Player::level`, as functions of their own in the flash): the two
    players (`sid_sample` 2 KB, `ay_sample` 1.3 KB) and the module's
    tick (`step_tick`, 3 KB) out of line and in the flash, the
    module's `next_sample` with the mixer in RAM. 3.5 KB in RAM, the
    main stack 33.6 KB: Halftime 10 602, Keygen 13 510, Morning Mood
    73 649 (70.8 %), the chip Morning 65 199, Mountain King 73 136,
    "Suspended Seconds" 21 843, "Starlight & Aurora" 72 257, and
    Synchronization, the PT3 through its function, 29 749 of 120 002
    (24.8 %, as it was). Some six tenths of the larger trial's gain
    for the large clips and none for the two small ones. SID and AY
    are not run by these clips, so what was lost is the tick's: 3 KB
    of code called some tens of times a second, which the samples
    push out of the cache between calls, by this reckoning; not
    counted.

    Two more, the same clips. With `step_tick` in RAM as well (6.4 KB
    there, the stack 30.7 KB): 10 517, 13 417, 73 160, 65 052,
    72 550, 21 635, 72 046. And with what the tick calls and
    `Stream::fill` (`trigger`, `read_cell`, `enter`, `sounding_now`,
    `Instrument::point`, `long`, the four of `refs`; 9.4 KB, the
    stack 27.7 KB; only the SID's and the AY's players are called in
    the flash from there): 10 470, 13 360, 72 880, 64 948, 72 114,
    21 505, 71 791. Morning Mood, then: 75 418 in the flash, 73 649
    with 3.5 KB in RAM, 73 160 with 6.4, 72 880 with 9.4, and 72 693
    with the first trial's 9.5 that had the other players in it:
    each 3 KB more buys less, 1.8 s, 0.5 and 0.3 of 104. No underruns
    in any, the watchdog's longest feed gap 1260 ms.
  - Kept of these (the user, 2026-11-23: "Берём вариант с 3.5кб"):
    the first of the smaller ones, as a feature of `core`, `ram`,
    that the firmware turns on: a `link_section` of `.rwtext` on the
    eight functions, on a target with no OS. The image has the same
    functions in RAM as the trial had, `.rwtext` 8032 bytes from
    4560, the main stack 33 648. On the board: Morning Mood 73 651
    of 104 042 ms, the chip Morning 65 201 of 104 045, as the trial
    had them, and Synchronization, the PT3 through its function,
    plays. How much of that stack is used is still not measured.
  - Owed: a clip heard (the board has no speaker and no buzzer), and
    the same on a clock with a panel.
- test-clock is dearer than the bare board for the same tune at 8
  bits (Morning 73 % against 66 %, the chip 70 against 64, Mountain
  King 53 against 48, 2026-11-19), a panel and its tasks being there.
- The board was left with the speaker's pack, as found, `output` on
  the buzzer and the melody Morning Mood, as found (the melody put back by
  hand: a pack without the chosen clip, as the A/B packs were, leaves
  it on the beeps), and on the firmware as found, built again from
  `d982662` and sent over the air after the trial: it is in `ota_0`.

## Checks still owed

- 16 bits predicted, on a clock (2026-11-22, on `next`, above; both
  default packs timed and heard on test-clock that day): a sound file
  made with the defaults.
- The module player's samples read from the channel's own slices,
  16 bits as `&[i16]` (2026-11-23, above; built and timed on the
  bare bench board): a clip heard, and the figures on a clock with a
  panel.

- The pack maker (2026-11-19): the release workflow's three builds of
  the command (Linux, Windows, macOS) and the deploy of `docs/pack/`
  with its wasm, which run only with a tag; the command in a user's
  hands.

Built, and by its heading in `HISTORY.md` not yet seen, heard or run
where it counts. Each line is that heading as it stood on 2026-11-17,
when the history was split off; strike or delete a line when the check
is made or is no longer wanted, and put the result in the section.

- **Hold the self-heal while the alarm rings, and ring for five minutes (after the brightness ceiling)** — built, not yet verified on the panel.
- **More tunes, and a random pick (after `/reload`)** — random verified on the bench 2026-10-28; the tunes retuned after two listens, a third owed.
- **M11a — read both sensors and show them on the page and in `/api`** — built 2026-11-01; the NTC verified on the bench the same day, the LDR read the rail (see the end).
- **Codex review of 2026-11-07: eleven findings fixed** — built 2026-11-07, released as v0.3 and soaked 11 h on the bench 2026-11-08; the fixes' own checks still owed.
- **ZX Spectrum tunes: an AY-3-8910 and a PT3 player on the chip** — built 2026-11-13, on white-clock to be heard; the register comparison with a reference player owed.
- **A Stop button beside Play** — built 2026-11-13, to be tried on white-clock.
- **Quieter noise for the AY's drums** — built 2026-11-13, an A/B/C on white-clock to be heard.
- **A low-pass for the AY's noise** — built 2026-11-13, an A/B/C on white-clock to be heard.
- **Bass cut for a speaker: an A/B on headphones** — built 2026-11-13, on test-clock to be heard.
- **Where a module's overdrive comes from: the limiter's release** — measured on the host 2026-11-13, the A/B on test-clock's headphones to be heard.
- **PT3 tunes flat, for headphones** — built 2026-11-13, to be heard on test-clock.
- **The station picks its access point: the strongest of each scan** — built 2026-11-13, not on a clock yet.
- **The page split in four: clock, settings, Home Assistant, system** — built 2026-11-14, to be tried on white-clock.
- **The thermometer held while Q1 is hot** — measured on white-clock 2026-11-14, the hold built and run there 2026-11-15; a short preview and the serial log's lines not yet seen.
- **The pages after a UX review: PIN at the click, a saved notice, the bad field named** — built 2026-11-15, run on test-clock in headless Chromium; the setup form the user's, 2026-11-15; the setup form and a real upload from the page run on test-clock 2026-11-17 ("An install from nothing").
- **PCM clips at half the flash: IMA ADPCM, `ADP1`** — the A/B heard on white-clock 2026-11-16 ("not bad", "way better than PCM 8k"); the format built 2026-11-16 and heard by the user on a speaker the same day ("audible, but acceptable"); the serial log's clip line read on test-clock; the buzzer through the clock's own decoder not yet.
- **A clip of a version no player reads is refused** — 2026-11-17, built and tested on the host; no upload of such a pack tried on a clock.
- **A watchdog** — built and every reset path run on test-clock 2026-11-17; soaked 24 h on the bare bench board 2026-11-19 with the longest feed gap 1180 ms through clips, saves and rings, and no trip at either upload (the gap during an upload itself cannot be read: the clock resets). On a clock with a panel, not yet.
- **Forward compatibility before the first public release** — 2026-11-17, built, tested on the host and on test-clock; not committed, not tagged.
- **Sixteen channels for the module player** — built, measured on test-clock and heard 2026-11-18; the 31 250 clip as a ringing alarm for five minutes not run, and the version line of a changed tree's build lacks `-dirty`.

Headings that still say planned or not run, though the work is in
the tree or was overtaken; nothing is owed but the heading's wording,
unless the user knows otherwise:

- **M11 — NTC thermometer and light sensor** — the readings built 2026-11-01 (M11a, below), the face and the dimming still planned (the face is "M11b, first half", done; the dimming is above).
- **Web flasher and releases** — planned 2026-11-07 (`docs/index.html`, `release.yml` and four releases are in the tree).
- **Timezone rules: POSIX TZ strings and daylight saving** — planned 2026-11-07 (`core/src/tz.rs` is in the tree).
- **Scan restart gap: measure the whole dark time** — built 2026-11-07, not yet verified on the bench (the restart it measures went with the ring).
- **Scan ring spike: the segmented transfer on the bench** — built 2026-11-08, not yet run (the ring is the scan path since "Hardware-chained scan").
- **Tracker modules (XM)** — first trial as a rendered clip 2026-11-12; a player on the chip only planned (`core/src/xm.rs` is in the tree).

## Review backlog

The findings of the four review passes that were open when the
history was split off, each as it was written, line numbers and all
(they are of the commit the pass read). Each pass's account of what it
read and what checked out, and the items closed, are under the same
headings in `HISTORY.md`. Strike an item when it is done or decided
against, with the date.

Every item was read against the code on 2026-11-17 (commit c08b6cd).
Four were found done and are struck; an item with a "Checked" note
has changed in part; the rest hold as written, in the functions they
name, at other line numbers.

### Review backlog (2026-10-21)

- ~~**Spaced day lists.**~~ Found done 2026-11-17: `form::apply` decodes a
  value into `VALUE_MAX` (64) bytes, so the 33-byte list reaches
  `parse_days`; no test puts the full spaced list through `apply`.
  `parse_days` trims each name, but the 32-byte
  value buffer in `form::apply` is one byte short of
  `mon, tue, wed, thu, fri, sat, sun`, so the API answers "bad value"
  before the trim runs. Raise the buffer to 40 or so.
- **Build-time timezone range.** `TZ_OFFSET_MINUTES` is cast to i16 in
  `config.rs` without a range check, so a wild value wraps silently and
  `Settings::DEFAULT` is never run through `is_valid`. A const assert
  against `TZ_LIMIT_MINUTES` catches it at compile time.

### Review backlog, second pass (2026-10-23)

Bugs, in rough order of value:

- **Self-heal is unreachable once the lease expires on a dead uplink.**
  `probe_task` starts every iteration with `stack.wait_config_up()`
  (`net.rs:328`). When the uplink dies with less lease time left than
  the five-minute budget, smoltcp's DHCP client resets at `expires_at`
  and embassy-net clears the address (`lib.rs:907-909`); from then on
  no probe is sent, the streak is frozen below ten, the link is still
  up so `wifi_task` sees nothing, and the connect budget needs connect
  failures that never come. The clock sits with the carried time and
  no page until power is cut. With a one-hour lease that is about one
  uplink death in twelve. Fix: bound the wait (`with_timeout(PROBE_PERIOD,
  wait_config_up())`) and, once armed, count a period with the link up
  and no address as a failed probe. Left as is, 2026-11-02: a corner case
  that needs a link to die in the last minutes of its lease, while a
  marginal link trips the probe budget well before that. The fix, if
  it is ever wanted, is the bounded wait above with the no-address
  period scored like a lost ping, and the streak logic moved into
  `core` under test.
- **`POST /play` during an alarm silences it.** `buzzer_task`'s `Ring`
  loop breaks on any command and the `Preview` arm plays once and then
  waits for the next command; the display re-issues `ring` only on the
  ringing edge. One preview, from a browser tab left on the page, say,
  leaves the panel flashing in silence for the rest of the 120 s. Fix:
  ignore `Preview` while ringing, or go back into the ring loop after
  it.

Gaps:

- ~~**No watchdog.**~~ Built 2026-11-17 ("A watchdog", above), with a
  minute's timeout, not a few seconds, and run on test-clock.
  `esp_hal::init` disables the SWD, RWDT and both
  timer-group watchdogs and nothing re-enables one. A hang that is not a
  panic (a lockup in the Wi-Fi driver, a critical section waiting on
  something, an infinite loop in a freshly uploaded image) leaves the
  panel dark until someone opens the case, and the OTA rollback counts
  only software resets, so a new image that hangs is never rolled back,
  contrary to `ota.rs`'s header. Fix: enable the RWDT with a few-second
  timeout fed from the heartbeat or the display tick, and keep
  `ota_boots` in `carry::init` on a watchdog reset reason.
- **A PIN issued while ringing or in alarm-set is dropped, not deferred.**
  `ui.show_pin` acts only in the clock, IP and PIN modes, but the
  `Signal` is consumed either way and `ota.rs:218` logs "shown on the
  panel" regardless; the user at the update page has nothing to type
  until they reload. Fix: keep the pending PIN in `Ui` and show it when
  the mode returns to clock within `PIN_TTL_MS`, or log that it was not
  shown.
- **The connect budget is cleared by an association that does not
  survive.** `failures` and `first_failure` reset on any `connect_async`
  success. An access point that associates and drops the station before
  DHCP finishes (MAC filter, client isolation, a flaky AP) gives a loop
  of connect-ok, disconnect, retry in which neither budget fires and the
  probe never arms. Fix: clear the budget only once DHCP has an address
  or the connection has lasted some tens of seconds.

Nits:

- **`scan::stop` is not honoured on the DMA start-failure path.** The
  `Err` branch sleeps and `continue`s past the `STOP` check, so if the
  start kept failing `heal::reset` and `ota::install` would spin for
  ever. Unreachable with esp-hal 1.2.2's constant error conditions;
  a `STOP` check before the retry keeps the invariant.
- ~~**Oversized or unparseable requests get a TCP reset, not a status.**~~
  Fixed 2026-11-17 ("The install run's findings fixed").
  `read_request` returns `None` for a head over `REQUEST_BUF`, a body
  over `BODY_MAX` or a bad request line, and the socket is closed with
  bytes pending. Answer 413/400 after a bounded drain, as `/ota` does.
  Checked 2026-11-17: a `Transfer-Encoding` now gets a 400 and a
  `POST` with no length a 411; the three cases above are still a
  silent close (`NoRequest::Close`).
- **First change on an empty flash writes the default record.** With
  `stored == None` the compare in `persist_task` is skipped, so ticking
  "skip the next alarm" on a fresh board costs a flash write for a field
  the record does not hold. Compare against the defaults when nothing
  was loaded.
- **A persistent `set_config` failure loops outside the connect budget.**
  The error path at `net.rs:162-166` sleeps and retries without touching
  `failures`. Static errors are caught earlier, so unlikely to matter.
- **`FIRMWARE_VERSION` goes into JSON and HTML unescaped.** It is
  `git describe`, so a tag with `"` or `<` breaks `/api` and the page.
  Sanitise in `build.rs`.
- **Silent fallbacks in the panel tables.** `com_index` returns slot 0
  for a value not in `COM_SEQ`, and nothing ties `GAMMA[255]` to
  `MAX_LEVEL`. Two `const` asserts lock in what is verbatim today.
  Checked 2026-11-17: `GAMMA` went with the colours in levels; the
  `com_index` half holds.
- **In the ringing off-phase the degree mark stays lit.** `set_degree`
  runs before the match, so the flash is digits-off, mark-on. Cosmetic.
- ~~**Stale names.**~~ Found done 2026-11-17: `check_rollback` is named
  nowhere, and the image has grown to nearly 900 KB, so the 800 KB in
  `http.rs` is no longer too much. `ota.rs:12` and `carry.rs:49` refer to
  `ota::check_rollback`, which is `ota::init`; `http.rs:29` says an
  upload is 800 KB where the image is about 680 KB; the M10 text said
  two `RECEIVERS` where there are three (fixed in place).

### Review backlog, fourth pass (2026-10-30): the code since the second pass

Bugs, in rough order of value:

- **`POST /scan` can only answer within the socket's 5 s idle
  timeout.** `rescan` waits up to `SCAN_WAIT` (10 s) for the AP task's
  scan, during which the browser sends nothing, and smoltcp's socket
  timeout is an idle timeout: `remote_last_ts` moves on every received
  segment and `dispatch` closes the socket at `remote_last_ts +
  timeout` with no regard to pending transmit data. A scan slower than
  5 s (13 channels at up to 200 ms is about 2.6 s today) answers into a
  closed socket and the phone shows a reset instead of the page. Fix:
  raise the timeout for that request, or cut `SCAN_WAIT` under
  `TIMEOUT` and say so in the comment.

Gaps:

- ~~**A record the map scratch cannot hold wipes every setting on
  rollback.**~~ Found done 2026-11-17 (the forward-compatibility work of
  that day): the scratch is 1024 with `ITEM_MAX` under it, and
  `BufferTooSmall` runs on the defaults without erasing; by design
  the first save after that does erase the map. `Store::load` erases the whole map on any `Err` from
  `fetch_item`, but sequential-storage returns `BufferTooSmall` before
  anything is corrupt when an item does not fit (`Corrupted` is already
  repaired inside `fetch_item`). The scratch is 256 bytes and a
  version-6 item is 176 (8-byte header, 1-byte key, 167 of record; the
  9-byte claim holds); the record grew 99 bytes in the last version, so
  one more set of fields of that order (a second network) takes a
  version-7 item past 256. An image with that record saves it, the
  `OTA_BOOT_LIMIT` rollback boots this one, `load` logs "cannot read the
  map, erasing it", and the clock comes up with hostname `clock`,
  timezone 0 and no network, the exact case the newer-prefix read was
  built to prevent. Fix: erase only on `Corrupted`, treat
  `BufferTooSmall` as "not understood, using defaults" without erasing,
  and size the scratch to a page.
- ~~**A string-rule change in a newer image throws the record away on
  rollback.**~~ Found done 2026-11-17 (the same work): a network or a
  token `from_record` does not accept is that field's default and the
  rest is kept. `from_record` clamps `color`, `melody` and `brightness`
  for exactly this case, but the version-6 strings go through
  `is_valid`, so a password of 1 to 7 bytes or a token outside
  `valid_token`, both legal in a later image that relaxed the rule,
  make this image discard the whole record. Fix: a bad string field
  falls back to its default (`Credentials::NONE`, empty token) and the
  rest is kept.
- ~~**`/api` on the access point saves a password with no network.**~~
  Fixed 2026-11-17 ("The install run's findings fixed").
  Since 2026-10-30 the persist task resets off the access point only
  with a network stored, so this is a pointless save and no reset;
  the guard on `/api` is still missing. The
  `trial.wifi.is_none()` guard is only on the form path; `POST /api
  password=…` on an unprovisioned access point saves, differs from
  `boot_wifi`, and the persist task resets into station mode with an
  empty SSID, which the boot turns back into an access point. A
  harmless loop, but the script sees "saved" and a reset for nothing.
  Fix: the same guard on `/api` in access-point mode.
- ~~**The OTA rollback carries a stale time.**~~ Fixed 2026-11-17 with
  the watchdog: `restore_carried` runs before `ota::init`. `ota::init` runs before
  `clock::restore_carried` in `main.rs`, so its `carry_time()` is a
  no-op (`unix_micros` is `None`) and the record keeps the time written
  at the end of the previous boot; the restore then adds `RESET_PAUSE`
  to a value older by that boot's span up to `ota::init` plus its own
  pause, about a second, shown until NTP. Fix: `restore_carried` before
  `ota::init`.
- **Without a usable `nvs` partition a provisioning save never leaves
  the access point.** The reset into station mode lives in
  `persist_task`, spawned only when `Store::open` succeeded; with no
  store the setup page still says the clock will reset and nothing
  does, while the unchanged-credentials path does reset. A bad
  partition table only; answer differently or reset from the listener
  when there is no store. The same shape as the open "failed save is
  never retried": on the access point a failed save shows the saved
  page and stays an access point until the next change.
  Checked 2026-11-17: a failed save is retried now (`persist_task`'s
  backoff), so only the no-store case is left.
- **The access point's transmit ceiling is set before the driver is
  restarted.** `ap_task` calls `set_max_tx_power` and then
  `set_config(AccessPointStation)`; `WifiController::new` has already
  started the driver as a station, so that `set_config` is a mode
  change, which stops and restarts the driver (`esp_wifi_stop`,
  `esp_wifi_set_mode`, `esp_wifi_start` in esp-radio). ESP-IDF
  documents the power call as "after WiFi start"; whether the value
  survives the restart is inside the blob and was not verified. The
  station path has no restart. Fix: move the call after `set_config`,
  so the boot line says what is in force.

Nits:

- **The display loop wakes every 64 ms for nothing on the access point
  and while a PIN shows.** The redraw wait is `BLINK_STEP / 4` for every
  view but `Time` and `Ip`, right for the blink and the sweep, but
  `View::Sweep` on the access point is a static `AP` for the whole boot
  and `View::Pin` is static for its minute. Each wake runs
  `carry_time`, a `settings::update` under the watch's critical section
  (which holds off the interrupt executor for its duration), a render
  and the image compare. Fix: wait `PIN_SHOW` for the PIN and a second
  for the sweep when `net::mode()` is the access point.
- **`Store::open` logs inside the flash critical section.** The
  `error!` lines for a missing partition table or `nvs` entry run
  inside `with_flash`'s `critical_section::with`, so the serial write
  stalls everything. Boot-only, before the scan starts; move the match
  outside the closure.
- **A discover from a client that already holds a lease cuts the lease
  to the offer hold.** `Leases::handle` writes `now + OFFER_HOLD_SECS`
  (60 s) over an existing slot when `find` hits. A phone that
  re-discovers and then does not request keeps using the address while
  the slot is free again after a minute and can be offered to the next
  client. Fix: on a discover, keep the later of the two expiries.
- **`dnsd::answer` indexes `out` by the query's length.** `out[HEADER..n]`
  with `n` up to `query.len()` is only checked against `PACKET`; a
  caller passing a query longer than `out` panics. The task's buffers
  are both 512 so it cannot happen on the air; a host test feeding a
  long query would hit it. Fix: `get_mut(HEADER..n)?` like the record
  copy below it.
- **`REQUEST_BUF` is tight for the setup form from a phone.** The head
  and the inline body share 1536 bytes; the setup form at its worst is
  about 457 bytes encoded (two 32-byte SSIDs and a 64-byte password,
  percent-escaped), leaving about 1080 for the head, and a mobile
  browser's POST head to a captive page is about 750 bytes before any
  cookie another gadget at 192.168.4.1 may have set. Over the limit is
  the silent close of the open "oversized requests" item, on the one
  page whose user cannot see the log.
  Since 2026-11-17 over the limit is a `431` the browser shows; the
  room is what it was.
  Checked 2026-11-17: the form also posts the timezone, the hostname
  and the transmit power now, and a body was refused over `BODY_MAX`
  (512) whatever the head left. Measured by a test the same day
  (`forms_fit_the_request_body`): 605 bytes with both names, the
  password and the timezone full and every byte escaped, so
  `BODY_MAX` is 640 now (built and tested on the host, not on a
  clock). What is left of the item is the head: at 605 of body it has
  931 bytes of the 1536, against the 750 or so given above for a
  phone.
- **No error from `ap_task` is ever healed.** A failed `set_config`
  logs once and sleeps for ever; the panel shows `AP` with no network
  and only a power cycle or a hold helps. Unlikely with the
  compile-time-checked SSID and password; a log line is probably all
  that is worth doing.
- **Stale comments.** `net::Mode`'s doc says the driver does not do
  AP+STA, which `ap_task` does for the scan; `http.rs` still says an
  upload is 800 KB where the image is about 650 KB (the second pass's
  `check_rollback` half is fixed).
  Checked 2026-11-17: the `net::Mode` half holds; the image is nearly
  900 KB now, so the 800 KB one does not.
