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
  from nothing" in `HISTORY.md`). The web installer itself is the last item's.
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

From the rc6 soak (its section in `HISTORY.md`): the near access point went
away for a couple of minutes, the clock rejoined the house's far one
at −82 within 10 s, and stayed there when the near one came back at
−40. It left only because the far link got bad enough for the
self-heal's ten probes lost in a row, 19 minutes later; a link that
answers one probe in four would hold it for good, with MQTT dropping
and the pages mostly unreachable. A scan now and then while the RSSI
is poor, and a reconnect when a much stronger BSSID of the same
network is seen, is the obvious shape; nothing is designed. A
feature, so for `next`, not for master before v0.5.

## An mDNS responder: the clock is `<hostname>.local` — built and tested on the bench board 2026-11-26, on `next`

A network whose router does not put DHCP names in its DNS had no
name for the clock. Crates looked at: `edge-mdns` 0.8 with
`edge-nal-embassy` 0.9 would work with embassy-net 0.9 but brings the
`domain` crate, a second embassy-sync and `rand_core`, and does not
probe for the name either; `mdns-proto` needs an allocator to build a
message. The user chose a responder of our own, as the access point's
DHCP and DNS are.

- `core/src/mdns.rs`, pure and under test: `respond` reads a query
  (several questions, compressed names, the known answers) and writes
  the answer, `announce` the unsolicited one. Only the host's records:
  an A for `<hostname>.local`, and an NSEC saying A is all there is,
  so a lookup of AAAA is answered "none" instead of timing out. No
  services (DNS-SD), no IPv6, no reverse lookup.
- A query from port 5353 is answered to the group, or to the asker
  alone when it set the unicast bit; one from another port (a plain
  resolver) to the asker with its id and question, a TTL of 10 s and
  no cache-flush bit. A known answer with half the TTL left
  suppresses ours.
- `src/mdns.rs`, the task, station only: a socket on 5353 with a hop
  limit of 255, the group joined when the address is up (embassy-net's
  `multicast` feature, which is IGMP) and left when it goes, two
  announcements a second apart at each address. The name is read from
  the settings at each query; a changed hostname renews DHCP, which
  drops the address, so the new name is announced with no receiver of
  the settings. A query from outside the subnet is not answered.
- The parser is what any host on the network reaches: pointers go
  back only, a name is 255 bytes at most, responses are not read; a
  test mutates queries by the hundred thousand. Answers to the group
  are held to one of a kind a second.
- Not done: a goodbye before a reset (the address comes back the
  same and is announced with the cache-flush bit; only a renamed
  clock's old name lingers, 120 s), services for `avahi-browse`, the
  name on the access point, `cargo-fuzz` on the parser.
- Not done, by the user's decision (2026-11-26): probing for the
  name and renaming on a conflict. Two clocks of one name both
  answer and the name flips between them, the default `clock` on two
  clocks above all; a suffix would be a name no setting shows, given
  by boot order, and one any host could force. Looked at and not
  taken either: detecting a conflict and saying so on the pages, a
  default hostname with the MAC's tail. The manual says to name each
  clock.

Tested 2026-11-26 on the bench board (another-bench) and, at first the build
before the parser's hardening, on white-clock:

- The name resolves from Linux, macOS and Android (the user), and by
  `ping`, `avahi-resolve -n` and a plain resolver's A and AAAA
  queries from the session, in about 200 ms; the page opens by it.
- Two hours of load with `stack-watch`, one before the hardening and
  one after (each four alarms rung to the time-out, 43 clips
  previewed, the pages, `/api` and lookups four times a minute, MQTT
  connected): no panic, no underrun, no watchdog trip, and the stack
  no deeper than before the responder (`agents/sizes.md`).
- Against `HEAD`: 2008 bytes of the main stack (32 656 → 30 648),
  7.8 KB of `.text` and 0.8 KB of `.rodata`, IGMP with them.
- A hostname changed over `/api` on the bench board: the old name
  is no longer answered from that moment, the new one is announced
  when DHCP has renewed 3 s later and resolves from then (a plain
  resolver and Avahi; the page opens by it), and the same back.

Learned:

- Nothing of the stack may be asked inside `recv_from_with`'s
  closure, which runs with it borrowed: `config_v4` there panicked at
  the first datagram. The image had confirmed itself by then, so it
  did not roll back, and with some five mDNS datagrams a second on
  the home network no upload could finish between resets: white-clock
  had to be flashed over USB. The address is taken once at each
  config-up instead.
- Avahi does not read the NSEC: `getent hosts`, which asks for AAAA
  first, still takes its 5 s.
- One plain resolver's query, the first after a flash, went
  unanswered and was not seen again; not explained.

Owed: the hardened build looked at on white-clock, which has it
since the evening of 2026-11-26; the name from Windows, and after
the network's access point restarts.

## The module player's effect R — built 2026-11-26, on `next`; held to openmpt123 on the host, not heard on a clock

FastTracker's R (retrigger every y ticks with a volume change x) was
the one effect the pack maker rewrote: as E9 where it changed no
volume, and not played otherwise. Of the 937 XM modules of the
user's key-generator collection some ninety use it, most with a
volume in the column or an x that changes one.

- The player has it (`Tune::retrigger`) as OpenMPT plays a
  FastTracker module's (`CSoundFile::RetrigNote` of 1.32, its
  compatible playback). It was first built as FastTracker 2.09's own
  replayer has it, read in 8bitbubsy's ports (ft2play, ft2-clone);
  the two differ in corners, and the user chose OpenMPT, which is
  what a module is heard in today. The rules: x and y remembered
  apart; the ticks counted across rows on a counter E9 shares,
  looked at before they are counted, and started over only by a
  retrigger; on a row's first tick a cell with an instrument sets the
  count to 1, a volume in the column (not 0) ends the tick, and a
  note in the cell is not begun twice; under a volume in the column
  x changes nothing; 6 is five eighths and E three halves, in
  quarters of a volume step, which a run of R rows keeps; the note
  begins again at its own period, on the sample it had. FastTracker's
  replayer starts the count over at every instrument, counts a row's
  first tick only where the column leaves nothing behind, and has a
  half, an eighth and a sixteenth for 6.
- The pack maker keeps R as it is. For `--firmware 0.5` it writes E9
  as before and names any other R. `clips/lera.xmc` stays the clip
  for v0.5: the player makes the same samples of its E92 as of the
  module's R02, so the buzzer's pack keeps its rolls on an older
  clock.
- Checked on the host, tick by tick, against openmpt123 0.8.10: a
  module's samples made a ramp, a tick's slope in the render is
  volume times pitch and a note's start is seen
  (`dump_the_ticks` is the player's side). A module of 209 rows
  written for it (every x, the memories, the volume column's sixteen
  kinds, a portamento before, other speeds, E9 between) is the same
  on every tick but where the measure fails (a volume slide, which
  OpenMPT ramps across the tick). Of the 169 channels with an R of
  89 modules of the collection and n1k-o's, 156 are the same on every
  tick of R; the six of the other 13 looked at are the measure's
  (notes too low for the ramp's steps, a pattern delay the player has
  not, which shifts the rows). The test's rows are each a render of
  their own.
- OpenMPT plays a module it takes for an old ModPlug Tracker's by
  other rules. A module rebuilt for the check with FastTracker's
  name in its header was taken for one, and 76 channels differed
  until the name was another; the player has the compatible rules
  only.
- Also run on the host: the players' tests with clips built in, the
  repository's eight and 70 of the collection's with an R (mixed by
  runs as a sample at a time, silent channels passed by); 150 tunes
  of random cells, half with R, the same ways and with a 16-bit
  sample predicted or plain; the player of v0.5-rc7 over a pack with
  R in it, which it takes and plays as the same module with no R.
- Cost: 612 bytes of flash, 12 of `.rwtext`, 88 of the main stack.

Owed: a module with an R that changes the volume heard on a clock.
ScreamTracker's Q, the same effect with another memory, is still not
converted.

## A line output on the adapter for an amplifier — the setting built and heard 2026-11-18; the shutdown pin built, not checked; the adapters ordered 2026-11-19

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

## Checks still owed

A line is struck or deleted when its check is made or no longer
wanted, with the result put in the section.

- The pack maker (2026-11-19): the command in a user's hands.

Sections in `HISTORY.md`, by title, with what each still owes (read
again against the sections on 2026-11-24, when both files were
condensed and the built sections moved there):

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
  bench board 2026-11-19, longest feed gap 1180 ms); a preview while
  the clips are under suspicion, a pack upload's clearing of it, and
  `clips_paused` seen in Home Assistant.
- **Sixteen channels for the module player** — the 31 250 clip as a
  ringing alarm for five minutes; the version line of a changed
  tree's build lacks `-dirty`.

- **An install from nothing on the bench, access point included** —
  the buttons (the 10 s hold, the IP display), the panel (`AP`, the
  PIN's digits, the blank station), a phone's sign-in sheet.
- **The install run's findings fixed** — the forms from a phone.
- **A preview never interrupts the alarm** — the ear, the page's own
  buttons, a press ending the ring.
- **A MOD converter** — heard on a clock, with any module.
- **A pack's clip this firmware does not play is skipped** — on a
  clock, a pack with a clip of an unknown version; the user's word on
  which branch takes it.
- **"Starlight & Aurora" clicks on a speaker** — a stock buzzer at
  128 bits a sample and 31 250 Hz.
- **The installer asks: buzzer or speaker** — an install of each
  choice from the page.
- **16 bits in 8 by prediction** — a sound file made with the
  defaults on a clock; a module with 9xx on a 16-bit sample played
  on a clock; the 4-bit residuals weighed.
- **The module player's cost** — a SID tune on a board; the deepest
  stack use after the render into the chunk; the figures on a clock
  with a panel.
- **A 24 h soak of the v0.5-rc6 release** — the uploads the soak of
  2026-11-18 ended with.
- **A keygen-style tune of our own for the alarm, "Serial Dawn"** —
  the user's ear on the tune, its levels and the limiter's settings.
- **n1k-o's "lerka" the default pack's third clip, "Lera"** — the
  buzzer's pack on a clock; the `clip:` line for underruns at 18
  channels.

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

### Found with the 16-bit samples (2026-11-20), in the shaper's high-pass

From "Samples of 16 bits in a module" in `HISTORY.md`; both in
`core/src/shape.rs`, neither heard.

- **A speaker's module clip carries a constant offset.** About -580
  of 32512 (1.8 % of full scale): the section's `>> 14` rounds down
  and the feedback's gain at 0 Hz is about 2500 for 100 Hz at 31 250
  (a ninth of that at the buzzer's 300 Hz). Fix: round the shift, or
  carry the remainder.
- **`highpass=20` gives a clip stuck at full scale.** The Q14
  coefficients cannot hold a corner that low. Fix: refuse it in the
  pack maker, or wider coefficients.

### Found with the key-generator collection (2026-11-26)

The pack maker and the player over the 937 XM modules of the user's
collection, for the effect R: 563 convert and play to their ends (17
hours), none panics or hangs, 387 are refused in words (245 for the
Amiga frequency table, 59 for more than twenty channels). Not R's,
and not fixed:

- **245 modules are refused for the Amiga frequency table**, a
  quarter of the collection and two thirds of what is refused: the
  player has the linear table only. Not designed. Either the player
  gets the table (a note's period from FastTracker's table and its
  finetune, the step from 8363 * 1712 over the period once a tick,
  slides, vibrato and arpeggio in those units, and a clip that says
  so, which an older firmware must not take for a linear one), or
  the pack maker writes such a module on the linear table as
  `modxm.rs` does a ProTracker one's slides, each converted at the
  period it is played at, which is no firmware's business and not
  exact.
- **A module that fades in is silent.** Three start with G00 and
  raise the global volume with H, which the player has not (the pack
  maker names it), so the tune is played at nothing to its end. Fix:
  H in the player, or the pack maker leaving a G00 out where only an
  H follows it.
- **80 modules are refused as cut short that OpenMPT plays.** The
  header counts more instruments than the file has: 66 end where an
  instrument would start, the rest inside a header or a sample (two
  by a few bytes). Fix: an instrument at or past the end is one with
  no sample, and a sample is what there is of it, as `Data::slice`
  already takes a run of bytes.
- **OpenMPT plays 28 of the 99 modules with an R by other rules**,
  those it takes for an old ModPlug Tracker's (`openmpt123 --info`);
  the player has the FastTracker-compatible rules for every module,
  so those sound otherwise on the clock than in OpenMPT. Nothing to
  fix unless a clip is to say which tracker made it.
