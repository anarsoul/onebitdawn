# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Releases are annotated `v*` tags; each one has its images on the
[GitHub release](https://github.com/anarsoul/onebitdawn/releases) and
in the [web installer](https://anarsoul.github.io/onebitdawn/).
A running clock takes the `onebitdawn-ota-<tag>.bin` of a release
from `http://<ip>/update`; its settings survive the update.

## [Unreleased]

### Added

- The module player has FastTracker's effect R, the retrigger with a
  volume change (on `next`, for the release after v0.5), as OpenMPT
  plays it: the count of ticks carried from row to row, each of x's
  fifteen changes, the volume column's say in it. The pack maker
  keeps a module's R as it is; for a clock
  at v0.5 (`--firmware 0.5`, or the page's choice) it goes on writing
  E9 where an R changes no volume and names any other as not played.

- The module player has FastTracker's effect H, the global volume
  slide, as OpenMPT plays it (on `next`, for the release after v0.5):
  a module that fades in from G00 was silent to its end. The pack
  maker names it as not played only for a clock at v0.5.
- The module player has FastTracker's effect EE, the pattern delay,
  as OpenMPT plays it (on `next`, for the release after v0.5): a
  tune with one ran on early and its clip came out short. The pack
  maker counts the delay into a clip's length.
- The module player has FastTracker's effect E6, the pattern loop,
  as OpenMPT plays it (on `next`, for the release after v0.5): a
  tune with one skipped its repeats and its clip came out short. The
  pack maker counts the loops into a clip's length and refuses a
  module whose loops never end.
- The module player has FastTracker's Amiga frequency table, as
  OpenMPT plays a module on it (on `next`, for the release after
  v0.5): the pack maker refused such a module, a quarter of a
  collection of key generators' music. For a clock at v0.5 it now
  converts one as well and says that the slides follow the linear
  table: the notes are right there, the slides and vibratos not.
- An arpeggio over a note that has slid plays the notes nearest to
  where it slid to, as FastTracker's does.
- An instrument's number with no note gives a channel back the
  volume of the instrument that is sounding, and the instrument
  named is the next note's, as in FastTracker: it took the named
  one's volume at once, and whole channels were too loud or too
  quiet for it.
- The module player has FastTracker's tremolo (7), tremor (T),
  extra fine portamento (X1, X2), envelope position (L) and
  finetune (E5), as OpenMPT plays them, and E1 and E2 remember what
  they last slid by (on `next`, for the release after v0.5). The
  pack maker names them as not played only for a clock at v0.5; it
  did not name the tremor at all, and no longer names what does
  nothing in FastTracker (E0, EF) or on one speaker (E8).
- An order that names a pattern the module has not, an empty one
  FastTracker did not save as a rule, is passed by, as OpenMPT
  passes it: it was 64 rows of silence, at a tune's end or its
  start.

### Changed

- On the line output the note melodies and a beeper clip (Trantor)
  are no louder than the other clips (on `next`, for the release
  after v0.5): they go through the chip's converter as samples at a
  level, where they were the pin from rail to rail, some 15 dB over
  the amplifier's pack. The level is a setting, "Beeps volume on
  amplifier" on the settings page and `beep_volume` in `/api`:
  `quietest`, `quiet` (the default), `loud` or `loudest`. The buzzer's
  sounds are as they were.
- A clip on the buzzer's pin goes through the chip's converter at 128
  bits a sample and no longer at 256 (the line output keeps the
  build's): a speaker in the buzzer's place, on the buzzer's
  transistor, hissed under any sound, and hisses less with half the
  edges.
- The module player no longer clicks where a note is cut by the next
  on its channel or a level changes: both are ramped over a few
  milliseconds, as FastTracker does. Keygen is played
  with it too.
- **The project is now called onebitdawn** (it was esp32clock; the
  releases up to v0.4 and their files keep that name). What changes
  for a clock that is updated:
  - the MQTT topics are `onebitdawn/<id>/…`, not `esp32clock/<id>/…`.
    Home Assistant follows through discovery, the device and its
    entities being the same ids; a broker's ACLs and anything that
    subscribes or publishes by topic need the new prefix, and the
    retained `esp32clock/<id>/status` and `state` stay on the broker
    until cleared (`mosquitto_pub -r -n -t esp32clock/<id>/status`);
  - the access point's default password is `onebitdawn`;
  - the release's files are `onebitdawn-<tag>.bin`,
    `onebitdawn-ota-<tag>.bin` and `onebitdawn-clips-<tag>.bin`;
  - the pages are headed `onebitdawn: <hostname>`.

- **The prebuilt images have no API token** (it was `esp32clock`,
  printed in the manual, so anything on the network could update a
  clock that kept it). A clock flashed with one updates from its
  pages with the PIN on its panel, and the scripts work once a token
  is set on `/system`. A clock that already stores a token keeps it.
- A clock keeps its settings, the network above all, when it is
  rolled back from a later firmware to this one or after it: a value
  in the stored record that the older firmware does not know costs
  that one setting, which goes to its default, and no longer the
  whole record, and the count that rolls a bad update back survives a
  change of the record it is carried in.
- The README is condensed to what the project is, how it differs from
  the original, how to build the clock and what the clips are. The
  rest (building from source, updates over the LAN, clip packs, the
  buttons, the pages, `/api`, Home Assistant) is now `MANUAL.md`.

- The clips (the alarm melodies that are not notes) are no longer
  part of the firmware: they are a *clip pack* in a new 1152 KB flash
  partition, `clips`: half a megabyte from each of the two firmware
  slots and the 128 KB at the end of the flash that nothing used. A pack is made with `onebitdawn-pack` or on its web page (up to
  16 clips), checked by a SHA-256 at upload and at every boot, and
  played in place from the flash. The default pack, the eight clips
  below, is `onebitdawn-clips-<tag>.bin` on a release, and the web
  installer writes it.
- **A clock flashed before this release has the old partition table,
  which an update over the LAN does not change.** It runs this
  firmware with the note melodies only, and an alarm set to a clip
  plays the beeps, until it is flashed once over USB or from the web
  installer (the settings survive). `ALARM_CLIP` at build time still
  builds clips into the firmware, for such a clock; it is empty by
  default now.

### Removed

- The software modulators for a PCM clip or a module, and their build
  variables `CLIP_MODULATOR`, `CLIP_KBPS` and `CLIP_CARRIER_KHZ` (on
  `next`, for the release after v0.5): every such clip goes through
  the chip's PCM-to-PDM converter, as it has in every prebuilt image.
  `CLIP_PDM_BITS` stays.

### Fixed

- The pack maker converts an XM module whose file ends before its
  instruments do (the header counts more than there are, or the last
  sample is cut) and one whose instruments give a wrong size for
  their samples' headers, as FastTracker and OpenMPT play them; it
  refused both as cut short (on `next`, for the release after v0.5).

- A timezone typed as `UTC+3` or `GMT+3` is three hours east, as it
  is written everywhere but in POSIX, and is stored as the rule
  `<+03>-3`. It was taken as a POSIX rule, where the sign counts west,
  so the clock ended up at UTC-3. Rules proper are read as before
  (`MSK-3`, `GMT0BST,M3.5.0/1,M10.5.0`).

- The web installer's dialogs follow the browser's dark theme, where
  they stayed white with the page's light text on them; their buttons
  have the page's accent colour.
- A request too long for the clock is told so, where the connection
  was closed with no answer: `431` for a head over 1536 bytes, `413`
  for a form's body over the limit, `400` for a head that does not
  parse.
- The setup form on the access point comes back filled in after it
  is refused (the password apart), where every field had to be
  entered again.
- `/api` refuses a `password` with no network to use it on, which an
  unprovisioned access point used to save.
- A clip no longer plays an octave down and stuttering, which every
  sampled clip, module and chip tune did now and then after a beeper
  clip had been played, until one was played again.
- A beeper clip plays at its own rate every time. On
  every play but the first after a reset the line ran 0.16 % off the
  clock the clip is fed by, which a long ring would have run into.
- Play no longer interrupts a ringing alarm (it played the melody
  once and left the alarm silent): Play, Stop and the MQTT `play`
  command do nothing while the alarm rings. A button on the clock or
  the MQTT `silence` command ends it.
- The System page's line after an upload or a reboot has its full
  stop.

### Removed

- Trantor, the ZX Spectrum beeper tune, is no longer in the default
  clip pack: the right to distribute it is not to be had. A clock
  whose alarm had it chosen rings its default melody after the new
  pack is written. The beeper player stays, for a pack of your own.
- Halftime, Dualtrax's other chiptune, is no longer in the default
  clip pack either: no licence for it is known, and its author could
  not be reached. A clock whose alarm had it chosen rings its default
  melody after the new pack is written.

### Added

- The clock answers multicast DNS (on `next`, for the release after
  v0.5): it is `<hostname>.local` on the network, `clock.local` for
  the default name, whatever the router does with DHCP names.
- The web installer asks whether the clock has a buzzer or a speaker
  and writes the default clip pack made for that.
- Sound of 16 bits in about a byte a sample. The pack maker keeps a
  `.wav` and a module's 16-bit samples as what is left of each sample
  after the samples before it have predicted it, with predictors it
  fits to the sound: a byte a sample and a sixteenth more, and for a
  piano some 25 dB nearer the 16 bits than 8 bits are. It is what both become unless
  told: `format=pcm16` and `samples=16` keep the 16 bits as they are
  at twice the bytes, `format=adpcm`, `format=pcm` and `samples=8`
  make what there was. The new kinds are clips of new versions, which
  a firmware up to v0.5 refuses with its pack; a `.wav` is now 17 KB
  of the pack a second and not 8.
- A module's 16-bit sample that holds 8 bits (its low bytes all the
  same) is written as 8 bits, the same sound in a clip every firmware
  plays.
- "Suspended Seconds", "Starlight & Aurora" and the three clips of
  Grieg have their instruments at 16 bits and their clips so; the
  buzzer's pack is 747 KB and the speaker's 978 KB of 1152.
- A release has `onebitdawn-licenses-<tag>.txt`: the licences of the
  crates its files are built from, and of the bootloader and the
  radio's libraries in the images. The installer's and the pack
  maker's pages link it.
- A clip of a version the firmware does not play no longer costs the
  whole pack: the clock skips it and takes the rest, and the upload's
  answer and the Clips page say how many it left out. (A clock at v0.5
  or before still refuses such a pack.)

- Three more clips in the default pack, made for this project from
  Grieg's first *Peer Gynt* suite: "Morning Mood" with an orchestra's
  instruments and again as a chiptune, and "Mountain King" (In the
  Hall of the Mountain King).
- A sixth, "Suspended Seconds", a piano tune by Vasily Khoruzhick
  written for the clock, played with its velocities and its sustain
  pedal. The tune is its composer's and not under the project's
  licence (`clips/README.md`).
- A seventh, "Serial Dawn", a chiptune in the manner of the key
  generators' music, written for the clock: a rising arpeggio, then a
  pulse lead over tonic, subdominant and dominant, of waves made by
  its script and nothing recorded. A ringing alarm plays it round
  from after its introduction. It is the project's, under its licence.
- An eighth, "Lera", a chiptune by n1k-o ("lerka", written for
  TSRh's key generators), played from its tracker module, here with
  his permission as "Synchronization" is; the music is his and not
  under the project's licence (`clips/README.md`). It is the pack's
  third clip. The pack is eight clips and 697 KB.
- The pack maker writes a module's effect R (retrigger) as E9, which
  the player has, where it changes no volume; until now such a
  module's rolls were single hits.
- The speaker's pack has a ninth, "Starlight & Aurora" (on the pages
  "Starlight and Aurora"), his too and under the same terms: a soft
  piano, a bell, a lead and a bass, which a buzzer cannot say. That
  pack is 919 KB.
- The alarm sound is stored by its name and no longer by its place in
  the list: an alarm keeps its clip through a new clip pack that still
  has it, in whatever order, and through a firmware with more note
  melodies. A sound no pack has any more is the beeps, as before. A
  firmware rolled back to goes by the place, as it always did. The
  stored settings are written once in the new layout at the first
  boot after an update, so this holds from the update on.
- A watchdog. A clock whose firmware hangs resets itself within a
  minute, where it used to stay dark or frozen until the power was
  cut, and an update that hangs is rolled back like one that never
  reaches the network. `/api` has the count of such resets since
  power-up as `watchdog`; the serial log's boot line and heartbeat say
  what caused the last. If it was a clip that hung the clock, the
  alarm and the previews play notes only until a firmware update, a
  new clip pack or a power cycle, so a bad clip cannot silence the
  alarm; the clock page says so, `/api` has `clips_paused` and Home
  Assistant a "Clips paused" binary sensor.
- A sampled clip can be IMA ADPCM, four bits a sample: half the flash
  of the same clip as PCM, 8 KB a second at 16 kHz.
  It is what the pack maker makes of a sound file. A firmware before this one plays
  such a clip as noise.
- The Wi-Fi transmit power is a setting (`tx_power` in `/api`, a
  select on `/settings`): the driver's levels from 2 to 16 dBm, 16
  being the default and the most there is, for a clock whose antenna
  works better with less. Too little and the clock cannot reach the
  network; the access point (both buttons for 10 s), which always
  transmits at 11 dBm, has the setting on its setup page.
  `WIFI_TX_POWER_QDBM` at build time is the default of
  a clock with nothing stored.
- A fifth page, `/clips`: the pack in use and its clips, and the
  upload of another with the PIN from the panel (`POST /clips`, also
  `scripts/clips.sh <ip> [pack]` with the token). The clock restarts
  into the new pack. `/api` has a read-only `clips` key, `pack` or
  `builtin`.

- Two more alarm melodies that are not beeps: "Keygen", a chiptune by Dualtrax (from
  ORiON's key generators; CC BY 3.0), played from its tracker module by a player in the
  firmware, and "Synchronization" by n1k-o, a ZX Spectrum AY tune
  played from its Pro Tracker 3 module by a player and an AY-3-8910
  in the firmware, here with his permission. They go out of the buzzer pin as a bitstream from I2S (the
  modules and PCM clips through its PCM-to-PDM converter, at 256 bits
  a sample; `CLIP_MODULATOR=sd` at build time is a software
  sigma-delta at a lower rate, for a buzzer circuit that asks for
  one), and are in the random draw, and a ringing alarm plays them round and
  round with no gap, as they are written to loop. The music is its
  authors'; see `clips/README.md`.
- A beeper clip is packed (`BPR3`): runs that were played before are
  a reference to them, read in place from the flash: a tune of 146 KB
  is 68, with the same pulses on the line. `scripts/zxbeeper.py` writes it, and packs an older clip
  given in a tape's place; the older format still plays.
- A SID tune's record is packed the same way (`SID2`,
  the pack maker's): a tune of 62 KB is 12.
- A pack maker for your own tunes, `onebitdawn-pack` (`tools/pack`):
  one command, built for Linux, Windows and macOS on a release, and
  one web page, <https://anarsoul.github.io/onebitdawn/pack/>, the
  same converters as WebAssembly, with a preview through the clock's
  own players. Tracker modules (`.xm`, `.mod`, `.s3m`), Commodore 64
  tunes (`.sid`, PSID: the firmware has a SID chip of its own to play
  the tune's register writes through), ZX Spectrum ones (`.pt3`, Pro
  Tracker 3 or Vortex Tracker II: the firmware has the player and an
  AY-3-8910 of its own) and sound files (`.wav`), each shaped for the
  buzzer or, with `--sound-output speaker`, for a speaker. A ZX
  Spectrum beeper recording is `scripts/zxbeeper.py`'s still.
- `ALARM_CLIP` at build time chooses clips built into the firmware,
  any that a pack takes (`onebitdawn-pack --clip` makes one alone).
- A Stop button beside Play on the clock page (`POST /stop`): it
  ends a preview.
- The MQTT `silence` command, and its button in Home Assistant, stops
  a preview as well as a ringing alarm.
- `CLIP_PDM_BITS` at build time (128, 256, the default, or 384) sets
  how many bits the PDM converter makes of a sample, and
  a module's `knee` option where its limiter starts, up to
  off: for a speaker in the buzzer's place, where the buzzer's hard
  limiting is heard as overdrive.
- For the same speaker: `scripts/speaker-clips.sh` makes the default
  clip pack with the modules' bass left in and a slower limiter, which
  is the pack maker's `--sound-output speaker`, and a release has
  that pack as `onebitdawn-clips-speaker-<tag>.bin`; its `flat` option
  makes a SID tune without the limiter (a firmware older than this
  plays such a clip limited).

## [0.4] - 2026-11-10

The display scan runs in hardware. Nothing new is configured; updating
keeps every setting.

### Changed

- The panel is scanned by one SPI transfer the hardware repeats by
  itself, a DMA descriptor ring in the ESP32-C3's segmented-transfer
  mode, with no CPU in the loop. The dark gap between two laps is a
  fixed 0.6 µs. Before, the firmware restarted every transfer, which
  took 10 to 19 µs each time and up to 1 ms whenever Wi-Fi, the heap
  or the log held interrupts off. The minute heartbeat's `scan` field
  reports images published, stops and the descriptor the DMA is at,
  in place of the gap statistics.
- The thermometer offset defaults to −2.5 °C instead of 0, the typical
  difference between the sensor in the case and the room. It applies
  to a clock that has no settings yet (a fresh flash, or the web
  installer with the erase chosen) and to one updated from a firmware
  older than the offset setting; a clock that already has settings
  keeps its offset, 0 included.

### Fixed

- A bare `curl -X POST` to `/reload`, `/ap`, `/pin` or `/scan` works
  again. v0.3 answered `411 Length Required` to every `POST` without
  `Content-Length`, and these four read no body, so
  `scripts/reload.sh` and `scripts/pin.sh` were refused. Only the
  routes that read a body (`/`, `/api`, `/play`, `/token`, `/ota`)
  ask for the length now.

## [0.3] - 2026-11-07

Fixes from a review of the v0.2 code. Nothing new is configured;
updating keeps every setting.

### Changed

- A `POST` without `Content-Length` is answered `411 Length Required`,
  and one with any `Transfer-Encoding` is answered `400 Bad Request`,
  before a route is looked at. Before, a bodiless or chunked post to
  the settings page was taken as an empty form, which cleared the
  checkboxes (alarm armed, 12 hour, skip) and saved them.
- The page's buttons sit at the same distance under a field or a
  fieldset as any field does; the shared stylesheet had stacked a
  button's own top margin on the block above it.
- The settings record that cannot be decoded is logged with its
  version byte and length only. The whole record used to be printed,
  and it holds the Wi-Fi password, the API token and the MQTT password
  in the clear.
- The README says the project was written with the assistance of an AI
  agent, from the author's design, measurements and review.

### Fixed

- A timezone rule with a standard offset past 14 hours (`<+15>-15`)
  was accepted, saved, and thrown away at the next boot with the whole
  record: network, token, broker and alarm. The rule is kept now, and
  an image older than this one reads it as a fixed rule clamped to
  its own range.
- A timezone transition that falls in the neighbouring calendar year
  (a rule like `J1/-1`, whose first transition is 23:00 on December
  31) is now evaluated in the right year; the evaluator takes the
  transitions of the year before and after as well.
- The **Use this browser's zone** button no longer calls a zone fixed
  because its January and July offsets match, and no longer returns a
  rule that fails to predict the six following years. It finds every
  offset change of the year; a zone with other than none or two, or
  one no candidate rule predicts (Casablanca's Ramadan hour), shows
  `no rule for <zone>` instead of a confident wrong rule.
- A broker host or hostname changed while the MQTT session was
  starting up was dropped unread, leaving the clock connected to the
  old broker for good or Home Assistant with stale discovery. The
  change is compared after the start now.
- The MQTT session checks the broker's SUBACKs: a subscription refused
  (reason code 0x80 or more) or unanswered within 10 s fails the
  session with the reason on the page, where before the page said
  `connected` and the controls did nothing.
- A settings save that failed was not tried again, so the unsaved
  settings lived in RAM until something else changed; on the access
  point a setup form could answer "saved, joining" and the clock stay
  where it was. A failed save is retried after 2 s, doubling to 60 s.
- Two Rescan requests at once on the access point (a double click, or
  a phone's sign-in sheet and a browser tab) each took the other's
  wake-up, and one waited out the timeout. Every waiter is now
  released by the next scan to finish.
- A firmware upload failing during the pause of a deliberate reset
  (self-heal, reload, OTA reboot) restarted the display scan, so the
  reset landed with a transfer in flight and the last row stayed lit
  at full current until the firmware was back. The stop is latched
  against the upload's resume.

### Security

- The settings page refuses to move the MQTT broker to another host
  unless the password is typed again, with the notice "a new broker
  needs its password typed again". Before, an unauthenticated post to
  the page could move the clock to a broker of the poster's choosing,
  and the clock handed it the stored user and password. The same host
  keeps its password on an empty field, an empty host clears the
  credentials, and the token and the setup form are exempt.

## [0.2] - 2026-11-07

### Added

- The timezone setting is a POSIX TZ rule (`PST8PDT,M3.2.0,M11.1.0`,
  `CET-1CEST,M3.5.0,M10.5.0/3`, `<-07>7`), evaluated on the clock from
  the NTP time, so daylight saving needs nothing from the network.
- The settings page and the setup form have a **Use this browser's
  zone** button that fills the rule in from the browser's own time
  zone and names the zone it used. The rule is derived from next
  year's transitions and checked against the six years after, so
  zones that change at midnight (Chile) or on a negative hour
  (Greenland) come out right, and a zone that is dropping daylight
  time this year comes out fixed.
- The status block shows the local time the rule gives, with the
  zone's name and the offset in force, for checking a rule.
- `/api` gains the read-only `utc_offset` (minutes east of UTC in
  force) and `dst` (whether daylight time is in force), both `null`
  before the first time sync.
- Build: `TZ_RULE` seeds the rule and a bad one fails the build.
- The README points at the installer first and walks through an OTA
  update.

### Changed

- The `tz` key of `/api` takes a rule or, as before, minutes east of
  UTC, which become a fixed rule; the JSON answers `tz` as a string
  now. `TZ_OFFSET_MINUTES` still works at build time as a fixed rule.
  The prebuilt images stay on Vancouver time without daylight saving,
  now written `<-07>7`.
- Updating from v0.1 keeps every setting: the stored offset becomes
  the fixed rule with the same minutes.

### Fixed

- The alarm handles a transition: an alarm minute that the spring
  jump skips rings when the clock lands past it, and the hour that
  the autumn change repeats rings once.
- The installer flashes four parts (bootloader, partition table,
  erased `otadata`, the application) and leaves the `nvs` partition
  alone, so a reflash keeps the saved settings unless the erase is
  chosen. The v0.1 installer wrote one merged image, which cleared
  them.

## [0.1] - 2026-11-07

The first release, for the HU-058 LED clock kit on an ESP32-C3.

### Added

- Time over NTP, 12 or 24 hour, with a fixed UTC offset.
- Alarm with a choice of melodies and a random pick, per-weekday
  arming, a one-time skip, set from the buttons or the page.
- Nine colours and three brightness steps, with a build-time current
  ceiling for the supply.
- Room temperature and light level from the kit's NTC and LDR
  (optional), the temperature shown on the panel once a minute, with
  an offset setting for the case's warm air.
- Settings page at `http://<ip>/` and a form-encoded JSON API at
  `/api`; a DHCP hostname.
- Wi-Fi provisioning from the clock's own access point with a captive
  setup form, and a 10 s button hold to get back there.
- Home Assistant integration over MQTT with discovery: the settings as
  entities, the readings as sensors, buttons for play, silence and
  reload.
- Firmware updates over the LAN from `/update` with a PIN shown on the
  panel, or from a script with a token; two application slots with
  rollback of an image that does not come up.
- Self-healing network: the clock resets itself when the gateway
  stops answering or the join keeps failing, and carries the time and
  the alarm state across the reset.
- A web installer on GitHub Pages and a release workflow.

[Unreleased]: https://github.com/anarsoul/onebitdawn/compare/v0.4...HEAD
[0.4]: https://github.com/anarsoul/onebitdawn/compare/v0.3...v0.4
[0.3]: https://github.com/anarsoul/onebitdawn/compare/v0.2...v0.3
[0.2]: https://github.com/anarsoul/onebitdawn/compare/v0.1...v0.2
[0.1]: https://github.com/anarsoul/onebitdawn/releases/tag/v0.1
