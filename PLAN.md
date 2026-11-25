# onebitdawn — plan

What is not done: the list for the first public release, the work
planned and not built, the checks a built change still owes, and the
open findings of the reviews. Work is designed here before it is
started, with what was measured and learned added as it goes; a
finished section moves to `HISTORY.md`, the record of every milestone
so far. `ARCHITECTURE.md` is the firmware as it is.

## To do before the first public release — a list, 2026-11-17

- [x] Screenshots of the web interface: six in `screenshots/`, from
  white-clock by headless Chromium, the network, the broker and its
  user blurred (the user's choice); retaken 2026-11-20 on `v0.5-rc3`.
- [x] Pictures of the board with the buzzer mod, in `pics/` (the
  user, 2026-11-17), retaken without RBE and the BAT42, which the
  scope showed change nothing: the mod is the diode and R5.
- [ ] `CHANGELOG.md`: `[Unreleased]` made the version's section, the
  compare links under it.
- [x] An install of an image with no token, the access point's setup,
  then a token set with the PIN: test-clock, 2026-11-17 ("An install
  from nothing", below). The web installer itself is the last item's.
- [ ] The default pack heard on a stock buzzer circuit (no diode, R5
  at 10 kΩ), and the README's wording on the mod by what is heard.
  Heard so far: a modded buzzer and a speaker, both good.
- [x] The installer no longer offers v0.1 to v0.4 (`OLD` in
  `docs/index.html`); live with the next tag.
- [x] A pack maker a user can use, one command and one web page:
  tried by the user on white-clock 2026-11-19. Its release builds run
  with the tag.
- [ ] The annotated tag; then the release's three files and one
  install from the live page, since the workflow has not run with the
  pack and the new names.

## Planned, not built

### The predicted samples' code built for speed, a function at a time — put off 2026-11-23

`core` at `opt-level` 3 makes clips with predicted samples 7 to 8 %
cheaper for 1.9 KB of the main stack; 2 gains next to nothing. The
user refused a crate of its own for it: when `#[optimize(speed)]` is
stable, put it on `adpcm::decode_block`, `xm::fill` and `xm::refill`,
measure, then look at `Tune::render`. A try on those three
(2026-11-23, `RUSTC_BOOTSTRAP`, nothing kept) gave 1.2 to 1.6 % for
192 bytes of `.rwtext`: the decoder's loop is 22 instructions a
sample at `s`, which 3 only unrolls. In November 2026 `speed` was
being taken out of the attribute's stabilization (rust-lang/rust,
issues #157273, #165590, #165592): look before relying on it.

Owed: the attributes and the measurement, once `speed` is stable.

### Dim in the dark (M11b, second half) — planned 2026-11-01, not built

The rest of M11 in `HISTORY.md`; the sensors are read and the light
index is on the page and in `/api`. The record has no field yet.

One boolean setting, `dim_in_dark`, key `dim`, labelled "Dim in the
dark". When it is on and the room is dark the panel runs at the
lowest brightness step, otherwise at the user's. Dark begins when the
10 s light average falls below `DARK_BELOW` and ends when it rises
above `DARK_ABOVE`, two constants in `config.rs` far enough apart
that a room light clears the gap and a candle or a passing car does
not flip it; first values guessed, tuned on the panel from the
heartbeat's millivolts. The user's step is untouched (the press
cycles it, the record stores it): the display task substitutes the
lowest step at draw time. `IS` comes from the step drawn, cap and
all.

Owed: all of it.

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

### A line output on the adapter for an amplifier — the setting built and heard 2026-11-18; the shutdown pin built, not checked; the adapters ordered 2026-11-19

The user's idea (2026-11-15): an amplifier in place of Q1, fed the
converter's bits through a filter on the adapter.

**Design (the user's): a pin of its own and a setting**, so one image
serves both kinds of clock and a wrong setting is silence, never a
buzzer held on.

- GPIO20, push-pull, low at rest; GPIO5 stays open-drain to Q1.
  Rejected: GPIO10 (the module's LED, which the bits would light),
  GPIO2 (a strapping pin the filter would hold low), GPIO21 (UART0's
  TX, the ROM's boot text), 18 and 19 (USB). GPIO20 is UART0's RX,
  unused: a pull-up while the ROM runs, so one step to 3.3 V at a
  reset.
- The filter and level are in `bench/pdm-amp.md`, from the datasheets
  (CJMCU-832, PAM8403): 2.2 kΩ / 4.7 nF, 10 kΩ / 4.7 nF, 3.3 kΩ to
  ground. A full-scale clip is about 1 V rms off the pin; the
  amplifier clips at 0.2 V rms (0.62 V peak to peak at 5 V). An
  earlier sketch (1 kΩ / 10 nF, 10 kΩ / 1 nF, 0.1 V rms) was wrong on
  both counts.
- 3V3 is the converter's reference, so the Wi-Fi's bursts on it are
  in the audio; if heard, a single-gate buffer from a filtered supply
  is the fix.

**Built 2026-11-18**: `output` (`settings::AudioOut`, record version
13), on `/settings` and `/api`, not in `mqtt::SET_KEYS`; `buzzer` is
the default and what an older record reads as.

- Read at every tone and clip, not at boot (the user's request), so a
  change takes effect at the next sound, with no reset.
- `buzzer::Pins` is the one place that knows each pin's drive and
  rest. LEDC stays on the last tone's pin at the silent duty, so a
  tone going to the other pin first takes the old one to its rest.
  The inverter and the thermometer's hold (`clip::converting`) are
  the buzzer's only.
- GPIO20 is driven low at boot only if the stored setting sends clips
  there; a clock switched later has it driven at its first sound, one
  step from the ROM's pull-up.
- `split` (the user's): notes on the buzzer, clips on the amplifier.
  Record byte 2, no new version; an older image reads it as `buzzer`.
  The setting answers a `settings::SoundPin` per kind of sound
  (`AudioOut::tones()`, `AudioOut::clips()`), so nothing can ask for
  GPIO5 push-pull. The page's option is "Notes: buzzer, clips: amp".
- A converter clip cut short on the line ends on a ramp one chunk
  long, written two chunks ahead of the DMA (`clip::fade`, under
  test), where it had clicked at Stop; Stop is about 0.1 s later
  there at 16 kHz.

**Heard 2026-11-18** (the user), test-clock, breadboard, a PAM8403
module with a 50 kΩ pot and the speaker mod's speaker; the scope
showed clips as audio and beeps as a square wave. In an A/B against
the S8550 stage at matched loudness the amplifier was noticeably
cleaner, by ear; a guess, untested, is that the transistor folds the
converter's shaped noise into the band ahead of any filter. A slight
background noise, taken for Wi-Fi pickup, source not told apart. No
click at Stop with the ramp. `split` worked on white-clock.

**The amplifier shut down between sounds** (the user's, 2026-11-18;
firmware built, not flashed): against very quiet clicks the amplifier
picks up from the Wi-Fi at rest.

- GPIO10, high is on: the PAM8302A's SD (active low) on the pin. The
  user gave up the module's LED; on the Zero it is addressable with
  GPIO10 its data input. Datasheet: SD high from 1.2 V, low to 0.4 V,
  under 1 µA shut down, SD to rise 1 to 100 ms after the supply, and
  shut down before the supply comes or goes for the least pop; no
  turn-on time given.
- Adafruit's board pulls SD to the supply with R3, 10 kΩ (copies to
  be checked): at 5 V a 5 V source on a floating GPIO, and 0.5 mA
  while shut down. Decided: R3 comes off and the adapter pulls GPIO10
  down (10 to 100 kΩ), so the amplifier is off through reset and
  boot. The amplifier on 3.3 V instead was refused by the user (it
  would share the module's regulator with the Wi-Fi).
- No setting. The pin is driven only when a sound goes to the line,
  and low at boot for `line` or `split`, before GPIO20; never on
  `buzzer`. A module with R3 on and SD unwired plays as before.
- `buzzer::Pins` has the pin and `AMP_ON`: `amp_on`, `amp_off` (only
  if a sound turned it on), `amp_shut` at boot. The buzzer task's
  `amp_for` turns it on before a melody's first sound and waits
  `AMP_WAKE` (20 ms, a guess); off at a `Stop` or a preview's end,
  on through gaps and a ring's repeats. A sound that finds the
  setting changed under it turns it on with no wait.

**The adapters with the filter** (drawn and ordered by the user
2026-11-19): SuperMini v2 and Zero v4 in `adapter/`. R4 2.2 kΩ, C1
4.7 nF, R5 10 kΩ, C2 2 nF, R6 3.3 kΩ to ground; R3 10 kΩ from GPIO10
to ground; J2 in the PAM8302 board's order (A+, A−, SD, 5 V, GND); R7
a 0 Ω link between socket pins 16 and 14 for an HU-058D only. R4 is
at the filter, not the pin, to keep the high-impedance track away
from the antenna. 6.5 mm (SuperMini) and 5.7 mm (Zero) wider than the
old 40.5 mm. With a PAM8302 board and its trimmer: a division of
0.152, −3 dB at 15.7 kHz, −38 dB at 250 kHz.

Owed:

- The setting on a scope or the amplifier: nothing on GPIO20 at
  `buzzer`; after `output=line` with no reset, a note melody, a
  converter clip and a beeper clip on GPIO20 with the buzzer silent;
  switching between previews and inside a note melody, never both at
  once, GPIO20 low at rest; the ramp from the bottom rail, and
  whether it or the first switch to `line` thumps; no thermometer
  hold on `line`; an update from record 12 and a rollback from 13
  keep the settings.
- `split`: notes with GPIO20 quiet, clips with the buzzer silent, the
  alarm with each, no hold during a converter clip.
- The PAM8302 (differential input) heard; the level at its input; the
  hiss with a silent clip and with Wi-Fi busy.
- The shutdown pin on a PAM8302: high from before a sound to after it
  (a melody, a clip, a clip cut short, a ring with its gaps);
  untouched on `buzzer`; the Wi-Fi's clicks gone; the amplifier's own
  click at the edges, which decides whether this is kept; the start
  of a sound not cut; a reset with and without the pull-down; the
  Zero's LED.
- The adapters: each fits its case, the filter's output, the
  amplifier on J2 with its pull-up off, and whether R6 must be 2.7 kΩ
  for a PAM8302 with no trimmer (`bench/pdm-amp.md`).

### A watchdog — built and run on test-clock 2026-11-17; the soak passed 2026-11-19

`esp_hal::init` disables every watchdog, so a hang that was not a
panic left the panel dark or one row lit, and an image that hung was
never rolled back.

- **The RWDT**, enabled right after `esp_hal::init`: `ResetCore` at
  `watchdog::HARD_TIMEOUT`, 60 s, `ResetSystem` a stage later. 60 s
  was the user's choice: a clock in a reset loop can still join the
  network each boot, and the boot needs no feeding. The cost is a row
  latched for up to a minute. Fed only by `watchdog_task` on the
  thread executor, once a second.
- **Check-ins** within `watchdog::DEADLINE`, 10 s, or the feeder
  stops: `display_task` every pass, `clip_task` while a clip plays.
  Not watched: tasks that wait on an event, the scan ring's position
  (decided against), the network (`heal`'s). On a miss the feeder
  writes the culprit into the carry record, then `scan::halt`,
  `clock::carry_time` and a software reset.
- **Across the reset**: carry layout 5, one word appended (the count
  and the last culprit), in the boot log, the heartbeat and `/api`.
  `ota_boots` survives, so a hanging image is rolled back;
  `carry::init` and `ota::init` moved ahead of `pack::init` and the
  settings load for that. The carried time may be a minute old until
  SNTP (accepted by the user). Storing the interrupted program
  counter was decided against by the user.

Run with the `bench-hang` feature (`POST /hang/<kind>`,
`BENCH_HANG=boot`): spins in thread mode, on the interrupt executor
and in a critical section were ended by the timer, back in 70 s with
the record kept (RTC RAM survives `ResetCore`); a parked display or
clip task by the feeder after about 10 s, named, back in 20 s; an
image that spins at boot was rolled back, answering 129 s after the
upload. espflash 4.7.0 flashes a clock whose timer runs. The RTC
watchdog outlives a reset into the ROM's download mode and fires
there. Feed gap after a boot with a join: 1261 ms.

**Clips under suspicion** (a review's finding, fixed 2026-11-17): a
clip that hung the clip task made a reset loop through the alarm's
grace. A reset over the clip task sets `Trips::clips_suspect` (bit 23
of the word), and while set `melody::pick_notes` gives beeps for a
clip. No clip can then play to its end, so it is cleared by a
firmware update or a pack upload (`watchdog::trust_clips`) or a power
cut. Shown as `form::Status::clips_paused`: a line on the clock page,
`"clips_paused"` in `/api` and a Home Assistant `problem` sensor, for
which `DISCOVERY_MAX` went from 4608 to 5120 (the user's choice;
4771 bytes with a full pack; 512 bytes of `.stack`). Both run on
test-clock, the second ring heard as beeps.

Owed: a preview while under suspicion; a pack upload's clearing of
it; `clips_paused` seen in Home Assistant. The soak and the feed gap
after clips, saves and uploads were paid by "The soak again" below.

## An install from nothing on the bench, access point included — run 2026-11-17

The release list's install on test-clock, the flash erased whole, an
image built as the CI builds it, the pages driven in headless
Chromium; the access point reached through a USB dongle
(`wlp0s20f0u1`) on a NetworkManager profile with no default route or
DNS. No code changed.

As designed: the first boot into the access point, the captive DNS
and redirects, `scripts/verify-csrf.sh`, the setup form and its
refusals, the token, an upload and Reboot each with a PIN, a wrong
PIN's 403 and 9 s lockout, the scripts with the token, and the way
back to the access point. A wrong Wi-Fi password retried ten times
14.6 s apart, self-healed after 131 s and never became an access
point; erasing `nvs` over USB brought it back.

Found, fixed in the next section: a request over `REQUEST_BUF` got an
empty reply; `POST /api password=…` on the unprovisioned access point
saved a password with no network; a refused setup form came back
empty; the System page's line lacked a full stop.

Owed: the buttons (the 10 s hold, the IP display), the panel (`AP`,
the PIN's digits, the blank station), a phone's sign-in sheet, the
web installer in a browser.

## The install run's findings fixed — built 2026-11-17, run on test-clock

- `read_request` answers a `431` (a head with no end in
  `REQUEST_BUF`), a `413` (a body over `BODY_MAX`) and a `400` where
  it closed. An answer written with bytes unread is lost to the
  reset, so the 413 discards the body by its length (`discard`) and
  the 431 until the client is quiet 100 ms or eight buffers have come
  (`discard_until_quiet`), inside `HEAD_TIMEOUT`: a body that takes
  over 2 s is still a close.
- `/api` refuses, with a `400`, a password that would leave no
  network (`Credentials::password_alone`).
- A refused setup form comes back filled in: the page's script keeps
  the fields, never the password, in `sessionStorage` and restores
  them when the query has `error=`.
- The System page's line has its full stop.

Unexplained: one `curl` on the access point straight after three
others was refused its connection, once.

Owed: the forms from a phone, whose head is what the 1536 bytes are
tight for.

## A preview never interrupts the alarm — built 2026-11-17, run on test-clock, not yet heard

The user reported `/play` cutting a ringing alarm: the buzzer task
took any latest command, and `/stop` ended a ring. Now the rule is
the buzzer's: `buzzer::preview` and a new `buzzer::stop_preview` do
nothing while `display::ringing()`, and log it; `/play` and the MQTT
`play` call the first, `/stop` the second. All the tasks involved are
on the thread executor and the flag goes up before the ring is
signalled, so no preview slips in. The MQTT `silence` ends a ring as
before and now also stops a preview (the user: Home Assistant had no
way to stop one), with no new command or entity.

Run on test-clock by the log and `/api` (no buzzer on the board):
each was ignored during a ring, and `silence` ended the ring and cut
a preview.

Owed: the ear; the page's own buttons; a press ending the ring.

## The state JSON's worst case — a review's finding, fixed 2026-11-17

The MQTT task's 1024-byte buffer for `form::write_json` was untested
and the worst case is 1467 bytes (about 640 on a clock): over it the
state would silently stop updating. The limit is `form::JSON_MAX`,
1536, with the buffer sized from it;
`the_json_fits_the_state_buffer` builds the worst case, control
characters in the tz, the SSID and the broker user whatever today's
rules admit. `const` asserts hold it under `PAGE_MAX` and the
socket's send buffer. `.stack` went from 41448 to 40936 bytes.

## A 24 h soak of `v0.4-93-g43d96d9` on the bare bench board — started 2026-11-18, stopped at 9 h 13 min

The watchdog's soak, on `another-bench` (10.1.1.65, no panel, speaker
or sensors); host side in `target/soak/` (`driver.sh`, `summary.sh`).
`espflash monitor` hung and left the chip in download mode, so the
log is a plain `cat` of the port.

Load: every 30 s a 498-byte ping and `GET /api`; every 5 min the five
pages; every 10 min a `/play` of the next clip or note melody,
`/stop` 30 s later; hourly a save; every 6 h the alarm rung out, with
previews posted. Pass: one boot; `heals` and `watchdog` 0; no `WARN`,
`ERROR` or panic; heap flat but for one-minute dips; the descriptor
index moving; every `clip:` line clean and at the clip's own length;
the feed gap well under `DEADLINE`; one MQTT session; the page's time
right. Then an OTA and the pack, the feed gap read before each.

Stopped at 9 h 13 min on stray positions in Trantor's clip; the next
section is the rerun. Not covered by either: a panel, a speaker, the
buttons, the access point, the sensors.

## The soak again, 24 h of `v0.4-95-gb7cb573` on the bare bench board — passed 2026-11-19

The same soak on the image with the I2S divider written in the IDF's
sequence, started by its OTA 2026-11-18 12:09 MST.

Passed 2026-11-19 12:09: one boot, heals and watchdog 0, no warning
past the update's own; longest feed gap 1180 ms through 96 clips, the
hourly saves and four rings; every `clip:` line with 0 underruns and
0 stray positions; one MQTT session; no `/api` failure in 2870 ticks.
The heap sat at 54080 bytes, stepped 188 down at 21:34 and was back
after the rejoin: the association's step of the 2026-11-08 soak, at
the same minute of the evening, not a leak. The OTA and the pack
upload afterwards did not trip the watchdog (feed gap 1262 and
1341 ms after); the gap during an upload cannot be read, the clock
resetting before the next heartbeat.

A `pkill` of every `cat` by another session cost a minute of the
serial log: match the capture's own process.

## Twenty channels for the module player, and Crush tried in the default pack — built and measured on test-clock 2026-11-18; Crush dropped the same day

`xm::MAX_CHANNELS` is 20 at the user's request, in `core/src/xm.rs`
and `scripts/xm2clip.py`; the format is still `XMC2`, and 304 bytes
of stack went (39 960 left). No test has more than sixteen channels.

It was for "Crush" by Drozerix (eighteen channels, public domain, The
Mod Archive 179581), made the default pack's fifth clip. On
test-clock it took 57 % of the processor, the dearest clip there is
(Children's sixteen channels were 49 %), with no underruns and the
feed gap at 1424 ms. The user heard it before and after the player's
fixes (next section), disliked it and dropped it from the pack; the
clip and module are in the ignored `xm/`. The twenty channels stay.

## The module player against openmpt123: vibrato's direction, a memory for each portamento — fixed 2026-11-18, heard on test-clock with Crush only

The user found Crush odd. Against openmpt123 (mono, 31 250 Hz;
`CLIP_RAW`), the whole mix and each channel alone:

- Vibrato went up first where FastTracker's goes down first (a lead
  13 to 27 cents off): fixed in `xm::Tune::vibrato`. The instrument's
  own vibrato does go up first.
- Portamento up and down shared one memory; FastTracker keeps one
  each: `Channel::porta_down`, eighty bytes of stack.
- After them: pitch within 6 cents and levels within 0.7 dB, but for
  three sweeps' tails (a slide into the period's limit, which is not
  FastTracker's) and channel 11, up to 1 dB under. Not chased.
- They change 11 of Crush's 48 orders, so they are not all of what
  was heard. Not tried: the stereo sides lost in mono, and a mix 5 dB
  hotter than Halftime's into the same limiter.

Owed: Halftime and Keygen heard since the fixes, which change both.

## A MOD converter, and Elysium the default pack's fifth clip — built 2026-11-18; Elysium withdrawn 2026-11-19, the converter not yet heard

`scripts/mod2xm.py`, at the user's request, makes an XM of a `.mod`
for `xm2clip.py`; the player is untouched. A period is the nearest
note with 428 as C-4, every finetune 20 of 128 lower (a PAL Amiga's
8287 Hz against FT2's 8363). The XM is on the linear table, the only
one `xm.rs` plays, so a slide in periods (1, 2, 3, 5, E1, E2) is
converted at the period the channel has when played, the song
followed once. E0 is left out. Against openmpt123 a channel at a
time, the MOD's and the XM's envelopes correlate at 0.986 to 0.999
with no drift over 222 s; the clock's own mix, 0.980.

"Elysium" by Jester (The Mod Archive 40475, CC BY-NC-SA 4.0) was the
pack's fifth clip for a day; its licence would have made the default
pack non-commercial. The user withdrew it 2026-11-19 to the ignored
`xm/`; the pack is the four again.

Owed: the converter heard on a clock, with any module.

## Two modules of our own: Grieg's Morning and In the Hall of the Mountain King — built 2026-11-19, three in the default pack; not yet heard on a clock

The user asked for XMs made here of two movements of the first Peer
Gynt suite, Op. 46 (Grieg died in 1907; the music is in the public
domain), the instruments public domain or under a permissive licence.

- *In the Hall of the Mountain King* (No. 4): the Mutopia Project's
  piano score, piece 1888, typeset by Coyau from The University
  Society's edition of 1918 and placed in the public domain by the
  typesetter (`copyright = "Public Domain"` in the `.ly`). Its MIDI
  file is `mods/src/mountain_king.mid`, the `.ly` beside it, both as
  Mutopia gives them.
- *Morning* (No. 1): no machine-readable score under a free licence
  was found (Mutopia has Nos. 2 to 4 only; IMSLP has scans; the MIDI
  files about say nothing of their terms). Read by eye off the
  composer's arrangement for piano four hands, Edition Peters 2432
  (plate 7191), the Sibley Music Library's scan
  (`hdl.handle.net/1802/16617`); a script caught one bar read a third
  too low and there may be others. Bars 1 to 31 and 56 to 76, 104.0 s
  (the chromatic middle is left out; the user had it end at the
  silent bar 76).
- Instruments: Versilian Studios' VSCO 2 Community Edition
  (`github.com/sgossner/VSCO-2-CE`, commit `4403009`), CC0. The
  files' names are an octave off as often as not, so the pitch is
  measured. MuseScore General (MIT; its licence asks for its notices
  in a derivative work) was no brighter; `mkxm.py` can read a
  SoundFont (`Sample.soundfont`) but no kit uses it.

**The tool.** `scripts/mkxm.py tune.py out.xm [kit]` (numpy, scipy):
the tune's file (`mods/src/morning.py`, `mods/src/mountain_king.py`)
defines `tune()`; VSCO files are fetched into the ignored `xm/vsco/`.
It writes only what `xm.rs` plays. `Voices` gives a note a channel
that has gone quiet and counts the notes it must still cut.

**Kits (the user's choice, 2026-11-19).** The first kit (VSCO's soft
layers, low sample rates, staccato recordings) the user found dull
and without sustain, the arrangements good; it lacked power over
4 kHz (0.7 % against Halftime's 2.6 %). Kept: `bright` (the default:
loud layers, held recordings, 10 dB more over 2 kHz) and `chip`
(waves made in the script). Modules: `mods/grieg-morning-bright.xm`
("Morning Mood"), `grieg-morning-chip.xm` ("Morning Mood Chip"),
`grieg-mountain-king-bright.xm` ("Mountain King"),
`grieg-mountain-king-chip.xm` ("Mountain King Chip", in no pack). The
first three are the default pack's fifth to seventh clips, after the
four so those keep their places; `clips/README.md` has their sources
and licences.

**A ramp in the player, 2026-11-19** (the user asked). The clicks and
cut releases were the player's: a new note replaced the channel's old
one within a sample, and a level changed once a tick. In `xm.rs` a
channel's gain now moves to the tick's level over a 512th of a second
(`RAMP_PER_SECOND`), and what a cut note was adding goes into
`Tune::cut`, which loses a 64th of itself a mixed sample
(`CUT_DECAY`); a note's own attack is not softened. It rounds off
every module's cuts and volume steps, Halftime's and Keygen's too.
The user confirmed in the page that the clicks were gone.

**Measured.** With the buzzer's defaults the limiter flattens both
tunes' crescendos (output at -5 to -8 dBFS throughout; first kit, a
speaker's defaults not measured). On the bare bench board
(`another-bench`), with the ramp, no underruns: Morning Mood 66 % of
the processor (20 channels, the player's limit), the chip 64 %,
Mountain King 48 %, Halftime 24 %, Keygen 23 %.

Owed: the three heard on a clock, buzzer and speaker; Halftime and
Keygen heard on a clock with the ramp.

## The speaker's pack in a release — built 2026-11-19, the workflow not yet run

v0.5-rc2 had the buzzer's pack only. The release workflow now runs
`scripts/speaker-clips.sh` (which takes the pack's path) and attaches
`onebitdawn-clips-speaker-<tag>.bin`, read back by
`the_pack_file_parses`; the CI makes and checks the same pack on
every push.

Owed: the next tag's release having the file.

## A 24 h soak of the v0.5-rc3 release, installed from nothing through the access point — passed 2026-11-20

The release's own files on the bare bench board (`another-bench`,
`/dev/ttyACM0`): the flash erased, the image at 0 and the pack at
0x2E0000 by espflash, then the setup as an owner makes it (the host's
dongle on the clock's access point, the form in headless Chromium,
then the token with the PIN from the serial log and the broker on
`/ha`). All as planned. The soak of 2026-11-18: same driver, load and
criteria.

**Passed 2026-11-20 16:18** (from 2026-11-19 16:18:07 MST): 24 h on
one boot, heals and watchdog 0, no panic. Heap free 54080, then 53892
from 21:34 (the same 188-byte step at the same minute as the soaks of
2026-11-08 and 2026-11-18), no trend; longest feed gap 1186 ms; 116
clips and four rings with 0 underruns and 0 stray positions. Not
held, both by an outside hand: three MQTT sessions and two
associations.

- The broker was down 8 min 15 s: backoff 5, 10, 20, 40 s then every
  60 s, reconnected by name, heap unchanged.
- The user kicked the clock off the access point on purpose: seven
  connects refused with `AuthenticationFailed`, joined 68 s later,
  three attempts short of the self-heal, no reset; a preview played
  through it with no underrun.
- Gateway probe: 5 lost of 2878, never two in a row, when someone was
  at the desk (2.4 GHz headphones and keyboard by the ceramic
  antenna; partly an inference). The watcher now passes a lone loss.
- After "token set." the System page reads "(none is set)" beside the
  field until reloaded.
- Attach the serial `cat` after `espflash reset`, or the chip is left
  in download mode. Keep a soak's files outside `target/`: a clean
  build removed them mid-run (the logs were saved to `soak-rc3/`;
  those of the soaks of 2026-11-17 and 2026-11-18 are gone).

The OTA and pack uploads the earlier soak ended with were not made.

## The licences of what a release is built from — built 2026-11-20, run on the host; the workflows not yet run

A release hands out binaries (the images, the pack maker's commands,
its WebAssembly) built from other people's crates under MIT,
Apache-2.0 and the like, whose notices are to go with a copy; a
release had none. The web interface needs no copyright line. Release
process only, so within the freeze.

- `cargo about` 0.9.3 (a pinned prebuilt archive in the workflows;
  `cargo install` needs `--features cli`) over the workspace and
  every target a release builds for: one list, a superset for each.
- `about.toml` names the targets and the licences accepted; a
  dependency under any other fails the run.
- `about.hbs`, plain text: a preamble for what is not a crate (the
  project's own licence, the second-stage bootloader espflash puts in
  the merged image, the radio's binary libraries, the clips with a
  pointer to `clips/README.md`), then each licence's text once with
  its crates.
- `scripts/licenses.sh [out]` makes the file; the CI runs it on every
  push; the release attaches `onebitdawn-licenses-<tag>.txt` and puts
  it on the pages as `licenses.txt`, linked from both footers.

The run: 171 crates, 162 taken under MIT (of an "MIT OR Apache-2.0"
the first accepted is chosen), ryu under Apache-2.0, the four riscv
crates ISC, smoltcp, managed and xarxa-driver 0BSD, unicode-ident
with Unicode-3.0 for its tables. Some fifty crates (esp-hal's and
embassy's among them) are published with no licence file and get the
plain text with `<year> <copyright holders>`; the preamble says so.
The radio's binaries are in `esp-wifi-sys-esp32c3`; the `LICENSE` of
`esp-idf`, `esp32-wifi-lib`, `esp-phy-lib` and `esp-coex-lib` is
Apache-2.0 in each. One of them, `libwpa_supplicant.a`, is ESP-IDF's
build of Jouni Malinen's wpa_supplicant and hostapd, BSD with three
clauses, and the firmware links it; a binary is to reproduce that
notice, so the preamble has it in full, from the component's
`COPYING` and `README`. The workspace's own crates are left out
(`private`, `publish = false`).

Owed: a push's CI run with the new step; the next tag's release
having the file and `licenses.txt` on the pages (until then the
footers' link has nothing behind it). Not looked for: other
third-party code inside Espressif's binaries or the bootloader beyond
the supplicant.

## `UTC+3` in the timezone field is east — built 2026-11-20, run on white-clock

The user typed `GMT+3` and got UTC-3: right by POSIX, where the
offset after a name counts west, but nobody means that. A fix, on
`master`.

A value that is all of `UTC` or `GMT` (either case), a sign and
`h[h][:mm]` is an offset east positive (`tz::shorthand`);
`form::apply_one` stores `tz::fixed` of it (`<+03>-3`). Past
`TZ_LIMIT_MINUTES` or with odd seconds it is a bad value. Normalised
at input, not in the parser, so every image reads the record alike
and a rollback keeps the zone. The sign is required: `GMT0`,
`GMT0BST,M3.5.0/1,M10.5.0` and other names (`MSK-3`) are rules as
before; the build's `TZ_RULE` stays POSIX. Refusing the form as
ambiguous was the stricter choice; the user took this one. The label
is "Timezone, POSIX rule or UTC±N". A clock with `GMT+3` stored in
POSIX's sense keeps it until its settings form is next saved;
`<-03>3` says that now.

On white-clock over `/api`, 2026-11-20: the forms stored as designed,
`UTC+15` a 400. The first image let `GMT+3:5` through to the parser
(185 minutes west); `shorthand` is now the parser's own reading
turned over, and the second image stored `<+0305>-3:05`.

## A pack's clip this firmware does not play is skipped — built 2026-11-22, on the host only

The user asked for it as a forward-compatibility change worth having
in the first public release; written to apply to `master` as to
`next`. Before, one clip of an unknown version refused the whole
pack, at upload and at boot.

- `Pack::parse` skips a clip `clip::is_unsupported` names (a known
  kind's other version, or a header cut short). `count` and `clip`
  number only the clips that play, so the melody table, pages, `/api`
  and discovery do not change; `skipped` and `skipped_clip` feed the
  boot's log line, the upload's answer and the Clips page.
- Still refused: a pack with no playable clip
  (`PackError::Unplayable`), a playable clip under a second
  (`PackError::Clip`), a bad name on any entry.
- An alarm set to a skipped clip is the beeps until a firmware plays
  it.
- Unchanged: a firmware up to v0.5 refuses such a pack whole; a clip
  of a new *kind* must still set a reserved bit or the version.

Owed: on a clock, a pack with a clip of an unknown version: the
upload's answer, the boot's log line, the Clips page, the list of
sounds. The user's word on which branch takes it.

## "Starlight & Aurora" clicks on a speaker: the limiter off, its knee from 12 to 64; its hiss is the transistor's stage — built 2026-11-20, heard on test-clock and white-clock

**The clicks.** Heard by the user with the speaker's pack on
test-clock. The mix before the shaper is clean (`CLIP_PCM_DUMP`,
`CLIP_WIDE`, `CLIP_RAW`). With `knee=12,release=13` the limiter is
never off (mix peaks near 62 000, knee 12 288), its attack is
instant, and each new peak gets a flat top: 1254 samples at full
scale at 181 places. At knee 48 it is 63; at 64, the limiter off,
none, at -15.7 dBFS, as loud as "Suspended Seconds". The user took 48
by ear, then asked for the limiter off: `scripts/speaker-clips.sh`
uses `knee=64`.

**The hiss.** Heard under the quiet piano. First put down to the
8-bit samples (a piano note is 43 to 45 dB over its rounding) and
linear interpolation's images; louder samples were tried, the user
heard no difference, nothing was kept. Then 16-bit samples (on
`next`) left it unchanged on test-clock, whose speaker is on the
buzzer's transistor (the S8550, the speaker mod), while white-clock's
amplifier was clean: it is the transistor's stage under any signal (a
pure sine hisses there, a clip of zeros much less). On the amplifier
the user found 16 bits better and 8 acceptable, so v0.5 ships 8 bits.
A clip of true zeros is made by hand (ADPCM codes of 0): the WAV
converter refuses silence and brings sox's dither up to full scale.

**128 bits a sample on the buzzer's pin, on `master` since
2026-11-21** (a fix, by the user's ruling). At 128 bits a sample
(4 Mbit/s) instead of 256 the transistor hisses less: half the edges
through a stage that turns off more slowly than on (not measured on a
scope). `clip::converter_bits` gives the buzzer's pin
`BUZZER_CONVERTER_BITS`, 128, and the line output the build's
`CLIP_PDM_BITS`. 64 bits the driver refuses (`Pdm(InvalidClock)`).
Heard on test-clock's speaker and white-clock's amplifier and
modified buzzer.

Owed: a stock buzzer at 128 bits and 31 250 Hz, which nobody has
heard.

## A 24 h soak of master, `v0.5-rc4-3-g9039ef5`, on the bare bench board — started 2026-11-21, passed 2026-11-22

A local build on `another-bench`, the first long run of the buzzer's
pin at 128 bits a sample and of the timezone commits; the soak of
2026-11-18 and 2026-11-20, files in `soak-master/` (untracked). The
start is the reboot into `ota_1` after `scripts/ota.sh`.

**Passed 2026-11-22 00:35** (from 2026-11-21 00:35:26 MST): 24 h on
one boot, heals and watchdog 0, no panic, no warning but the update's
own and three lone lost gateway probes of 2879. Heap free 54080, then
53892 from 21:34 (the same step again), no trend; longest feed gap
1178 ms; 116 clips and four rings with 0 underruns and 0 stray
positions; one MQTT session, one association; no `/api` failure.
Read again 2026-11-23 at 55 h 23 min: still one boot, no second heap
step, the daily alarm rang by itself cleanly.

- The descriptor index read 2 once in 1440 heartbeats where the
  others read 118 to 146 times (twice in the rc3 run): not looked
  into.
- `stty -F /dev/ttyACM0 raw -echo -hupcl` hung for two minutes on a
  port with echo on; setting it from Python with `TCSANOW` after a
  `tcflush` works at once, with no reset.

## The installer asks: buzzer or speaker — built 2026-11-21, deployed by `v0.5-rc5`, no install from it yet

The user asked that the web installer offer which clips to install,
as a last touch before v0.5 on the frozen `master`.

ESP Web Tools takes one manifest, so a release has two:
`manifest.json` with the buzzer's pack and `manifest-speaker.json`
with `onebitdawn-clips-speaker-<tag>.bin` at 0x2E0000. The page's
step "Choose the sound" (buzzer the default) and the release together
pick the manifest. The choice is the pack only: the `output` setting
is the clock's, in `nvs`, which the installer does not write; the
page says so. Earlier tags have `manifest.json` only, and the list
offers none of them. No firmware change.

Checked in headless Chromium with the releases stubbed; `v0.5-rc5`
(2026-11-21) deployed both manifests and the speaker's pack (949750
bytes). After the tag the line about `output` became a notice of its
own, shown while the speaker is chosen.

Owed: an install of each choice from the page onto a clock; the
notice is not deployed until the next tag.

## Samples of 16 bits in a module, for the piano of "Starlight & Aurora" — on `next`, a feature for after v0.5; built 2026-11-20

Begun because the hiss under the tune's piano was thought to be its
8-bit samples; it is the transistor's stage (the section above).

- A module with a 16-bit sample is `XMC3`; one with none stays
  `XMC2`, byte for byte. `XMC3` uses the first of an instrument's
  three spare bytes as the sample's flags: bit 0 for 16 bits,
  little-endian, signed. Length and loop stay in samples. An older
  firmware refuses the pack for it (`clip::is_unsupported`).
- The player (`core/src/xm.rs`) mixes at 16 bits, an 8-bit sample
  being its byte times 256: the same numbers as before.
- The converter (`tools/pack/src/xmc.rs`): `samples=16` keeps 16-bit
  samples; cutting to 8 now rounds instead of dropping the low byte
  (the next section changed the default). `mkxm.py`:
  `Sample(bits=16)`.
- With the piano's ten samples at 16 bits the clip was 363 309 bytes
  (221 795 at 8). On test-clock: no underruns, 80 of 111 s filling.

True of the tune, but not the hiss: interpolation images (the piano's
samples are at 11 025 Hz), and a loop keeping the treble the note had
where the loop starts (a low-pass of the loop changed nothing
measurable; not used).

Owed, found and not fixed, in the shaper's high-pass
(`core/src/shape.rs`): every module's clip for a speaker carries a
constant offset of about -580 of 32512 (1.8 % of full scale), since
the section's `>> 14` rounds down and the feedback's gain at 0 Hz is
about 2500 for 100 Hz at 31 250 (a ninth of that at the buzzer's
300 Hz; not heard); and `highpass=20` gives a clip stuck at full
scale (Q14 coefficients with the corner that low).

## 16 bits in 8 by prediction, for a module's samples and for sound files — on `next`, a feature for after v0.5; built 2026-11-22, measured on test-clock and the bench board

A 16-bit sample at 8 bits a sample and a sixteenth more, as the
difference from what the samples before predict: the SNES's BRR with
8-bit codes. Signal over rounding noise on the host: "Suspended
Seconds" 39.4 dB at 8 bits, 46.1 with only a shift a block, 67.4
predicted; "Starlight & Aurora" 43.2, 45.2, 55.9 (its bells predict
badly). By the user's ear the shift was no better than 8 bits and the
predicted close to 16.

**The format** (the user asked for it as the default for 16-bit
input, plain 16-bit PCM an option):

- Blocks of 17 bytes for 16 samples: a byte with the predictor (high
  nibble) and a shift (low), then sixteen signed residuals; a sample
  is `predict(the three before) + (residual << shift)`, held to 16
  bits.
- A table of 15 predictors written once (three `i16` coefficients in
  64ths each, 90 bytes, held to ±1023 as read), fitted to the sound.
  Predictor 0, "no prediction", is in the decoder
  (`adpcm::coefficients`) and never in the table, with a test: fitted
  predictors alone were 9 dB worse on loud material, where at full
  scale a prediction can be further off than a residual reaches. The
  user insisted it not be forgotten.
- The fitting (`tools/pack/src/adpcm.rs`, `train`): 25 rounds of
  assigning blocks and refitting, starting from seven fixed
  predictors; nothing random, `f64`, so the host and the page agree.
  Then all 144 predictor and shift pairs are tried a block on the
  samples as decoded, so no error adds up.
- A module, `XMC4`: bit 1 of the sample's flags. A predicted sample
  is an 8-byte head for the loop (the three samples before the loop's
  block, newest first, and the one at the loop's start, each `i16`),
  the table, the blocks. The default for a 16-bit sample;
  `samples=16` keeps it, `samples=8` cuts it.
- A sound file: `ADP2` (magic, rate, sample count, table, blocks),
  the default (`format=packed`); `PCM2` is 16-bit PCM
  (`format=pcm16`); `format=adpcm` and `format=pcm` as before. All
  through `adpcm::Decoder`. 17 KB of pack a second where it was 8.
- A firmware up to v0.5 refuses a pack with any of these, so a pack
  made with the defaults is not for such a clock: the user's choice;
  the manual and the page say what to give instead.
- The converter (`xmc.rs`) writes a 16-bit sample whose low bytes are
  all one value as 8 bits whatever `samples=` says (Keygen's lead, so
  its clip is unchanged), and keeps a sample at 16 bits where
  predicting would not be smaller (the chip Morning's 64-sample
  waves; that clip is `XMC3`).

**The player** (`core/src/xm.rs`). A channel keeps two decoded
blocks, its position's and the next, decoded on entering a new block;
the mixer reads `i16`s from them (decoding sample by sample cost 26
and 50 % of the player's time on the host). The loop's wrap decodes
from the head.

**Seek points for 9xx** (a review's finding; the user chose the
variant that costs nothing where no 9xx is). A 9FF would decode 4080
blocks before the first sample. A predicted sample of an instrument
played with 9xx has, after the table (bit 2 of the flags), for every
256 samples but the first the three decoded samples before that
block: 6 bytes for 272. `refill` starts from the nearest known state,
so a 9xx decodes two blocks. A clip from elsewhere without seek
points decodes from the start: slow and right.

**Rejected, with the deciding figure.** CD-XA Level A's parameters:
3.6 dB worse on pianos (63.8 against 67.4), no loop to the sample.
More predictors or a block's own: no gain over 16 fitted, which gave
3 to 4 dB on a piano and 8 on the bells over the seven fixed. A table
a segment: within 0.4 dB of one table; the user settled on one a
clip. Not built: 4-bit residuals (7 to 8 dB over sox's IMA ADPCM on
Futurama at an eighth more bytes), to weigh against the pack maker's
own ADPCM encoder first.

**The tunes.** "Suspended Seconds"' piano, all of "Starlight &
Aurora"'s instruments and every Grieg sample are 16 bits in their
modules; the clips were made again. Against the 16-bit mix:
"Suspended Seconds" 67.2 dB (43.7 at 8 bits), "Starlight & Aurora"
69.4 (47.4), Morning 65.4, Mountain King 64.5. Clips:
`suspended_seconds.xmc` 272 902 bytes, "Starlight & Aurora" 236 371
(428 359 at 16 bits). The buzzer's pack is 765 228 bytes, the
speaker's 1 001 663 of 1 179 648.

**On a clock, 2026-11-22 and 23.** On test-clock
(`v0.5-rc5-9-gd982662`) the user heard both packs and found them
good; no underruns. Predicted clips cost no more than 8-bit ones, but
the chip Morning at plain 16 bits took 82 % of the processor against
70 % at 8: two byte loads and two checks a sample, 62 500 times a
second over twenty channels. What was found and kept:

- Cache misses are not it (`EXTMEM`'s counters: a 32-byte line is
  filled on a miss, one read in 180 for Morning, under 2 µs each).
  The flash's cache is 16 KB, shared by code and data.
- The chip loads unaligned correctly, but telling the compiler
  (`+unaligned-scalar-mem`) is an unstable feature set to become a
  hard error (rust-lang/rust#162235): not kept.
- Kept (the user's decision: align, no byte-wise fallback): a channel
  holds its sample as a slice, `held` for 8 bits, `words` (`&[i16]`
  by `align_to`, the one `unsafe` of `core/`) for 16. The converter
  puts a 16-bit sample at an even offset in the clip, the pack maker
  such a clip at an even offset in the pack (a byte of 0 before it,
  layout unchanged), `core/build.rs` aligns built-in clips,
  `Pack::parse` skips what `xm::is_misplaced` names. Output byte for
  byte as before. The chip Morning at 16 bits: 82 % to 64 % on
  `another-bench`.
- Kept (the user chose the 3.5 KB variant): `core`'s `ram` feature
  puts the per-sample code in `.rwtext` (eight functions: the
  module's `next_sample` with the mixer, the fills, `refill`,
  `decode_block`, `Shaper::run`); `step_tick` and the SID and AY
  players stay in the flash. `.rwtext` 8032 bytes from 4560, the main
  stack 33 648 from 37 120. Morning Mood 72.5 to 70.8 %; 6.4 and
  9.4 KB in RAM bought 0.5 and 0.3 s of 104 more.
- The stack (`src/stack.rs`, the `stack-watch` feature): of 33 648
  bytes, 20 532 used in the boot, 17 432 at the deepest in a
  four-hour run of alarms, previews, saves and `/api` (no underrun,
  no failed request, heap 54 080); 13.1 KB never touched.

Shares on `another-bench` after the alignment, before the code in
RAM: Halftime 24 %, Keygen 23 %, Morning Mood 72 % (the dearest),
the chip 64 %, Mountain King 54 %, "Suspended Seconds" 46 %,
"Starlight & Aurora" 66 %, the same on either output. A clip moves
two or three points between builds for no reason of its own;
test-clock is dearer than the bare board for the same tune (Morning
73 % against 66 %), why is not known (not the panel).

Owed: a clip heard from the build with the aligned read and the code
in RAM, and its run on a clock with a panel; the Grieg tunes at 16
bits heard; a module with 9xx on a 16-bit sample played on a clock
(there is none here); the 4-bit residuals weighed.

## The module player's cost: silent channels, the shaper's division, mixing by blocks — built and run on the bare bench board 2026-11-23, on `next`; heard on test-clock that evening

The user asked what would make the module player noticeably cheaper,
then for the four answers below, one at a time in this order. A host
count had shown 52 to 62 % of the channels mixed in the Grieg clips
and Suspended Seconds silent (gain 0, no ramp; 1 to 4 % in the
four-channel tunes): a looped sample whose envelope or fade has
reached 0 is `playing` for ever.

Run on `another-bench` (the bare board, nothing heard), each clip
through by `/play`; no underruns or stray positions in any. Filling
as a share of the processor, the speaker's pack on the line output
(the buzzer's pack gave the same):

| clip | before | 1 | 2 | 3, by runs | 3, into the chunk |
| --- | --- | --- | --- | --- | --- |
| Halftime | 24.3 % | 24.3 | 21.5 | 17.7 | 15.6 |
| Keygen | 23.5 | 23.4 | 20.7 | 17.1 | 14.9 |
| Morning Mood | 70.8 | 39.0 | 35.7 | 29.3 | 27.1 |
| Morning Mood Chip | 62.7 | 35.5 | 32.2 | 23.5 | 21.3 |
| Mountain King | 52.3 | 34.5 | 31.3 | 25.6 | 23.4 |
| Suspended Seconds | 45.4 | 29.0 | 25.4 | 21.6 | 19.3 |
| Starlight & Aurora | 65.1 | 40.3 | 36.5 | 29.9 | 27.7 |

A chunk is 16.4 ms of sound and the ring sixteen of them, so an
underrun wants the filling behind for a quarter of a second;
Morning Mood's worst chunk went from 14 285 us to 10 699. `.rwtext`
went from 8032 to 9092 bytes, the main stack from 33 648 to 32 472.

1. **A silent channel is not played.** `settle`, which ends every
   tick, marks a channel quiet; the mixer passes it by (one flag,
   `mixing`, the single test the loop had: two tests and a counter
   cost the small tunes two points), and the next tick moves it on
   by the mixed samples since (`Tune::wound`) in one 64-bit step, so
   the mix is the same sample for sample. A predicted sample moved
   on is within `REACH` (64 blocks) of a place decoding can start
   from, or is decoded up to where it is in the tick. The clip
   task's log line now ends "a chunk in N us at most".
   - Without seek points the decoder still did a tick's blocks of
     every quiet channel at once (576 for one output sample at worst
     in Morning Mood). The user decided: `xmc::predicted` writes
     seek points for every predicted sample, not only one a 9xx
     plays from an offset; no format change, older clips play as
     before. Morning Mood's blocks decoded fell from 2 477 302 to
     867 144. The three predicted clips in `clips/` were made again,
     about 2 % larger; the default pack is 777 948 bytes, the
     speaker's 1 019 189, 86 % of the partition.
   - Host: a channel silenced and brought back with no new note, at
     each of the three kinds, against the mix with nothing skipped
     (`EAGER`); the raw mix of nine clips byte for byte as before.
2. **The shaper without a division of 64 bits.** Every clip is
   `wide`, so `Shaper::run` called the ROM's `__divdi3` once an
   output sample. With `gain` under 2^23 (`SMALL`) the sample is
   `q * STEP + r * STEP / gain`, each division a multiply by
   `m = 2^32 / gain` with an exact one-step correction (proof beside
   the code); `m` is prepared by one `divu` only when `gain`
   changes, and a larger gain takes the old division (`slow`). Same
   bits out, tested against `slow` over two million levels. Then the
   high-pass: `Biquad::run` with `i32` coefficients and products,
   `wild` (the old arithmetic) for an output no `i32` holds. The ROM
   division was some 110 cycles an output sample, two to three
   points of any clip; the high-pass half a point. A limiter at rest
   is no cheaper than one preparing its divisor every sample.
3. **The mixer by blocks.** First put off (not needed against
   underruns; it is for what else the clock does while it rings),
   then built the same day. `Tune::render` fills `RUN` (16) mixed
   samples into a local `i32` buffer, in runs cut short where a tick
   falls; sixteen is a whole number of oversampling groups. `mix`
   adds each channel over the run in one generic loop built three
   times, a kind each, its state in locals; the shaped samples go to
   `Tune::ready`. `mix_one`, the old mixer, is kept for the tests,
   which hold the two to the same bits. The chip Morning (samples
   read from the flash) gained 27 %, the predicted one 18 %.
   Deepest stack use (`stack-watch`): 17 500 of 32 728 bytes.
   - **A render straight into the chunk.** `Tune::block` gives what
     is left of `ready` as a slice and `taken` says how many were
     used; `fill_samples` writes them into the chunk while the
     stream is in a tune's body, the ramps and the end going the old
     way through `Player::level`. The chunk is bytes, so rendering
     into it directly would need an `unsafe`. It saved 2.1 to 2.3
     points of every clip.
   - Heard 2026-11-23 on test-clock (the speaker's pack written to
     `clips`): the user played the default pack and his local
     speaker's pack and found all well, the first three items.
4. **`opt-level` 2 or 3 for `core` alone, last of all; tried
   2026-11-23, the image left at `s`.** Tried with
   `--config profile.release.package.onebitdawn-core.opt-level=N`.
   At 2 every clip is 2 to 3 % cheaper for 1.2 KB of the main stack;
   at 3 the clips with predicted samples are 7 to 8 % cheaper
   (Morning Mood 25.0 % against 27.1) for 1.9 KB of the stack and
   13 KB of the image. Nothing broke; not heard, the output not
   compared on the host. `#[optimize(speed)]` (unstable; it lifts
   the size attribute, it is not level 3) on `adpcm::decode_block`,
   `xm::fill` and `xm::refill` gave a fifth of what 3 gives. The
   user decided against a crate of its own for the decoder; the
   attribute is in "Planned, not built" until it is stable.

Owed: a SID tune, which ends in the same shaper, run on a board; the
deepest stack use measured again after the render into the chunk; the
figures on a clock with a panel; the pack maker's page tried in a
browser with the new converter rule, unless the page's player heard
on 2026-11-24 (next section) counts.

## `knee=auto`: the limiter's knee fitted to a tune, a speaker's choice of four, and the installer's third sound — built 2026-11-24, on `next`; heard in the page's player, not on a clock; the release workflow not run

The user asked whether the pack maker could choose a speaker's
limiter options itself, as loud as possible with no clicks, the
buzzer's left alone (its overdrive sounds better).

A click is not clipping: the limiter's attack is immediate
(`core/src/shape.rs`), so a sample over the decayed envelope comes
out at exactly full scale, and a run of them is a flat top. The knee
only lets by the samples under it, so the count at any knee comes
from one pass over the sorted levels of the samples flattened.
Measured on the host: Starlight & Aurora (release 13) has 687 flat
samples a minute at a knee of 12, 34 at 48, 0 at 64 (the user's
choice); the speaker's default knee of 3 flattens the other modules
5600 to 10 200 times a minute. On headphones the user heard those
clicks too, masked by loudness, so one budget fits every tune and its
cost is loudness.

- `knee=auto` for a module (`xmc::fit_knee`): the lowest knee with no
  more than `tops` flat samples a minute (60 unless `tops=N`); the
  note gives the knee, the peak and the count. Starlight & Aurora
  gets 46 and Suspended Seconds 14 (held by a test); the other
  modules 72 to 97, 5 to 7 dB quieter than in the speaker's pack.
- `Limiter`, the third part of a `Target`: `loudest` (the knee as it
  is), `loud` (1650 a minute, which the user found acceptable in
  headphones), `cleaner` (600), `cleanest` (60), for a speaker's
  module that gives no knee of its own; `--limiter` on the command,
  a box on the page. The default is `cleanest` (the user's
  decision); the tests hold the repository's clips to `loudest` by
  name and both packs are unchanged. The page starts at Speaker (the
  user's decision); the command's default is still the buzzer.
- `pt3c::fit_knee`: the flags' fifteen knee steps tried from the
  highest down with the clip's own limiter, with the player's own
  drive (the speaker's default `knee=37,drive=150` flattens at any
  knee). Synchronization at `cleanest`: 93.
- The installer's third sound (the user's reasoning: an amplifier
  has a volume, so it takes the clean pack; a speaker on the
  transistor has none). `scripts/speaker-clips.sh <pack> amplifier`
  makes the pack with `--limiter cleanest`; a release has it as
  `onebitdawn-clips-amplifier-<tag>.bin` with
  `manifest-amplifier.json`, and the CI makes and checks it. The
  sound is a drop-down box; for a release without that pack (v0.5)
  the option is disabled, says "(not in this release)", and such a
  choice becomes the speaker's. Checked in headless Chromium with
  GitHub's answer stubbed.

Not built: the same for `.sid` (no knee). The user saw the pages and
heard the page's player on 2026-11-24 and approved.

Owed: Synchronization as the amplifier's pack has it heard on a
clock; the amplifier's pack heard on white-clock; the release
workflow run, which only a tag does. Open: louder with no flat tops
at all needs a limiter that sees the peak coming, which is the
firmware's and a new clip version.

## A review's two findings in the pack maker: the fitted knee at a drive over 127, and a module no offset mends — fixed 2026-11-24, on `next`, on the host

Two findings of a review of the commits of 2026-11-23 and 24 (the
user's `bug-report.md`, not in the repository), both in `tools/pack`
and reproduced on the host first; firmware, formats and the
repository's clips untouched.

- **The fitted knee did not count the drive.** `xmc::fit_knee` was
  right only at a drive of 127: a higher drive flattens samples
  under the knee too (Keygen, `knee=auto,tops=0`, `drive=255`: knee
  98 with 8598 samples at full scale, the note claiming none; now
  197 and none). A sample now counts as flat at a knee if its level
  times the drive is 127 knees or more; at 127 and under nothing
  changes. A knee that would pass 255 stays there and the note says
  the budget is not kept. `pt3c::fit_knee` had no such fault. Test:
  `the_knee_fitted_counts_the_drive`.
- **A module with 16-bit samples at both even and odd offsets.**
  `pack::placed` puts a byte before a clip `xm::is_misplaced` names,
  which cannot mend a clip with both, and the pack came out with a
  clip the clock would skip. Only a `.xmc` from elsewhere or a
  damaged one is such. `convert_for` and `build` refuse it by name,
  and `build` refuses any pack with a clip skipped. Test:
  `pack::tests::a_module_no_offset_mends_is_refused`.

## A 24 h soak of the v0.5-rc6 release, installed from nothing through the access point — passed 2026-11-25, with a Wi-Fi outage and a self-heal on the way

The release's own image and buzzer's pack on the bare bench board
(`another-bench`), the flash erased whole, set up as a new owner
does: the setup form on the clock's access point in headless
Chromium, then on the LAN the token on `/system` with a PIN and the
broker on `/ha`. The driver, the watcher, the load and the criteria
are those of the soaks of 2026-11-18, 2026-11-20 and 2026-11-21; the
files are in `soak-rc6/` (untracked). The install went as planned:
the station 2 s after the save, the address at 5.6 s, the pack of 8
clips checked in 501 ms, MQTT 3 s after the broker's save.

**The first stretch**, from 2026-11-24 11:39:23 MST: 5 h 4 min clean
(27 clips and one ring with no underruns, heap free 54080, the feed
gap 1185 ms). At 16:43 the home access point went away for a couple
of minutes, the user's own outage. The clock lost one probe,
disconnected on `BeaconTimeout`, found only the house's far access
point at −82, and had the same address 10.1 s after the disconnect
and MQTT 4.3 s later; no reset, no heal, no watchdog count.

**The marginal link, 19 minutes.** The clock stayed on the far access
point when the near one came back, and the user chose to let it run
there. The signal fell to −86…−90: 28 probes lost of 644 in growing
runs, MQTT lost twice more, the host reaching the clock at 9 ticks of
30. A preview still played with 0 underruns; the heap and the feed
gap did not move. At 17:02:33 the self-heal fired as designed (ten
probes lost in a row, 300 s); the boot after it carried the time and
`heals: 1`, took the near access point at −40 and had MQTT at 6.8 s.

**Passed 2026-11-25 17:02**, the 24 h counted again from the
self-heal's boot: one boot, heals still 1, watchdog 0, no panic, no
warning but two lone lost gateway probes of 2880.

- Heap free 54080, then 53892 from 16:50 of the second day (the
  188-byte step of the earlier soaks, here 23 h 47 min into the boot
  and not at 21:34); nine one-minute dips, lowest 50672. No trend.
- Longest feed gap 1180 ms, through 115 clips, 28 saves and four
  rings; every clip with 0 underruns and 0 stray positions, each ring
  five minutes.
- One MQTT session, one association, RSSI −36 to −40.
- The host: 2870 ticks, no `/api` failure, the slowest 387 ms; three
  lone lost pings; every page 200.
- The descriptor index read 2 once in 1440 heartbeats, as in the
  master soak.

With the first stretch that is 29 h 23 min of rc6 on this board, the
only reset in it the self-heal the outage led to.

Learned: a station that rejoined a much weaker access point never
looks for the better one again (its section under "Planned, not
built"), and a link that answers one probe in four keeps the
self-heal from firing for as long as it does. Setting the port raw
from Python reset the board this time, so the very first boot's log
was not captured.

Owed: the uploads the soak of 2026-11-18 ended with were not made.

## A keygen-style tune of our own for the alarm, "Serial Dawn" — built 2026-11-25, in the default packs, on white-clock in the speaker's pack

The user asked for a module in the key generators' manner, of
samples made in the script and none recorded, for the speaker and the
buzzer alike. It went into the default packs in Trantor's room, on
master: a change of the melodies with no code changed is not a
feature (the user).

- `mods/src/serial_dawn.py` for `scripts/mkxm.py`,
  `mods/serial-dawn.xm`: A minor, 150 beats a minute, 28 bars, 44.8 s
  the first time and 38.4 s a round. A four-bar introduction of
  climbing sixteenths, A (Am, Dm, E, Am twice), B (F, G, Em, Am, F,
  G, E, E), A again; the last bar is the pickup into A and the
  module's restart position is order 1.
- Pulses of three widths (lead, stabs and bass, echo), a swept sine
  for the kick, noise for the snare and the hat; the chords are
  effect 0 arpeggios. Eight instruments in 6888 bytes of samples.
- `mkxm.py` has `Module.loop_from(row)`, the restart position, which
  it wrote as 0; the converter and the player had it already.
- A narrow pulse has a mean, and a note of it starting or stopping is
  a step: the pulses are put about their mean in the tune's script
  (`mkxm.py`'s `pulse` is as it was, for the Grieg kits).
- `clips/serial_dawn.xmc` is the buzzer's eighth clip, after
  Suspended Seconds; the speaker's pack makes its own from the module
  before Starlight and Aurora, which is the ninth again. It is the
  project's, under its licence.

The notes were written without being heard, so the tune, its levels
and the limiter's settings are the user's ear's to judge. The
speaker's pack was written to white-clock and the user had it
committed after; what was heard is not recorded.

## Trantor out of the default pack — built 2026-11-25, on the host

The right to distribute Trantor's title music is not to be had, so it
left the default pack and the repository's tree for the local packs.
A fix of what is distributed, so on master.

- `clips/trantor.bpr` is `tap/trantor-packed.bpr` (ignored), and
  `packs/local.sh` takes it from there under its old name and label,
  first in the three local packs as it was.
- The melody is kept by name: a clock with Trantor chosen rings its
  default after the new pack. The beeper player and
  `scripts/zxbeeper.py` stay; no clip in the repository is a beeper
  clip now, so the CI's run with the clips built in no longer covers
  `the_beeper_clips_built_in_add_up` (the synthetic beeper tests
  remain).
- The file is still in the repository's history and in the packs of
  the releases up to v0.5-rc6; nothing was done about either.

## Halftime out of the default pack; Keygen is CC BY 3.0 — built 2026-11-26, on the host

Dualtrax published Keygen on SoundCloud as CC BY (the
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

## n1k-o's "lerka" the default pack's third clip, "Lera"; the pack maker writes R as E9 — built 2026-11-26, heard on white-clock in the speaker's pack

The user has Oleg Nikitin's permission for `nq-lera.xm` ("nq/skrju -
lerka", 2012, for TSRh). A clip of the default pack, so on master,
with the converter's change the clip needs to be played whole.

- `mods/nq-lera.xm` as found; `clips/lera.xmc` at the pack maker's
  defaults, 27 412 bytes, 18 channels, 115 s. Name `lera`, label
  "Lera", third, after Synchronization.
- The module has R02 26 times, a roll on one instrument, which the
  player has not. `xmc.rs` writes an R that changes no volume as E9
  (x of 8; x of 0 on a channel with no R that has a volume; y not 0),
  over the packed cells in place, so the roll plays on every firmware
  that plays a module. No other clip in `clips/` changed by a byte.
- The default pack is eight clips, 697 KB, and the speaker's nine,
  919 KB as master makes it.
- Heard and passed on white-clock in the speaker's pack, over a build
  of `next`. Owed: the buzzer's pack on a clock, and the `clip:` line
  for underruns at 18 channels.

## Checks still owed

A line is struck or deleted when its check is made or no longer
wanted, with the result put in the section.

- 16 bits predicted, on a clock (2026-11-22, on `next`; both default
  packs timed and heard on test-clock that day): a sound file made
  with the defaults.
- The module player's samples read from the channel's own slices,
  16 bits as `&[i16]` (2026-11-23; built and timed on the bare bench
  board): a clip heard, and the figures on a clock with a panel.
- The pack maker (2026-11-19): the release workflow's three builds of
  the command (Linux, Windows, macOS) and the deploy of `docs/pack/`
  with its wasm, which run only with a tag; the command in a user's
  hands.

Sections in `HISTORY.md`, by title, with what each still owes (read
again against the sections on 2026-11-24, when both files were
condensed; the lines settled there are gone):

- **Hold the self-heal while the alarm rings, and ring for five
  minutes** — built, not verified on the panel.
- **Codex review of 2026-11-07: eleven findings fixed** — released as
  v0.3 and soaked; the fixes' own checks owed.
- **Web flasher and releases** — an install from the browser.
- **Timezone rules: POSIX TZ strings and daylight saving** — the
  panel, a real transition, the page on a phone.
- **ZX Spectrum tunes: an AY-3-8910 and a PT3 player on the chip** —
  the register comparison with a reference player.
- **The station picks its access point: the strongest of each scan**
  — built 2026-11-13, not on a clock yet.
- **The page split in four** — white-clock, a phone and the access
  point.
- **The thermometer held while Q1 is hot** — a short preview and the
  serial log's lines.
- **PCM clips at half the flash: IMA ADPCM, `ADP1`** — the buzzer
  through the clock's own decoder.
- **A clip of a version no player reads is refused** — no upload of
  such a pack tried on a clock.
- **A watchdog** — on a clock with a panel (soaked 24 h on the bare
  bench board 2026-11-19, longest feed gap 1180 ms).
- **Sixteen channels for the module player** — the 31 250 clip as a
  ringing alarm for five minutes; the version line of a changed
  tree's build lacks `-dirty`.

## Review backlog

The open findings of the review passes. Line numbers are those of the
commit each pass read; every item was re-read against the code on
2026-11-17 and holds in the function it names unless a "Checked" note
says otherwise. Each pass's account and its closed items are under
the same headings in `HISTORY.md`. Strike an item when it is done or
decided against, with the date.

### Review backlog (2026-10-21)

- **Build-time timezone range.** `TZ_OFFSET_MINUTES` is cast to i16
  in `config.rs` with no range check, so a wild value wraps and
  `Settings::DEFAULT` never goes through `is_valid`. Fix: a const
  assert against `TZ_LIMIT_MINUTES`.

### Review backlog, second pass (2026-10-23)

Bugs, in rough order of value:

- **Self-heal is unreachable once the lease expires on a dead
  uplink.** `probe_task` starts each iteration with
  `stack.wait_config_up()` (`net.rs:328`); when the lease expires
  embassy-net clears the address, no probe is sent, the streak
  freezes, the link is still up and no connect fails, so the clock
  sits with no page until power is cut. Left as is 2026-11-02: it
  needs a link to die in the last minutes of its lease. Fix, if
  wanted: `with_timeout(PROBE_PERIOD, wait_config_up())`, a period
  with the link up and no address scored as a lost ping, and the
  streak logic moved into `core` under test.
- **`POST /play` during an alarm silences it.** `buzzer_task`'s
  `Ring` loop breaks on any command, `Preview` plays once and waits,
  and the display re-issues `ring` only on the ringing edge. Fix:
  ignore `Preview` while ringing, or return to the ring loop after
  it.

Gaps:

- **A PIN issued while ringing or in alarm-set is dropped, not
  deferred.** `ui.show_pin` acts only in the clock, IP and PIN
  modes, but the `Signal` is consumed and `ota.rs:218` logs "shown
  on the panel" regardless. Fix: keep the pending PIN in `Ui` and
  show it when the mode returns to clock within `PIN_TTL_MS`, or log
  that it was not shown.
- **The connect budget is cleared by an association that does not
  survive.** `failures` and `first_failure` reset on any
  `connect_async` success, so an access point that associates and
  drops the station before DHCP finishes gives a loop in which
  neither budget fires and the probe never arms. Fix: clear the
  budget only once DHCP has an address or the connection has lasted
  some tens of seconds.

Nits:

- **`scan::stop` is not honoured on the DMA start-failure path.**
  The `Err` branch sleeps and `continue`s past the `STOP` check, so
  a start that kept failing would make `heal::reset` and
  `ota::install` spin. Unreachable with esp-hal 1.2.2. Fix: a `STOP`
  check before the retry.
- **First change on an empty flash writes the default record.** With
  `stored == None` the compare in `persist_task` is skipped. Fix:
  compare against the defaults when nothing was loaded.
- **A persistent `set_config` failure loops outside the connect
  budget.** The error path at `net.rs:162-166` sleeps and retries
  without touching `failures`. Unlikely to matter.
- **`FIRMWARE_VERSION` goes into JSON and HTML unescaped.** It is
  `git describe`, so a tag with `"` or `<` breaks `/api` and the
  page. Fix: sanitise in `build.rs`.
- **Silent fallback in the panel tables.** `com_index` returns slot
  0 for a value not in `COM_SEQ`. Fix: a `const` assert. Checked
  2026-11-17: the `GAMMA` half of the item went with the colours in
  levels.
- **In the ringing off-phase the degree mark stays lit.**
  `set_degree` runs before the match. Cosmetic.

### Review backlog, fourth pass (2026-10-30): the code since the second pass

Bugs, in rough order of value:

- **`POST /scan` can only answer within the socket's 5 s idle
  timeout.** `rescan` waits up to `SCAN_WAIT` (10 s) while the
  browser sends nothing, and smoltcp closes an idle socket whatever
  is pending to send; a scan slower than 5 s (about 2.6 s today)
  answers into a closed socket. Fix: raise the timeout for that
  request, or cut `SCAN_WAIT` under `TIMEOUT` and say so in the
  comment.

Gaps:

- **Without a usable `nvs` partition a provisioning save never
  leaves the access point.** The reset into station mode is in
  `persist_task`, spawned only when `Store::open` succeeded; with no
  store the setup page still says the clock will reset. A bad
  partition table only. Fix: answer differently, or reset from the
  listener when there is no store. Checked 2026-11-17: a failed save
  is retried now (`persist_task`'s backoff), so only the no-store
  case is left.
- **The access point's transmit ceiling is set before the driver is
  restarted.** `ap_task` calls `set_max_tx_power` and then
  `set_config(AccessPointStation)`, a mode change that stops and
  restarts the driver; whether the value survives is inside the blob
  and was not verified. Fix: move the call after `set_config`.

Nits:

- **The display loop wakes every 64 ms for nothing on the access
  point and while a PIN shows.** The redraw wait is `BLINK_STEP / 4`
  for every view but `Time` and `Ip`, though `View::Sweep` on the
  access point and `View::Pin` are static; each wake runs
  `carry_time`, a `settings::update`, a render and the image
  compare. Fix: wait `PIN_SHOW` for the PIN, and a second for the
  sweep when `net::mode()` is the access point.
- **`Store::open` logs inside the flash critical section.** The
  `error!` lines for a missing partition table or `nvs` entry run
  inside `with_flash`. Boot-only. Fix: move the match outside the
  closure.
- **A discover from a client that already holds a lease cuts the
  lease to the offer hold.** `Leases::handle` writes `now +
  OFFER_HOLD_SECS` (60 s) over an existing slot. Fix: keep the later
  of the two expiries.
- **`dnsd::answer` indexes `out` by the query's length.**
  `out[HEADER..n]` is checked only against `PACKET`; the task's
  buffers are both 512, so only a host test with a long query would
  panic. Fix: `get_mut(HEADER..n)?`.
- **`REQUEST_BUF` is tight for the setup form from a phone.** Head
  and inline body share 1536 bytes. Checked 2026-11-17: over the
  limit is a `431` now, and `BODY_MAX` is 640 after a test
  (`forms_fit_the_request_body`) measured the form at 605 bytes at
  worst (built and tested on the host, not on a clock). What is
  left is the head: 931 bytes at a body of 605, against some 750 for
  a phone's POST head before any cookie.
- **No error from `ap_task` is ever healed.** A failed `set_config`
  logs once and sleeps for ever, the panel showing `AP` with no
  network. Unlikely; a log line is probably all it is worth.
- **Stale comment.** `net::Mode`'s doc says the driver does not do
  AP+STA, which `ap_task` does for the scan. Checked 2026-11-17: the
  800 KB half of the item no longer holds (the image is nearly
  900 KB).
