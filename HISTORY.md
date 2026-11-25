# onebitdawn — history

The design record of every milestone, in the order written. Until
2026-11-17 this was `PLAN.md`, and the text calls itself that; what is
still open stayed there. A heading has its status as of when it was
last touched. Sections 1 to 4 are the design the project started from;
`ARCHITECTURE.md` is the firmware as it is now.

## ESP32-C3 clock for the HU-058 panel — implementation plan

Target: a static `12:34` on the HU-058/HU-058D panel from an ESP32-C3,
in Rust with esp-hal and Embassy, the scan offloaded to SPI2 in quad
mode with DMA. Reference: `3rdparty/HU-058_ESPHome/docs/` and
`firmware/`.

## 1. What the panel needs (from the reference)

- Two AiP33628 drivers, one per two-wire bus; driver 1 is the hours,
  driver 2 the minutes. Each is a 30-bit shift register and latch, no
  strobe, no commands. Frame, LSB first: `SS[15:0]` (SEG sinks),
  `CS[7:0]` (COM anodes), `IS[3:0]` (current, bit-reversed on the
  wire), 2 zeros.
- Shift on CLK rising; DATA changes only while CLK is low. Latch is a
  DATA rising edge with CLK held high after bit 29
  (`panel-test/src/main.cpp:sendFrame`).
- The MCU multiplexes four COM pairs (`0x30, 0x0C, 0x03, 0xC0`); CS
  and IS are the same for both drivers in a slot.
- LEDn = SEG(3n-2..3n), channel = SEG index mod 3 (1 blue, 2 green,
  0 red); tables verbatim from `display-map.md` / `aip33628.cpp`.
- `IS` from brightness only; about step 11 at most, the board is
  rated 250 mA.

## 2. Hardware offload design: SPI2 quad mode as a 4-channel pattern generator

SPI's SCK cannot make the latch, so all four wires are quad-SPI data
lines and SCK goes nowhere; each SPI clock period puts out one nibble,
the whole state of the wires. Bit 0 SIO0 GPIO9 DATA, bit 1 SIO1 GPIO8
CLK, bit 2 SIO2 GPIO7 DATA_1, bit 3 SIO3 GPIO6 CLK_1; this and the
high-nibble-first byte order were confirmed on the panel 2026-10-21.
A frame to both drivers is a fixed byte string, the dwell is zero
padding, and timing is cycle-exact, free of task and ISR jitter (the
ESPHome component needed a gptimer ISR for its dithering).

### Frame encoding (per 30-bit frame, both drivers)

Three nibbles a bit (data with clocks low, clocks high, clocks low
with data stable), not two: a DATA change in the nibble of a CLK
falling edge could read as a latch. Bit 29 leaves CLK high, then DATA
low, high (latch), low, CLK low. 93 nibbles, 47 bytes. The chip's
minimum phase is 16 ns.

### Scan cycle buffer

First design: four 600-byte slots (frame, then zeros) and a blanking
frame, 2447 bytes at 2 MHz, about 409 Hz. The blanking frame is there
because the latch holds its pattern for ever: without a last `SS = 0`
frame the fourth COM pair stays lit through the gap between transfers,
brighter than the others and pulsing with gap jitter. The rule since:
the last thing latched before any gap is dark.

### Idle-line safety

The peripheral may release the lines between transfers, so each pin is
a `Flex` with `Pull::Down`, driven low, before `with_sio0..3`.

### esp-hal API (verified against esp-hal 1.2.2 source)

`Spi::new` with no `with_sck` or `with_cs`; `with_sio0..3` need
`PeripheralInput + PeripheralOutput`, which `Flex` is and `Output` is
not; `half_duplex_write_buffer(DataMode::Quad, Command::None,
Address::None, 0, len, buf)`. All behind `unstable`.

## 3. Project layout (Rust, project root = this directory)

Scaffolded with `esp-generate` (`-c esp32c3 -o embassy -o unstable-hal
-o esp-backtrace -o log`), esp-rtos as the runtime; `aip33628.rs`,
`panel.rs`, `scan.rs`, `bin/main.rs`. Crates: esp-hal 1.2.2, esp-rtos
0.4.0, esp-bootloader-esp-idf 0.6.0, embassy-executor 0.10.

## 4. Other pins (configured in milestone 1 so nothing misbehaves at boot)

GPIO5 the buzzer (PNP base via 10 k, emitter at +5 V): open-drain,
released at boot, low = on. GPIO4 the upper and GPIO3 the lower
button, to ground, internal pull-ups. GPIO0 the NTC and GPIO1 the LDR
dividers (adapter 10 k to 3V3), ADC1 at 11 dB with `AdcCalCurve`
(M11a, 2026-11-01). GPIO8 and GPIO9 are strapping pins, driven only
after boot and not loaded by the drivers, so flashing is unaffected.

## 5. Milestones

### M1 — static `12:34` — DONE, verified on the panel 2026-10-21

Encoder, panel tables and font, cycle builder, scan task; `12:34` in
red at `IS` step 5 (15 mA). The nibble table was right as written and
the last slot's segments were no brighter than the rest.

### M2 — real time over NTP — DONE, verified on the panel 2026-10-21

esp-radio 1.0.0-beta.1 (heap 64 KiB reclaimed + 36 KiB), embassy-net
0.9.1, sntpc 0.11. `WIFI_SSID`, `WIFI_PASSWORD`, `TZ_OFFSET_MINUTES`
come from the build environment so no secret is in the tree; without
them the clock shows `--:--`. Time is Embassy uptime plus an offset
from each SNTP result (`pool.ntp.org`, hourly, retry after 30 s).
`net.rs` has `wifi_task`, `net_task`, `ntp_task`; `display.rs` draws
`HH:MM`, blinks the colon and publishes only a changed image.

### M2b — buzzer — DONE, verified 2026-10-21; reserved for the alarm

A magnetic buzzer behind an S8550 PNP: a 3.3 V high still leaves 1.7 V
across base-emitter, so only high impedance is off. `buzzer.rs` used
LEDC (low-speed timer 0, channel 0) open-drain, idle duty 100 %
(released), a tone 50 %; the pin goes to LEDC as an `Output` already
high, so there is no low blip. Tone 2.5 kHz. The boot beep and test
melody were removed.

### M3 — colour and brightness — DONE, verified on the panel 2026-10-21

Colour is time: a channel's duty within the COM slot is its level;
brightness is `IS` alone.

- A slot is binary-weighted regions, each starting with a frame to
  both drivers so the latch lands at the same offset. Identical
  neighbours are not merged as in the reference: a frame costs the CPU
  nothing.
- `Panel` keeps a level triple per position; a gamma 2.2 table maps a
  component to a level, and brightness percent maps to a current step
  like the reference's `current_for` (2.5 to 40.4 mA, gamma 2.8),
  capped at `MAX_CURRENT_STEP` (11, 30 mA).
- Scan rate: at a 2400 µs cycle (417 Hz) orange shimmered, its green
  one 80 µs pulse a cycle, seen as a phantom array when the eyes move;
  2.8 kHz still shimmered. Settled: three planes (8 levels a channel),
  plane-major frame order (a digit lit three times a cycle, longest
  dark stretch 12 units, not 21), SPI 20 MHz, unit 5 µs, cycle 140 µs,
  7.1 kHz. Fine from 6 kHz on.
- A transfer carried 22 cycles (the 32 KB DMA limit) and one blanking
  frame, so the restart gap came once a transfer.
- A 644 µs maximum gap was first blamed on Wi-Fi and the scan task
  moved to an `InterruptExecutor` (software interrupt 1, priority 2);
  it was really the 31 KB rebuild between transfers, then the report
  line's serial write. With double buffering and the report moved
  out: average 1 µs, maximum 49 µs over 3222 transfers.

### M4 — buttons and alarm — DONE, verified on the panel 2026-10-21

Top short: next colour; bottom short: next brightness; top long:
alarm-set (top increments, hour +1, minute +5; bottom next field, then
save); bottom long: arm or disarm (degree mark); any press stops
ringing. Palette red, orange, yellow, green, cyan, blue, magenta,
white; brightness 10, 25, 50, 75, 100 %. Alarm-set leaves after 15 s
idle; the alarm rings until a press or for two minutes, once per
minute. `buttons.rs` polls every 10 ms, long at 800 ms while held;
`ui.rs` is the pure state machine. Lessons: `Instant - Duration`
panics when uptime is shorter, so "last happened" is an `Option`; the
top button is GPIO4, not GPIO3.

### M5 — persistence — DONE, verified on the panel 2026-10-21

`settings.rs` owns `Settings` in a `Watch`; `settings::update`
publishes only a real change, and `Ui` takes the settings as a
parameter. The record lives in the 24 KB `nvs` partition at 0x9000
through `sequential-storage`'s map over `esp-storage`, key 0, behind
an async adapter in `storage.rs`; with no such partition settings stay
in RAM. The persist task writes 2 s after the last change, if the
value differs. `esp-storage` runs with interrupts off, so the scan
stalls: a write is under a millisecond, a sector erase tens of
milliseconds, a dark blink once per few hundred changes. Host check
with the mock flash: 2000 saves, 5 page erases. The image grew 33 KB.

### M6 — settings over Wi-Fi — DONE, verified on the panel 2026-10-21

A page and the same values as an API; no authentication at this point.

- Record version 2 added twelve-hour and the timezone (i16 minutes),
  version 3 the hostname (a length byte and 24 bytes); older versions
  load with defaults for what they lack.
- `http.rs`: three identical listener tasks, HTTP/1.0, one request a
  connection. Three because a browser opens more connections than it
  uses, and with one socket each spare was a refused connection for
  the next request (found on the panel); a silent client is dropped
  after 2 s. `picoserve` was rejected as more than four routes
  deserve and a tie to its embassy-net version. Parsing is in the
  pure `form.rs`.
- `GET /`, `POST /` (absent checkboxes mean off, then a redirect),
  `GET /api`, `POST /api` (only the keys given; 400 on a bad one),
  both form-urlencoded so `curl -d` and Home Assistant's
  `rest_command` need no JSON. Keys: `color`, `brightness`, `alarm`,
  `armed`, `twelve_hour`, `tz` (−840 to 840), `hostname`, `melody`.
- Both buttons held show the IP, an octet at a time with its number
  first (`1192`, `2168`, `3  1`, `4 42`), `no1P` before DHCP. No
  mDNS: embassy-net has no responder.
- The hostname goes out as DHCP option 12; a change calls
  `Stack::set_config_v4` only after 3 s stable, since at once it
  dropped the address under the page's own redirect.
- With two APs on one SSID the clock sometimes took the far one:
  esp-radio's default `Fast` scan takes the first beacon.
  `ScanMethod::AllChannels` connects to the strongest.

### M7 — alarm days — DONE, verified on the panel 2026-10-21

`alarm_days`, a 7-bit mask, bit 0 Monday, default all; record version
4, the spare byte at offset 34. Zero is refused: a disarmed alarm that
looks armed. `LocalTime::weekday` is `(days + 3) mod 7`. The degree
mark means armed whatever the day. Key `days`: `all`, `weekdays`,
`weekend` or a list of `mon` to `sun`.

### M8 — alarm melodies — DONE, verified on the panel 2026-10-21

`melody.rs`, pure: a `Note` is a pitch (0 a rest) and a length in
sixteenths, sounding for its length less a gap of an eighth of it,
clamped to 15 to 60 ms. `beeps`, `twinkle`, `ode`, `frere`. The buzzer
resonates near 2 kHz: A6 (1760 Hz) and then Ode's G6 (1568 Hz) rang
far louder than the rest, so all tunes are in G with E6 (1319 Hz) the
top; a per-pitch duty table is the next step if a pitch still stands
out (not done). The buzzer takes a `Signal` of `Ring`, `Preview` or
`Stop`, polled between notes. Setting `melody`, record version 5,
offset 35. The page's "Play" posts to `/play`, which previews once and
saves nothing; the user wanted preview and save separate.

### Wi-Fi reach (after M8) — DONE, verified on the panel 2026-10-21

esp-radio starts at a 5 dBm transmit ceiling, so a far AP heard at
-78 dBm got the clock's reply at its noise floor and
`AuthenticationExpired` followed. `config::WIFI_MAX_TX_POWER_QDBM`
became 34 (8.5 dBm, the user's figure for SuperMini boards, whose
antenna makes more power worse), then 44 (11 dBm) for the case; later
68. The station scans before each connect and logs the candidates;
the default 10 to 20 ms dwell saw nothing from afar, so 120 to 200 ms.

### Missed alarm after a power cut (after M8) — DONE, verified on the panel 2026-10-21

On the first tick with a valid time, an alarm armed for its day and
at most `MISSED_ALARM_GRACE` (10) minutes old rings; the age is modulo
the day. Once per boot (`Ui::had_time`), in clock mode only: someone
at the buttons is awake.

### Skip the next alarm (after M8) — DONE, verified on the panel 2026-10-21

`Settings::skip_next` (key `skip`) rides the `Watch` but is left out
of the record, so a reboot clears it and it costs no flash write: the
persist task compares records, not structs. `Ui::tick` clears it
instead of ringing, only on a minute the alarm would have rung. No
indication on the panel.

### Form posts are all or nothing (after M8) — DONE, verified on the panel 2026-10-21

`POST /` applied the form to the live settings, clearing the
checkboxes first and stopping at the first bad pair: a cleared alarm
time disarmed the alarm behind the usual redirect. Now the body is
applied to a copy and committed with one `update` only if all parsed;
on an error the redirect is `/?error=value` or `/?error=key` and the
page says nothing was saved. `scripts/verify-form-post.sh` checks it.

### Wi-Fi drops out after a while (after M8) — debugging plan, 2026-10-22

The clock lost the network after a while with nothing in the log; the
router's lease table kept the old lease, so no reconnect or reboot had
happened. A spare board was instrumented and run against a host-side
probe log before any fix.

- Traces kept in the normal build: timestamps on every line, the
  reset reason, a minute line with RSSI, BSSID and channel,
  `net::monitor_task` (link and config transitions),
  `net::probe_task` (a gateway ping every 30 s, embassy-net's `icmp`
  feature), per-listener HTTP counters and uptime in the heartbeat.
  Debug flash: `ESP_LOG='info,embassy_net=debug,esp_radio=debug'`.
  Host side: `scripts/watch-clock.sh <ip>`.
- Ping never worked: embassy-net does not enable smoltcp's
  `auto-icmp-echo-reply` by default. Enabled.
- `RX QUEUE FULL` in bursts: esp-radio's `rx_queue_size` is 5 frames
  and LAN broadcasts outrun the executor; a DHCP offer was lost. Now
  `config::WIFI_RX_QUEUE_FRAMES`, 32.
- The first DHCP discover is lost anyway (the user knew it from
  another project): the station reports connected before it can pass
  a frame and smoltcp retries after 10 s. `wifi_task` switches DHCP
  on `LINK_SETTLE` (500 ms) after each connect and off at each
  disconnect; embassy-net does not wake its runner for that, hence
  `net::poke`. The address then came 20 ms after the switch.
- Review finding, unproven on hardware and left in the backlog:
  esp-radio's `WIFI_TX_INFLIGHT` is not decremented when
  `esp_wifi_send_data` returns early on a disconnect and is never
  reset; `can_send()` is `inflight < 3` and gates `rx_token()` too, so
  three leaks deafen the stack until a reset. The patch was not made.
- Self-heal, bench-tested on the spare board 2026-10-22:
  `probe_task` arms on the first gateway answer since boot (a router
  that never answers cannot reboot-loop the clock), then
  `config::PROBE_RESET_FAILURES` (10, five minutes) failures with the
  link up call `heal::reset`. It waits for `scan::stop` first, at the
  user's request: a reset mid-transfer leaves the last latched row lit
  at full current. A panic reset cannot blank. Test: drop the router's
  echo replies (`nft insert rule inet fw4 output ip daddr <ip> icmp
  type echo-reply drop`); the heal came at 325.8 s, the panel stayed
  dark, and a second round counted 2. The first ping after a boot
  fails by design: smoltcp sends an ARP request instead of queueing.
- `carry.rs`: one RTC-RAM record, kept only across a software reset
  (`CoreSw`, `Cpu0Sw`) with its mark; the user asked for that gate. It
  carries the heal count and time, the skip flag and the alarm last
  handled, so a heal neither loses the skip nor rings again, through
  the grace, an alarm dismissed within 15 minutes. A reset while
  ringing records nothing, so the grace brings that alarm back.
- The page and the JSON show `uptime`, `heals`, `last_heal`;
  `form::Status` keeps `form.rs` clockless; `RESPONSE_BUF` is 4096.

### What the capture showed (after M8) — DONE, verified on the panel 2026-10-22

Eleven hours on the bench showed no fault; kicking the station off
the near AP (hostapd's `del_client`) did. ESP-IDF avoids the AP that
just dropped it, so the driver tried only the far AP for a minute and
once held it for 223 s, connected and dead: the symptom. The AP hears
the board 21 dB weaker than the board hears it (-59 against -38 at
11 dBm), so the far AP cannot hold it; the real clock sat 7 dB above
the near AP's noise, the limit of OpenWrt's lowest rate of 6 Mbit/s
(`legacy_rates 0`). A lossy uplink was the cause; the self-heal is
the backstop.

- Transmit ceiling: the driver rounds down to a level, 44 = 11 dBm,
  56 = 14, 68 = 16, 72 to 79 = 18, 80 = 20. The AP read -59, -55, -52
  up to 68; at 72 and 80 the link got worse everywhere, the SuperMini
  antenna's mismatch. 68 is the ceiling, verified by the user at the
  AP. The real clock in its case went from -84/-91 to -74/-76 dBm,
  with 0 heals over 24 h (2026-10-23).
- A 10-byte ping passed on the far AP while pages crawled, so the
  probe is `config::PROBE_PAYLOAD_BYTES`, 1400, with a 2 s timeout.
- Not done: smoltcp's own `log` feature for its DHCP lines, wanted
  before another far-AP session (DHCP took 30 to 135 s there);
  `legacy_rates '1'` on the AP for about 10 dB more uplink margin.
  BSSID pinning and re-selection after three probe failures were set
  aside by the user for the next section.

### Heal when the station cannot connect, and keep the time across a heal (after M8) — DONE, verified on the panel 2026-10-22

The probe heals only a station with an address; one that never
associates was not healed, and only a reset clears the driver's
avoidance. `wifi_task` calls `heal::reset` after
`config::CONNECT_RESET_ATTEMPTS` (10) failed connects in a row or
`CONNECT_RESET_SECS` (180). The user chose this over a test for a
candidate AP: with every AP down the clock resets every three
minutes, a dark blink. The time is carried as `time_offset_us`, Unix
minus the RTC timer, which runs through any reset but power-up
(`clock::carry_time`, `clock::restore_carried`); the minute heartbeat
refreshes it, so a panic reset carries the time too. Verified
2026-10-22 on the spare board: the clock had the time 260 ms into the
boot, 5 s before NTP agreed. The connect budget's own reset was not
forced to fire.

### Review backlog (2026-10-21)

The items still open on 2026-11-17 moved to `PLAN.md`'s review
backlog; the four left here were all closed or decided not bugs.

### Review backlog, second pass (2026-10-23)

A whole-tree review of commit 0e85ee9 against the dependencies'
sources. The items still open on 2026-11-17 moved to `PLAN.md`'s
review backlog; those left here were all closed by 2026-11-07. Decided
not bugs: the grace used up in another mode, a reset inside the 2 s
settle window, no chip-id check on an upload (the bootloader falls
back), the display task frozen during an upload.

### Review backlog, third pass (2026-10-27): more than a clock needs

The items still open on 2026-11-17 moved to `PLAN.md`'s review backlog.

The brief was over-engineering: the panel shows HH:MM and a minute of
drift is acceptable, so anything working below a second, or measuring
a problem since solved, was a candidate. Every file was read; nothing
found was a bug. Left alone as justified: the self-heal and connect
budgets, the persist debounce, the scan stop for a reset or upload,
the upload drain, the `RefCell` in `settings::update`, the buzzer's
per-note timer. Found, all closed:

- **Sub-second time on a minute display** (microsecond clock, the
  `RESET_PAUSE` add-back, two carry words): the user chose to keep it,
  2026-10-29.
- **Colour through 8-bit RGB and two lookup tables**: done 2026-10-29
  after M10; `PALETTE` holds `Levels`, the sweep's tail dims by
  halving levels. 75 % is step 9, not 8.
- **Instrumentation from the dropout campaign** (the listeners'
  `State` and `log_status`, `net::monitor_task`,
  `net::log_candidates`, `scan::GapStats`): kept by the user,
  2026-10-29.
- **Dead state** (`ota::SELECTED`, `carry::SOFT`, `clock::is_synced`,
  unused panel methods): removed 2026-10-29.
- **Nits** (the PIN's constant-time compare, `try_changed` in
  `display_task`, `Hostname::DEFAULT`): done 2026-10-29, except
  `http::Counting`, kept.

### Review backlog, fourth pass (2026-10-30): the code since the second pass

The items still open on 2026-11-17 moved to `PLAN.md`'s review backlog.

A bug review of the whole tree in four slices by four independent
readers, each finding confirmed in the code path by a fifth, against
the locked dependency sources. Everything since the second pass
(`/reload`, the tunes, credentials and token as settings, the access
point with DHCP and DNS, colour in levels, the core split) had only
had the over-engineering pass, and the findings were all there.
Nothing was found in the scan path, the encoder, the carry codec, the
record decode, the auth rules or the upload.

An independent review by another model (`codex-review.patch`,
2026-10-30) agreed on every point and added none: six earlier backlog
items re-confirmed open and three of the bugs below. It ranked the
self-heal lost once the lease expires, the alarm lost in another mode
and the heal reset during a failing upload highest. Its host
reproducers are the regression tests to write: the hidden-SSID
overwrite (`ssid_pick=new&ssid=old&password=…` saves `old`), the token
restored after a clear, the grace consumed by a first time at 07:05
during a PIN display for an armed 07:00 alarm, and a bodiless `POST /`
clearing armed, twelve-hour and skip. Rust 1.99.1's clippy flagged
`chunks_exact_mut` in `scan.rs`; it is `as_chunks_mut::<CYCLE_BYTES>()`
now.

Found, all closed:

- **Re-saving the setup form with the stored network wiped its
  password** (the write-only field posts empty): fixed 2026-10-30,
  `form::settle_password`; verified from a phone.
- **The hidden "other network" field overrode the list's pick**:
  fixed 2026-10-30, the pick wins in `form::apply` and the field is
  disabled; verified from a phone.
- **Timezone and hostname changes on the setup form were lost when
  the network was unchanged** (reset inside the persist settle):
  fixed 2026-10-30, every save on the access point leaves it.
- **Clearing the API token did not survive a reboot on a build with
  `API_TOKEN`** (`seed` refilled it): fixed 2026-10-30, record
  version 7, `token_cleared`.
- **A two-button slap to silence the alarm changed a setting**: fixed
  2026-11-02, a one-second quiet window in `Ui` ("Quiet after the
  press that ends a ring").
- **The raw record, password and token included, went into the serial
  log** (`Store::load`): fixed 2026-11-07, version and length only.
- **Disarming from the page or `/api` does not stop a ring**: decided
  2026-11-07 not a bug; MQTT has `silence`.
- **Two `/scan` posts at once thrashed the executor** (two waiters on
  one embassy-sync `Signal` wake each other): fixed 2026-11-07, a scan
  count each waiter polls.
- **A bad token body burned the PIN**: fixed 2026-10-30, the body is
  parsed first.

## 6. Risks and how they are handled

| risk | mitigation |
| ---- | ---------- |
| quad nibble bit ↔ SIO line order | confirmed on hardware (M1) |
| quad lines float between transfers | pull-downs on all four pins; frames end all-low |
| DATA/CLK simultaneous edge read as latch | 3 nibbles per bit, data never changes when CLK falls |
| last COM slot lit across the restart gap | cycle ends with a blanking frame |
| transfer too long for one DMA write | limit is 32 KB on the C3 |
| esp-rtos / esp-hal version mismatch | esp-generate's pinned set |
| thermal: too many sinks at high IS | IS from brightness, never all-white for long |

## 7. Roadmap (2026-10-22)

Planned order: over-the-air updates, Wi-Fi provisioning from an access
point, (added 2026-10-29) a `hosttest/` crate so the scratch-crate
checks become `cargo test`, then the board's two sensors. Hardware
facts are from `3rdparty/HU-058_ESPHome/docs/hardware.md` and the
adapter's schematic, `adapter/adapter/adapter.kicad_sch` (an
ESP32-C3-Zero on a 2×8 header in the 8051 socket).

### M9 — OTA updates — DONE, verified on the panel 2026-10-22

The clock lives in its case with no USB reachable.

**Flash layout.** `partitions.csv` replaced espflash's default table
(one `factory` app), passed from the runner line in
`.cargo/config.toml`: `nvs` 0x9000 (0x6000, offset kept so the
settings survived), `phy_init` 0xF000, `otadata` 0x10000 (0x2000),
`ota_0` 0x20000 and `ota_1` 0x200000, 0x1E0000 each. The runner
passes `--erase-parts otadata` so a USB flash boots `ota_0`, the image
just flashed. espflash's prebuilt bootloader does the slot switch,
though its docs say it "might not".

**Upload.** `POST /ota` with the raw image (`espflash save-image`) as
the body and `Authorization: Bearer <token>`; the token is build-time
(`OTA_TOKEN`, renamed in M10), at least 8 characters, compared in
constant time, and a refusal comes before the body is read. The body
is streamed (`Body::Stream`) into `ota::Update`, which erases, writes
and reads back one 4 KB sector at a time; `OtaUpdater` picks the slot
and switches `otadata`. Checked before any erase: `Content-Length`
(a sector to a slot), the image magic 0xE9, the descriptor magic
0xABCD5432 at 0x20, the hash-appended flag (byte 23). The stream is
hashed (`sha2`) against the trailing SHA-256, the only thing that
catches a truncated file sent with its own length. Any failure leaves
`otadata` alone. One upload at a time; a second gets `503`.

**One flash handle.** A second `FlashStorage` could not read the
partition table: `FlashStorage::new` sizes the flash by a chip-ID
command, which returns 0 once the first handle has used the flash.
Hence `storage::with_flash`.

**The scan.** Flash writes run with interrupts off, so the upload
stops the scan (`scan::stop`) once the first sector passes; the panel
is dark until the reboot. A failure calls `scan::resume` (new), and
`stop` became idempotent so a self-heal during an upload cannot hang.
`http::RX_BUF` went from 1 KB to 4 KB, being the receive window.

**Rollback.** The bootloader falls back only on a bad checksum, so
the carry record has `ota_boots`: set to 1 by the upload, bumped by
`ota::check_rollback` at each boot, and at `config::OTA_BOOT_LIMIT`
(3) the previous slot is activated. `ota::confirm` clears it at the
first gateway probe answered or HTTP request served. A power cut
zeroes the record, deliberately.

**Visible.** `FIRMWARE_VERSION` from `git describe` (31 characters)
in the app descriptor; read-only `version` and `slot` on `/api`.
`scripts/ota.sh <ip> [app.bin]`.

**Bench, 2026-10-22.** A 672544-byte image in 10.4 s, about 65 KB/s
with flash writes. Bad token `403`; random bytes `400 not an
application image`; a truncated file or a flipped byte `400 image
hash mismatch`, the other slot written but not selected; a body
shorter than claimed ended in the 5 s socket timeout and a reset by
peer, the running image kept. An image that panics at 1 s was rolled
back on the third boot. Image 652 KB, 33 % of a slot; main stack
98 KB. The host checks caught `fill` dropping the request-buffer
prefix past the first sector. The user confirmed it on the panel.

### M9b — upload from the page — DONE, verified on the panel 2026-10-22

`GET /update` serves a page whose script posts the file as the body of
`POST /ota` by `XMLHttpRequest` (it reports upload progress; a plain
form would need a multipart parser on the clock). On `200` it waits
5 s, polls `/api` every 2 s up to 20 times and shows the version and
slot. `scripts/ota.sh` with no argument only builds the image. Checked
under node with a mocked DOM and from headless Chromium over the
DevTools protocol (`DOM.setFileInputFiles`): up in 10 s, back at 18 s.

### M9c — a PIN on the panel for the update page — DONE, verified on the bench 2026-10-22 (panel not watched)

An owner does not build firmware, and a token in a published image is
public; what every owner has is the panel. `GET /update` issues four
digits from the hardware RNG, shown on the panel for `ui::PIN_SHOW`
(60 s) or until a press, and logged. The upload sends `Authorization:
PIN 1234`. One attempt uses the PIN up; a wrong one starts
`PIN_LOCKOUT` (10 s); `PIN_TTL` is 5 min. The PIN is never in the
page, and a cross-site page cannot send the header without a preflight
the clock does not answer. `PinState` is pure. The script keeps the
token. Bench: each refusal as designed, 683 KB in 11.6 s.

**A refusal drains the body first** (found by the user in a real
browser). A `403` and close while 680 KB was still coming reset the
connection before the browser read the answer. `http::upload` now
discards the rest of the body (capped at 2 MB and 40 s) before any
refusal but `503`; at about 200 KB/s a wrong PIN costs 2 to 4 s.

### Brightness ceiling (after M9c) — DONE, verified on the bench 2026-10-22

Quick presses on the brightness button reset the board with no panic
logged. Capping the current step at 5 made it stable; steps 8 and 15
reset it, and only since the transmit ceiling went to 16 dBm: the
supply is the limit (10 % to 100 % is about 2.5 to 40 mA on every lit
sink at once). `MAX_CURRENT_STEP` is a build variable, default 5,
bright enough for the user. `settings::BRIGHTNESS_STEPS` is the
choices within the ceiling (three by default); `from_record` moves a
step the build does not offer to the brightest. Not done, for a higher
ceiling: slew the step over a few frames, and log the reset reason so
a `SysBrownOut` is told from a panic.

### Hold the self-heal while the alarm rings, and ring for five minutes (after the brightness ceiling) — built, not yet verified on the panel

A self-heal could reset the chip mid-ring, and the grace then rang the
alarm again; the user also found 120 s too short. `display.rs` owns a
`RINGING` atomic, cleared after the alarm is recorded as handled.
`heal::reset` waits while it is set, at most `ui::RING_TIMEOUT` plus
30 s, then resets as before. The panic path cannot wait.
`RING_TIMEOUT` is 300 s; `MISSED_ALARM_GRACE` stays 10 minutes.

Owed on the panel: pull the uplink while the alarm rings; the log must
show `heal: ... alarm ringing, holding the reset`, the ring must go on
until a press or five minutes, and no second ring after the reset.
Also: an undismissed alarm stops after five minutes.

### Carry the time across a reset as plain Unix time (after the ring hold) — verified on the bench 2026-10-28 at short uptime; at 4.5 h of uptime in the soak of 2026-11-08

The user saw the clock jump back nine minutes after a self-heal. The
carry was Unix time minus `Rtc::time_since_power_up`, and esp-hal
converts RTC ticks with a slow-clock period it calibrates afresh at
every boot, so the whole count since power-up was re-read with a new
period: 0.3 % of two days is nine minutes, either sign.

The user chose the simpler fix: carry plain Unix microseconds
(`Carried.unix_us`) rather than raw ticks (exact, but three PAC
reads). `clock::restore_carried` adds `clock::RESET_PAUSE` (200 ms),
which every deliberate reset waits. The mark became `"CAR"` plus a
layout version (`0x4341_5202`), so a record of another layout is
discarded; an OTA across a layout change is not covered by the
rollback. The display loop calls `clock::carry_time` every iteration,
so a panic reset is at most a second stale. Verified by `/reload`
below; on 2026-11-08 an OTA at 4h31m of uptime carried the time to the
second.

### Three simplifications from the third review pass (after the carry) — DONE, verified on the bench 2026-10-28 (panel and buttons not watched)

Chosen by the user.

- `http::parse_head` returns a `Head<'a>` of borrows instead of byte
  spans; `read_request` parses twice, before the body read and after
  it on the head half of `buf.split_at`.
- `Ui` has one timer, `since`, set by `Ui::enter`, in place of four.
- `Ui::last_alarm` is the Unix time the alarm last rang or was
  skipped; an occurrence is handled when that is at or after the
  moment the alarm last came round. It replaced `carry.handled`, its
  window and `Ui::fired_minute`. It is carried as `None` while
  ringing, so a reset mid-ring rings again through the grace. Carry
  layout 3, eight words.

Bench: a skipped alarm did not ring after an OTA thirty seconds later
(`last_alarm_unix: Some(…)`); an upload during a ring rang again
352 ms after boot. Not driven: the IP, alarm-set and PIN timeouts.

### `POST /reload`: a deliberate reset on demand (after the third-pass items) — DONE, verified on the bench 2026-10-28

The user asked for a way to exercise the carry record without a heal
or an OTA. `POST /reload` takes the `Bearer` token only, answers, and
after the flush calls `heal::reboot`, now the common tail of
`heal::reset` and `ota::reboot` (`scan::stop`, `carry_time`, the
pause, the reset). It has no ringing hold and no heal count.
`http::respond` returns `Then::{Close, NewImage, Reload}`.
`scripts/reload.sh <ip>` wraps it and polls `/api` until the clock is
back, about 3 s.

Bench: the time came back within a second (request 13:50:04.687,
record 13:50:05.002) and the skip flag carried. A colour change two
seconds before a reload was lost, the backlog's "reset inside the
settle window". The rollback count cannot be tested this way: serving
the request confirms the image.

### More tunes, and a random pick (after `/reload`) — random verified on the bench 2026-10-28; the tunes retuned over three listens that day, Reveille heard 2026-10-30

Five more public-domain tunes in `melody.rs` after `frere` (the table
order is the stored index): `morning` (Grieg's Morning Mood),
`westminster`, `reveille`, `elise`, `jingle`. Random is a value of
`melody`: `melody::RANDOM`, `random` on the API, stored as `0xFF` so
the record version stayed 5. The buzzer task resolves it at each ring
or preview; `melody::pick` never draws the beeps nor the last pick.
Bench: six random previews drew six tunes.

**By ear (the user, 2026-10-28).** The notes that rang out were the
new pitches below G5: a square wave's third harmonic lands at 1.6 to
2.1 kHz for C5 to F5, on the buzzer's resonance. Westminster and
Reveille were moved up to C (lowest note G5), which cured them; Für
Elise and Frère Jacques cannot move, so `melody::duty_pct` gives 33 %
below 700 Hz, a duty with no third harmonic. Jingle Bells went to
160 bpm, Für Elise to 75. Morning Mood was rewritten from the user's
score (`morning-mood-notes.png`) in G at 75 bpm. Its D6 (1175 Hz)
stays weak: a temporary `/play?duty=NN` hook showed 50 % cleanest, so
it is the fundamental on the resonance's slope and firmware cannot
help. Tape over the buzzer's port made it louder; tape with a pricked
hole (a Helmholtz resonator tuned low) is what the user kept, and with
it 33 % against 50 % made little difference. Reveille was rewritten
from the user's score (`reveille.png`; quarters at 160 bpm, no dotted
figures) and heard better on 2026-10-30.

### Fewer `unsafe` blocks (after the tunes) — DONE, verified on the panel 2026-10-29

At the user's request; of seven, two remain.

- `ScanHardware`'s `Send` impl covered `SpiDma<'static, Async>`,
  `!Send` on purpose (an async driver installs its interrupt handler
  where it is created). `scan_task` now builds the bus itself from the
  `SPI2` and `DMA_CH0` singletons and four `Flex` pins.
  `TransferBuf`'s impl was redundant.
- The RTC record was a `static mut`; `#[ram(persistent)]` takes any
  `Persistable` static, so `carry::RECORD` is
  `[portable_atomic::AtomicU32; WORDS]` at the same address.
- The pull-downs went through the PAC because the SPI builder resets
  a bare pin's pull. A `Flex` goes through the frozen-signal path,
  which the builder leaves alone, so `display_wire` in `main.rs` sets
  `Pull::Down` on it. The `esp32c3` dependency went.
- The two `#[unsafe(no_mangle)]` stay: esp-backtrace and esp-println
  resolve those symbols.

Bench: the new image read the old one's record; the restart gap was
unchanged; the user saw the panel dark with nothing faintly lit.

### M10 — Wi-Fi provisioning from an access point — DONE, verified on the bench and with a phone 2026-10-29

Credentials became a stored setting, and a clock with none raises its
own network. As an access point it has no time and no alarm; the panel
says `AP`. `WIFI_SSID`/`WIFI_PASSWORD` stay as the seed.

**From the sources** (esp-radio 1.0.0-beta.1, smoltcp 0.13.1): the AP
link is up when `set_config` returns; one receive callback serves both
interfaces, so one mode per boot. A connect failure's `reason` tells
`NoAccessPointFound` from a wrong password
(`FourWayHandshakeTimeout`, `HandshakeTimeout`). smoltcp sends to
`255.255.255.255` with no ARP, so a DHCP server fits on a `UdpSocket`;
embassy-net has none.

**The carry record starts over at a mode switch**: its heal count,
time, skip flag and last alarm belong to the mode that wrote them.
`heal::switch_mode(boot_ap)` writes a default record keeping
`ota_boots`, with `boot_ap` (word 8, layout 4, nine words), and no
time. A panic keeps the record, so an access point comes back as one.
The display tick wrote the time back during the 200 ms pause, hence
`carry::freeze`.

**Step 0: a record from a newer image survives a rollback.**
`from_record` reads the prefix it knows of any newer version and
clamps `color` and `melody` (`0xff` stays random). Flashed first, so a
rollback from the version 6 record would keep the settings.

**Step 1: credentials as a setting, and the switch reset.**

- `settings.rs`: `Text<N>`, `Secret<N>` (`Debug` prints `<set>`),
  `Credentials` with `NONE`; SSID 1 to 32 bytes, password empty or 8
  to 64. The rules are `const`, so a bad build constant is a build
  error.
- The token is a setting too (the user's decision: a build without
  one must be able to get one, and the PIN proves sight of the panel):
  empty or 8 to 32 printable ASCII, no spaces; never in a page, the
  JSON or `Debug`. `POST /token` takes it with a PIN or the current
  token; empty clears it.
- Record version 6 appends SSID, password and token, `RECORD_LEN`
  167; `storage::Scratch` is 256 bytes.
- `POST /pin` (the user's decision, for the bench) issues a PIN,
  never in the response; `scripts/pin.sh <ip> <serial-log>` reads it
  from the monitor's log.
- `settings::seed` fills the network and token from the build where
  the record has none; afterwards the record wins.
- `OTA_TOKEN` became `API_TOKEN` (the user's decision: writing
  credentials needs the same proof as an upload); the token and PIN
  code moved to `src/auth.rs`, hardware-free.
- `form.rs`: `ssid` and `password` are `FormError::Forbidden` (`403
  token required`) without the token, on the key alone: refusing only
  a changed value would be a password oracle. `JsonStr` and `HtmlStr`
  escape the SSID.
- `storage::persist_task` resets into the station after saving
  credentials that differ from this boot's; the 2 s settle lets the
  response finish.

Main stack 98.2 to 96.4 KB. Bench: a wrong password reset the clock
into a station logging `wrong password?` every 15 s, never an access
point; recovery by USB with `--erase-parts otadata,nvs`.

**Step 2: the access point.** `main.rs` chooses it if
`carry::get().boot_ap` or no network is stored. SSID `clock-xxxx`
(last two MAC bytes), WPA2 with `config::AP_PASSWORD` (default
`esp32clock`), channel 1, static 192.168.4.1/24. `src/dhcpd.rs`: pure
`parse`, `build` and `Leases` (192.168.4.2 to .9; an offer holds 60 s,
a lease 3600 s), replies broadcast and padded to 300 bytes. `GET /`
serves `form::write_provision_page` and `/api` takes the network keys
without the token there. The display task draws `AP` in place of the
sweep, with no new `Ui` mode. `Kind::Hold` (both buttons,
`HOLD_PRESS` 10 s) toggles station and access point, back only if a
network is stored; a station that cannot join never becomes an access
point.

**Step 2b** (the user's decision): `POST /ap` with the token resets
into the access point. The setup page lists networks: the driver
refuses a scan in AP-only mode, so the AP is
`Config::AccessPointStation` with a station half never connected, and
it stays up through a scan (bench). Up to `NETWORKS_MAX` (12), one per
SSID, strongest first; `<select name=ssid_pick>`, a typed `ssid` for
"Other", `POST /scan` to rescan. An empty `ssid` is ignored. The page
needs `[hidden]{display:none}`: `label{display:block}` outranks the
browser's rule. With a phone: wrong password, the hold, then the right
one; on the network in 2.9 s, synced at 5.8 s.

`Tracker::new` takes whether the button is down at boot, so the
release of the hold after the reset emits nothing; not verified,
nothing wrong was seen.

**Step 3: captive DNS.** `src/dnsd.rs` answers every A query with
192.168.4.1 (TTL 60 s) and other types with an empty NOERROR, and on
the access point a path that is no route answers `302` to
`http://192.168.4.1/`. The user's Android phone opened its sign-in
sheet on the setup page 5 s after joining.

The user confirmed the whole flow on the real clock, 2026-10-29.
`heap free` on the access point was 49820 bytes, flat for five
minutes, against 54084 as a station: about 4.3 KB.

### Colour in levels, dead state and nits from the third review pass (after M10) — done, verified on the panel 2026-10-29 (the PIN not re-checked)

Items 2, 4 and 5 of the third-pass backlog; the user dropped items 1
(sub-second time) and 3 (dropout instrumentation).

- **Colour in duty levels.** `settings::PALETTE` is `panel::Levels`
  in the panel's blue-green-red order, exactly what the old `GAMMA`
  made of the old `Rgb` values (255 → 7, 96 → 1); `Rgb` and `GAMMA`
  went. `BRIGHTNESS_CHOICES` is `(percent, step)` pairs read off the
  old table: `(10, 1)`, `(25, 2)`, `(50, 5)`, `(75, 9)`, `(100, 15)`.
  The display no longer clamps to `MAX_CURRENT_STEP`: every offered
  choice is within the ceiling by construction.
- **The sweep.** The old dimmed tail was never lit (a quarter of RGB
  is 0 after gamma) and blinked the armed mark. A half-level tail was
  built and seen; the user asked for a single dash instead, so
  `draw_sweep` lights one dash per step and nothing recolours a block.
- **Dead state deleted** (`ota::SELECTED`, `carry::SOFT`,
  `clock::is_synced`, `Panel::set_time` and others);
  `Panel::lamp_test` stays as a hardware aid.
- **Nits.** `PinState::check` compares the PIN as a number: a
  one-attempt PIN with a lockout gains nothing from a constant-time
  compare (`bytes_eq` stays for the token; `+123` passes for `0123`,
  another spelling of the same secret). `Hostname::new` is a `const
  fn`, so `Hostname::DEFAULT` is checked at compile time.
  `http::Counting` was kept: removing it only moves it into `ota.rs`.

### Host tests: a library crate the host can `cargo test` — done, verified on the panel 2026-10-29

Every milestone since M5 had been checked in a scratch crate outside
the repo, recreated each session, so the checks drifted from the code.
The firmware crate cannot host `cargo test`: it depends on esp-hal,
and `.cargo/config.toml` sets the riscv target for every subcommand.
Cargo has no build-only default target (rust-lang/cargo#6784, closed
"not planned"); `--target host-tuple` is stable since Rust 1.91. The
practice of Ferrous Systems, Veecle and Embassy's CI is a HAL-free
library crate, `#![cfg_attr(not(test), no_std)]`, tests beside the
code. A draft `hosttest/` crate pulling files in by `#[path]` was
dropped as the hack.

Built: a two-member workspace, the root the firmware and `core/` the
library it re-exports from `lib.rs`, so `crate::settings` still
resolves; `cargo host-test` is the alias. Moved whole: `aip33628`,
`panel`, `melody`, `config` (hence `core/build.rs` for the
`rerun-if-env-changed` lines), `settings`, `form`, `auth`, `ui`.
Split, the pure half into `core`: `buttons`, `clock`, `carry`, `http`,
`ota`. Not moved: `dhcpd.rs` and `dnsd.rs` (embassy-net's address
types), `storage.rs`, `scan.rs` (`build_cycle`). The host gets its
time driver and critical section from the `std` features of
`embassy-time` and `critical-section` as dev-dependencies. The page
tests assert the bound (`PAGE_MAX`) and print the sizes, so a label
edit is not a failing test. The crate boundary cost 828 bytes of text
under fat LTO. The tests found the next section's bug.

### Setup page within the buffer on hostile SSIDs (after the host tests) — done on the host 2026-10-29

`write_provision_page` escapes each scanned SSID twice; an SSID of 32
`"` is 417 bytes an option, so five such networks in range made the
setup page a `500`, which any phone hotspot could do. Fix:
`form::NETWORK_LIST_BUDGET` (1900 bytes). Each option is measured
through a counting `Write` and written only if it fits; the first that
does not ends the list (sorted strongest first), and the page says "N
more not shown; type the name." A stored network that was dropped
lands in the typed field with "Other" selected. Tests lock the budget
against `PAGE_MAX` with everything else at its worst. Never tried with
a real hostile hotspot.

### Setup form and cleared token, from the fourth pass (after the setup page budget) — done, verified on the bench 2026-10-30

Five fixes, each a pure rule in `core/` under test.

1. **An empty password keeps the stored one**
   (`form::settle_password`, after `apply`): the stored SSID with an
   empty password keeps the password unless the scan lists the network
   as open; another SSID listed as secured with no password is
   `BadValue`; anything else is as posted. `/api` is untouched.
2. **The list's pick wins over the typed name.** The script disables
   the typed input when a network is picked, and `form::apply` writes
   a non-empty `ssid_pick` over `ssid` after the loop, whatever the
   key order.
3. **Every save on the access point leaves it.** `persist_task`
   resets after a save: on the station if the credentials changed, on
   the access point always when a network is stored. The handler's own
   reset (`Then::Join`) remains only for a form equal to the current
   settings. So a hostname posted with the stored network reaches
   flash first, and `/api tz=` on the access point joins the network
   too. A failed save still leaves the clock on the access point.
4. **A cleared token stays cleared.** A build with `API_TOKEN`
   re-seeded a token cleared on purpose. Record version 7 appends a
   `flags` byte (`RECORD_LEN` 168), bit 0 `TOKEN_CLEARED`, the other
   bits ignored on read; `seed` fills the token only when empty and
   not cleared. Not a key, not in the JSON; the boot line says `api
   token: cleared`. An older image rolled back re-seeds, as before.
5. **The token body is parsed before the PIN is spent**
   (`form::parse_token` first in the `/token` handler).

Bench, 2026-10-30, the bench board over the air: the version-6 record
read intact; all five behaved as designed, the setup form from a phone
with the serial log read. The `api token: cleared` boot line itself
was not captured (the monitor loses the first two seconds after its
reset); `token_set: false` after the reset was.

### M11 — NTC thermometer and light sensor — readings built 2026-11-01 (M11a), the face in M11b, the dimming moved to `PLAN.md`

The adapter carries the pull-ups (10 kΩ to 3V3): NTC R4 on J1 pin 3 to
GPIO0 (ADC1 ch 0), LDR R3 on J1 pin 4 to GPIO1 (ADC1 ch 1). The clock
board's own R1 and R2 pull up to +5 V and are left off: the C3's ADC
reads reliably to about 2.5 V at 11 dB and the pin's maximum is 3.6 V.
From 3.3 V a 10 kΩ NTC gives 1.65 V at 25 °C, 2.5 V near 0 °C, 1.1 V
at 40 °C. Only ADC1 (GPIO0–4) works with Wi-Fi on; GPIO3 and 4 are the
buttons and GPIO2 is a strapping pin. The NTC's part number is not in
the reference; 10 kΩ, B = 3950 was assumed.

The first design had a `temp_show` period and a separate `temp_unit`;
what was built is in the subsections. Dimming by the light was not
built: its design moved to `PLAN.md` ("Dim in the dark") on
2026-11-17.

#### M11a — read both sensors and show them on the page and in `/api` — built and verified on the bench 2026-11-01

The user asked for the readings first; no setting, no record change.

**Correction to the design.** A missing LDR leaves the pin at the
pull-up, as a pitch-dark one does, so only the NTC can say "no
sensor". The raw code pins near 3.1 V, so the NTC is imprecise below
0 °C and unreadable below −20 °C.

**`core/src/sensors.rs`.** `temperature_dc(mv)` interpolates a
21-entry table (−20 to 80 °C, from the B equation) to tenths, within
0.17 °C of the equation; colder is `None`, hotter `Some(800)`.
`light_pct(mv)` takes the logarithm of the LDR's resistance between
`LIGHT_FULL_OHMS` (1 kΩ) and `LIGHT_DARK_OHMS` (50 kΩ); a first cut
linear in the voltage squeezed the bench's lighting into 5–21 %.

**`src/sensors.rs`.** One thread-mode `sensor_task` owns ADC1 and both
pins (`AdcCalCurve`): 8 samples of each every second, 50 µs apart,
into an exponential average of weight 1/8 (about 10 s) seeded by the
first reading, published in two atomics. The JSON has `temp_c`,
`temp_f` and `light` (or `null`); the page a `Room …` line; the
heartbeat the millivolts.

**Bench.** The NTC read 1657 mV, 24.8 °C, the table's 25 °C point. The
LDR read the rail (2894 mV) only because the bench was dark for it,
then 1959 to 2372 mV. A stale version string came from cargo reusing a
build-script output; `touch build.rs` cures it.

**The case warms the NTC.** Upright with the lid open the clock read a
constant 0.7 to 0.8 °C above a thermometer beside it (lying on its
face it reached 25.9 °C in a room of about 21); the pull-ups measured 9.96 and 9.99 kΩ, so
the conversion is sound and the gap is the warm air at the hole. With
the case closed it was about −2.0 that day. Noise is ±0.25 °C on the
10 s average. Owed: the closed-case offset on the installed clock, and
a second day's readings before trusting the tenths.

**Wi-Fi on the adapter builds.** The first adapter-built clock (C3
Zero, chip antenna next to the clock board's ground plane) heard the
near access point 10 to 25 dB weaker than the old bench board (−52 to
−65 against −38 to −42 dBm). A U.FL receptacle at the chip-antenna
feed with a patch antenna, fitted by the user on 2026-11-01, gave −29;
that board is `martin-clock`, read by both access points at about
−70 dBm from its real spot. Its first connect after the rework went to
the far access point although the near one scanned stronger; not seen
again (a repeat would warrant `with_bssid`).

Compared on 2026-11-12 in the same case and spot, lid open, three
minutes connected with 498-byte pings (none or one of 180 lost on any
board):

| board | near AP, connected | far AP, boot scans | boot probe |
| --- | --- | --- | --- |
| SuperMini Plus adapter, external antenna | −21 to −23 | −62 to −74 | 10 ms |
| old white-clock board | −38/−39 and −52 to −54 | −82 to −86 | 57 ms |
| v3 Zero adapter (edge notch) | −54 to −57 | −86, −87 | 70 ms |

Upright, the two chip-antenna boards were within 3 to 5 dB (the old
board's −38 level was the case lying down); the notch only made the
far access point audible, too weakly to use. The user's decision: the
SuperMini Plus adapter with the external antenna is the one to build.
The SuperMini Plus was not measured upright. Owed: the access points'
figures for each clock from the real spot with the lid closed, which
the user reads from the router; the U.FL rework on the second adapter
board.

#### Thermometer offset (after M11a) — done, verified on the bench 2026-11-01

`temp_offset`, tenths of a degree Celsius added in `sensors::room`
before both units, −10.0 to 10.0, default 0; a decimal with at most
one place on the form and `/api` (`form::parse_tenths`).
`RECORD_VERSION` 8 appends two bytes (`RECORD_LEN` 170). On blue-clock
`-0.7` moved 23.5 to 22.8 °C, `10.1` was a 400, and the value survived
a reload.

#### M11b, first half — the temperature on the panel each minute — done, verified on the panel 2026-11-01

The user asked for the temperature for 5 s in the middle of each
minute, with an off switch: `temp_show` is `off`, `c` or `f` (default
`c`), `RECORD_VERSION` 9, one byte (`RECORD_LEN` 171). The face is not
a mode: in `Mode::Clock`, `Ui::view` answers `View::Temp` for seconds
30 to 34 (`TEMP_SHOW_AT`, `TEMP_SHOW_SECS`) when given a reading, so
presses keep their meaning and ringing, alarm-set, the IP and the PIN
win. `sensors::face` lays out whole degrees, rounded, right aligned in
three positions, the unit letter in the fourth, clamped to −99 and
999; the degree mark is lit for it whatever the alarm.

#### Short frames on the uplink (after the temperature face) — done, verified on the bench 2026-11-01

A marginal uplink passes short frames long after full ones fail.
`ESP_RADIO_CONFIG_WIFI_MTU` is 540 in `.cargo/config.toml`'s `[env]`
(default 1492): a 486-byte TCP payload and a largest ICMP payload of
498. `config::WIFI_MTU` reads the same variable and
`PROBE_PAYLOAD_BYTES` is `WIFI_MTU - 42`, so the probe is the largest
frame the clock sends. Kept after the antenna rework, for a clock
built without it. It limits transmission only (smoltcp 0.14 applies
the MTU in `dispatch_ip` and the segment size, never on ingress). A
ping over 498 bytes gets no reply, since embassy-net has no IPv4
fragmentation. An OTA arrives in 486-byte segments and is no slower
(714 KB in 7.4 s; the flash write is the bound). The evening's
failed-probe count was never recorded.

#### Settings page in three groups (after the short frames) — done, verified in a browser 2026-11-01

Three `<fieldset>`s in the user's order: Alarm, Display, Other. Play
sits under the melody select and posts `melody=<picked>` to `/play` by
`fetch` without following the redirect, so unsaved fields survive;
without script `formaction=/play` works with a reload. The fieldset's
style is written by `write_page` alone, not the shared `HEAD`, where
it put the setup page 15 bytes over its budget. Field names and `/api`
are unchanged.

#### Yellow was pale green (after the settings groups) — done, verified on the panel 2026-11-01

The user found yellow, `(0, 7, 7)`, a very light green: the green die
gives far more light per unit of current than the red, and keeps its
efficiency at low current while the red loses it. `(0, 4, 7)` was
yellow at 50 % but greenish at 10 %; a ramp of the green level by
brightness was greenish at 25 and 50 %. The user chose level 2 flat:
yellow is `(0, 2, 7)` at every brightness, locked by the palette test.
White is presumably greenish too and was left alone. The old mix, put
in for comparison as `lime` right after yellow, the user kept as a
ninth colour; the insertion shifted the stored index of green and
later colours by one (a clock saved on green came up lime).

#### Wi-Fi status line on the page (after yellow) — done, verified on the bench 2026-11-01

Asked for by the user. The controller belongs to `wifi_task`, so the
task publishes the reading in an atomic (`net::RSSI`, a sentinel for
"not connected", read by `net::rssi()`), refreshed every `RSSI_PERIOD`
(10 s). It is the last beacon's figure, stale until a disconnect
lands. The page shows the stored SSID and the dBm, or "not connected"
or "no network stored"; `/api` has the read-only `rssi` (`null` when
not connected and on the access point). The page test's worst case now
stores a hostile SSID.

#### Unsaved-changes mark on the settings page (after the Wi-Fi line) — done, verified on the bench 2026-11-02 with headless Chromium

Asked for by the user. The page's one script sets `f.oninput` to
compare every named control with the defaults the page was served
with, and shows `<b id=u>Changes not saved yet.</b>` while any
differs: "modified", not "touched". It relies on every `<select>`
writing `selected` on its stored option. No warning on leaving the
page: a mark was asked for, not a gate.

The page had 69 bytes to spare and the mark costs 308, so
`RESPONSE_BUF` grew; the user chose 6 KB (`PAGE_MAX` 6016) over the
minimum. Three listeners hold one each: `.stack` went from 90664 to
84520 bytes.

Verified from the session with `chromium --headless=new
--remote-debugging-port` and a Node script over the DevTools protocol,
against test-clock. The default headless window is 493 px high and
clicks below it land nowhere: set it to 1400
(`Emulation.setDeviceMetricsOverride`). The stored melody changed once
during the first run with nothing in the browser's requests to explain
it and no reproduction; most likely another browser at the bench.

#### Favicon and the hostname in the title (after the unsaved-changes mark) — done, verified in a browser 2026-11-02

`src/favicon.ico` (337 bytes, two PNG entries, the panel as pixel art)
is embedded with `include_bytes!` and served at `GET /favicon.ico`
with `Cache-Control: max-age=86400`, on the access point ahead of the
captive-portal redirect; no `<link>` tag, so the pages did not grow.
At the user's request the title and `<h1>` of the settings and update
pages are `Clock: <hostname>`; the setup pages keep `Clock setup`.

#### Quiet after the press that ends a ring (after the favicon) — done, verified on the panel 2026-11-02

A two-button slap to silence the alarm changed a setting: the first
event ended `Ringing` and the second landed in `Clock`. The second can
trail by up to 800 ms, when one button is let go early and the other,
still held, sends its `Long`. Fix in `core/src/ui.rs`:
`Ui::quiet_until`; the press that ends `Ringing`, `ShowIp` or
`ShowPin` sets it to `now + PRESS_QUIET` (one second) and `handle`
drops events before it. A timeout never starts the window. The user
checked it with and without a lingering hand.

#### Two pages for the two PIN actions (after the press quiet) — done, verified on the panel and in a browser 2026-11-02

The update page held both the upload and the token form under one PIN.
The user asked for two pages, each issuing its own PIN, the token page
reachable by its address only: `/update` is the upload alone, the new
`GET /token` the token form, and nothing links to `/token` (test
`nothing_links_to_the_token_page`). Both use `auth::issue_pin`, so
opening either replaces an unused PIN; `POST /token` and the `Bearer`
path are unchanged. An empty token still clears, after the browser's
`confirm()`. The shared script helpers are `form::PIN_JS`.

Found by the user: a spent PIN stayed on the panel. `auth::SHOW_PIN`
now carries an `Option<u16>` and `token_or_pin` signals `None` after
any attempt that spent the PIN; `ui::hide_pin` returns to the clock
without the quiet window. Both pages' buttons stay disabled after any
answer, and an error ends "Reload this page for a new PIN."

### M12 — Home Assistant over MQTT — done, verified in Home Assistant and on the bench 2026-11-07

**Goal.** One device in Home Assistant, announced by MQTT discovery
with nothing typed there: the settings as entities, the readings as
sensors, three buttons (play, silence, reload).

**Discovery** (Home Assistant's MQTT page, read 2026-11-07). One
payload per device, the form recommended since 2024.11, on
`homeassistant/device/<id>/config`, not retained: the clock subscribes
to the birth message, `homeassistant/status` = `online`, and resends
after 0 to 2 s. As built, nineteen entities, 4174 bytes at the worst
hostname of `mqtt::DISCOVERY_MAX` 4608. A `light` entity was rejected:
a light has an off state the panel lacks, and the palette is named
duty triples, not RGB.

**Client: `rust-mqtt` 0.6.0** (the user's pick), `no_std`, on the
`embedded-io-async` and `heapless` versions already in use, features
`v5`, `bump`, `log`, `log-level-warn`, no `alloc`. It does the
protocol only, no reconnect or keepalive loop, which suits a task that
owns its loop. Received data lands in a `BumpBuffer` over a 1 KB
static, reset after each event (the module's one `unsafe` call).
`poll_header` is cancel-safe and `poll_body` is run to completion; the
select loop relies on both. Everything is QoS 0 with a clean start, so
the session is stateless. **The broker must speak MQTT 5** (Mosquitto
1.6 or later). No TLS: about 100 KB of flash and tens of KB of RAM,
and the LAN is already the trust boundary.

**Settings.** `mqtt_host` (64 bytes, empty means off), `mqtt_port`
(default 1883), `mqtt_user` (32), `mqtt_password` (64, never read
back); record version 10 (`RECORD_LEN` 336, `storage.rs`'s scratch
512). Flags bit 1, `FLAG_MQTT_CLEARED`, keeps a cleared host cleared
on a build with `MQTT_HOST`. Clearing the host clears the user and
password too. The page takes the keys without the token (the user's
decision); `/api` only with the `Bearer` token, since another broker
would be handed the stored password: hence `form::Allow { network,
broker }`. The JSON gained `mqtt` (`off`, `connecting`, `connected`)
and `ringing`.

**Topics.** The device id is `clock-` plus the last three MAC bytes in
hex, so a rename leaves no stale retained message. Under the device's
base topic: `status` (retained, also the will: `online`/`offline`),
`state` (retained, the `/api` JSON byte for byte, so there is one
serialisation), `set/<key>` (in, the value as `/api` takes it, through
`form::apply_one` on a copy; only `mqtt::SET_KEYS`, the rest logged
and dropped) and `cmd` (`play`, `silence`, `reload`). The state goes
out on connect, on a settings change (debounced 300 ms), on the
ringing edges and every 60 s. `tz` is not on the bus and `temp_offset`
is, as a `number`: the test tying discovery to `SET_KEYS` forced the
choice.

**The task,** `mqtt_task` in `src/mqtt.rs`, station only: resolve and
connect (5 s timeouts), 60 s keepalive (ping after 45 s idle, drop
after 10 s without `Pingresp`), back-off 5 s doubling to 60 s.
Publishes happen between polls, so no two client futures exist at
once. `heal`'s resets and the OTA reboot set `GOODBYE` and wait up to
500 ms for `offline` and DISCONNECT, so Home Assistant sees the clock
gone within a second, not after the broker's 90 s; a panic leaves it
to the will.

**Sizes.** `StackResources` 11, `RECEIVERS` 4. The task's future is
about 15 KB (9 KB estimated): the main stack went from 84.5 to
69.6 KB. The settings page is 5149 of 6016 at its worst. `serde_json`
is a host-only dev-dependency, under
`[target.'cfg(not(target_os = "none"))'.dev-dependencies]`.

**Bench, 2026-11-07.** The user checked the device in Home Assistant
with three clocks on the bus; on test-clock every path was exercised
with a subscriber on the host (`set/`, the refused keys, the birth
resend, ringing and silence, the goodbye at OTA and `/reload`, a wrong
password's `not authorized` and back-off, a cleared host). Fixed in
the session: `cmd` was never subscribed (`SUBSCRIBE_MAXIMUM` is 3),
and the state went out twice at connect (the receiver is now drained).
Not fixed: the root `build.rs` reruns `git describe` only when
`.git/HEAD` or the index changes, so a dirty tree built right after a
commit carries the clean hash.

### Web flasher and releases — built 2026-11-07, `v0.1` released; browser install not tried

A clock is flashed the first time from a browser: a GitHub Pages page
with an ESP Web Tools install button (Web Serial, Chrome or Edge)
offering every release, newest first; modelled on the user's
`esp32-bthidbridge`.

- `.github/workflows/release.yml` on tags `v*`: `build` (as `ci.yml`,
  then `esp32clock-ota-<tag>.bin` for `/update` and the merged
  `esp32clock-<tag>.bin`), `release` (both files on the release),
  `pages` (`docs/<tag>/manifest.json` and the images, again under
  `docs/latest/`, deployed to `gh-pages` with `keep_files: true`).
- The version is `git describe` without `--tags`, so a release tag
  must be annotated.
- `docs/index.html` fills a select from the GitHub releases API and
  points the button at `<tag>/manifest.json`, or `latest/` when the
  API is unreachable. `new_install_improv_wait_time` is 0 (no Improv;
  the default waits 10 s).

Bench finding: the merged image (`espflash save-image --merge
--skip-padding -s 4mb`) is contiguous from 0 and so carries erased
`nvs`; `write-bin 0x0` of it wiped the bench board's settings. The
installer therefore flashes four parts (bootloader and partition table
cut from the merged image with `dd`, an `otadata` of 0xFF, the OTA
image at 0x20000) and never touches `nvs`, so its erase prompt means
what it says; the merged file stays on the release, documented as
clearing the settings. `write-bin` leaves the chip in the bootloader
(`espflash reset` after). The `v0.1` tag was moved to the fix.

Owed: the install button in a Web Serial browser was not tried, and
no check of the four-part install is recorded.

### Timezone rules: POSIX TZ strings and daylight saving — built and verified on test-clock 2026-11-07; panel, real transition and phone owed

The timezone setting is a POSIX TZ rule (`PST8PDT,M3.2.0,M11.1.0`,
`<+0545>-5:45`) evaluated on the chip. The user asked for it after
learning Home Assistant cannot push an offset over MQTT without an
automation.

- `core/src/tz.rs`: `parse` is a `const fn`, so a bad build rule
  fails the build. Offsets positive west as POSIX, stored
  east-positive; `Mm.w.d`, `Jn` and `n` dates; times −167 to 167 h.
  Refused: a daylight zone without rules, trailing junk, over `TZ_MAX`
  (48) bytes. `Tz::period_at(unix)` gives offset, `dst`, name;
  `fixed(minutes)` writes `<-07>7`.
- `Settings.tz` replaced `tz_offset_minutes`. Record version 11
  appends the rule (`RECORD_LEN` 385) and still writes the standard
  offset in minutes at bytes 7 and 8, so a rolled-back image shows
  standard time; older records read as `fixed(minutes)`. `TZ_RULE`
  seeds it and wins over `TZ_OFFSET_MINUTES`; `TZ` is not read (shells
  export an IANA name).
- `/api`: `tz` takes a rule or plain minutes and answers a string (a
  type change); read-only `utc_offset` and `dst`, `null` before the
  first sync, from `http::status`. `set/tz` on MQTT stays refused.
- The settings and setup pages have the field and a button that runs
  `posixTz(zone)` from `/tz.js`, loaded as `/tz.js?v=<version>` since
  a browser caches it a day. `scripts/tz-check.mjs` checks the
  function under Node.
- The alarm (`ui`, host tests only): one in the skipped hour rings at
  the crossing, but only when the UTC offset changed between two
  ticks (a plain gap is the boot grace's); the last ring is kept in
  local seconds (`last_alarm_local` in the carry record) so the
  repeated hour rings once.

From the headless-Chromium run against the real page: Chromium has
Vancouver on PDT from March 2026 with no return, so the script reads
the rule from next year and checks candidates against the six years
after, which also tells `Saturday 24:00` (Chile) from `Sunday 00:00`
and Nuuk's `-1:00`. Zone names come from the first locale that has
letters, else `<+03>`. A `const` assert in `src/http.rs` keeps the
script (5.7 KB) under `RESPONSE_BUF`.

Verified on test-clock over the air: the version 10 record came up as
`<-07>7`; CET, Pacific and Sydney rules gave the right local-time
line; an IANA name is a `400`. Owed: the panel was not watched (only
the JSON), the alarm across a real transition, the setup form's
button on a phone.

### Codex review of 2026-11-07: eleven findings fixed — built 2026-11-07, released as v0.3, soaked 11 h 2026-11-08; the fixes' own checks owed

Another agent's review (`codex-review.md`) listed fourteen findings.
The user set three aside as not bugs: disarming does not silence a
ringing alarm, the display task freezes during an upload, a reset
discards a change inside the save debounce. Fixed:

1. The settings page handed the MQTT password to a new host:
   `form::apply`, after the pairs, on `Allow::PAGE` refuses a host
   changed to another non-empty one with a password stored and none
   typed (`FormError::BrokerPassword`). `POST /play` uses
   `Allow::ALL` on its scratch copy.
2. A rule past 14 h lost the whole record: `to_record` clamps the
   legacy minute field to ±14 h, `from_record` checks it only below
   version 11.
3. MQTT startup swallowed a broker or hostname change: the value is
   kept and compared after the start (`Outcome::Settings`, or the
   discovery resent).
4. A refused subscription looked like success: `Event::Suback` by
   packet identifier; a reason code of 0x80 or above, or no answer in
   `SUBACK_TIMEOUT` (10 s), fails the session.
5. `tz.js` invented fixed rules: `changes(yr)` samples every month;
   anything but none or two changes throws ("no rule for
   Africa/Casablanca"), shown on the page.
6. A transition displaced across New Year: `period_at` takes the
   latest of six candidates over three years.
7. A failed upload could resume the scan under a reset: `scan::halt`
   latches `FINAL` with `STOP` in one critical section and `resume`
   honours it.
8. A failed save is retried, 2 s doubling to 60 s.
9. A `POST` without `Content-Length` is `411`, a `Transfer-Encoding`
   is `400`.
10. Two rescans shared one `Signal`: a scan generation counter
    (`request_scan`, `scan_done(gen)`).
11. A bad record is logged by version and length, not contents.

Learned: a second `Response` in another arm of the listener loop is
not overlapped by the compiler; `.bss` grew 17.5 KB and the main
stack fell to 50.3 KB until it was one buffer per loop (67.1 KB),
found by `llvm-readelf -S` against a build of `HEAD`.

Owed, none checked on a clock: `POST /` with no body is `411` and the
alarm stays armed; a changed broker host with an empty password gives
the notice, and connects with one typed; the network survives a
reboot with `tz=<+15>-15`; a reload during an upload that fails
leaves the panel dark, never a lit row; Rescan from two tabs returns
both; the MQTT page line when the broker's ACL denies `set/+`.

### Scan restart gap: measure the whole dark time — built and measured on the bench 2026-11-07; superseded by the ring

Whether Wi-Fi caused the restart gap's maximum. From the sources
(esp-hal 1.2.2, esp-rtos 0.4.0, esp-radio 1.0.0-beta.1): nothing
preempts the scan executor by priority; what delays a restart is a
critical section anywhere, since esp-sync's `RawMutex` masks every
interrupt on the C3. The long holders are the Wi-Fi blob's lock, the
heap lock (a buffer per frame) and esp-println's lock while it
busy-waits on a full USB-JTAG FIFO with a monitor attached.

The old statistic (1 to 2 µs, maximum 49 to 99) covered only the
slice between the task waking and the restart. Rebuilt as the
interval between starts less `scan::TRANSFER_NS`, with a histogram,
the bench read: average 24 to 28 µs, maximum 0.7 to 1.1 ms; three
quarters of gaps in 10 to 19 µs (the restart's fixed cost), a smear
at 20 to 89 µs that page loads did not move, and some 550 a minute of
90 µs and up, near the beacon rate. A 1 ms gap is a 30 % dip of one
lap, invisible. Remapping SPI2's done interrupt to priority 3 (it
shared priority 1 with the Wi-Fi ISR) changed nothing measurable, so
the cause is short critical sections no priority reaches. Trimming
them was not pursued; the segmented transfer below replaced all of
this, statistic and remap included.

### Scan ring spike: the segmented transfer on the bench — built, run and scoped on the bench 2026-11-08

`src/ring.rs` behind a feature `ring-spike` (since removed) asked
whether the C3's SPI2 loops a DMA-controlled configurable segmented
transfer in quad mode from our descriptor chain. Espressif never
enabled its driver for it on the C3 (no `SOC_SPI_SCT_SUPPORTED`) and
esp-hal does not expose it.

From the TRM (v1.4, 27.5.8.5) and esp-idf's `spi_ll.h`: each
segment's CONF state reads a conf buffer from the TX DMA stream, a
bitmap word with a magic in bits 31:28 (against
`SPI_DMA_SEG_MAGIC_VALUE`) and bits 0 to 13 naming the registers that
follow, ADDR to DMA_INT_CLR; only changed ones need a word.
`SPI_USR_CONF_NXT` (USER bit 15) says another segment follows;
`SPI_CONF_BITLEN` is the CONF state's length, value + 5 APB clocks.

What worked: `Ring` as an esp-hal `DmaTxBuffer`, a conf descriptor of
8 bytes (bitmap with magic 0xA and bit 3, then USER) and eleven data
descriptors, the last pointing at the first, owner check off as in
`DmaLoopBuf`. One ordinary quad transfer first, so esp-hal programs
every register; the ring then starts through
`half_duplex_write_buffer`, whose `prepare` call lands after SLAVE is
zeroed and before `usr` is set, the place for `usr_conf`, the magic,
`conf_bitlen` and `usr_conf_nxt`. Stop: clear `usr_conf_nxt` in the
conf buffer; the next CONF state applies it.

Results: `trans_done` is not raised per segment in CONF mode (laps
are counted by the GDMA's `out_eof`, `suc_eof` on the last
descriptor); 19 447 laps in 60 s, a fixed gap of about 0.6 µs, no
magic errors; a stop took about 3750 µs. On the user's scope, with
the CONF state stretched to 50 µs (`CONF_BITLEN` 3995) and chip
select on GPIO10 as the trigger, CLK and DATA sat flat low through
the CONF state after the blanking frame.

### Soak of v0.2 and v0.3 on the bench — 2026-11-07/08, passed

Released images on `test-clock.lan` (no sensors, −40 to −48 dBm),
with a serial log and `scripts/watch-clock.sh` every 30 s (a 498-byte
ping and a `GET /api`).

- v0.2, 4 h 28 min: no reset, heal or warning; one ping missed 12 s
  after boot; heap free 54080 bytes with one 188-byte step down.
- Update to v0.3 by `POST /ota` with a panel PIN: 23 s, time and
  settings kept.
- v0.3, 10 h 59 min: no reset, heal, panic, WARN or ERROR; 1315 of
  1315 pings and fetches; one MQTT session; heap 54032 bytes but for
  four one-minute dips of up to 8520 (frames queued in the Wi-Fi
  driver, the heap's only user).
- The crystal runs about 24 ppm fast; the hourly NTP sync corrects
  80 to 90 ms.

Not covered: the alarm never rang, no save retry or double rescan
ran, and the codex fixes' checks stay owed.

### Hardware-chained scan: the ring as the scan path — DONE, verified on the panel by the user and on the bench 2026-11-08; soaked to 2026-11-12

The spike's ring became the scan path (`CONF_BITLEN` 0). `scan_task`
runs one dark transfer, builds the ring, hands both buffers to the
publisher (`ring::init_slots`) and waits on `STOP_REQ`. A stop clears
`usr_conf_nxt` (`request_stop`), waits up to 20 ms for
`dma_seg_trans_done`, cancels if `usr` is still set (never seen;
counted) and sets `HALTED`; `resume` restarts the ring.
`ring::publish` waits `HOLD` after the previous swap, builds into the
spare and repoints the data descriptors (`point_at`); the display
task is the one publisher. The `FREE`/`READY` channels, `GapStats`
and the priority-3 remap went.

Bench, test-clock, 2026-11-08:

- A regression from the codex fixes: `scripts/reload.sh` got `411`,
  as body-less posts (`/reload`, `/ap`, `/pin`, `/scan`) are a bare
  `curl -X POST`. Now `http::reads_body` names the routes that read a
  body (`/`, `/api`, `/play`, `/token`, `/ota`) and only those get it.
- A truncated upload left the running image and the ring resumed; a
  full upload booted from `ota_1` with the time carried;
  `scripts/reload.sh` came back in 3 s through the latched halt.
- The record's token wins over the build's: the bench path is a PIN
  from `POST /pin`, then `POST /token`.
- Two station drops under HTTP traffic in a quarter hour, one into a
  self-heal that worked. The user's verdict: the board (a SuperMini
  with the known antenna mismatch), not the firmware.

The user confirmed the display on a panel the same day. Owed: the
buttons without a blink, the temperature face and the alarm on a
panel with the ring.

Soak of `v0.3-3-gdf7f547` on test-clock from 2026-11-08: at 90 h 58
min no disconnect, heal, stop or panic, one association and one MQTT
session, the descriptor index moving, heap on 53 844 bytes after one
188-byte step. Drift −23.1 ppm (83 ms an hour; 2 s a day without the
network). 8 of 10 911 gateway probes lost, clustered 06:00 to 08:30
on two mornings while the host's pings passed: the router or the air,
not settled. At 92 h 13 min, 2026-11-12, two `MissingAcks` drops
16 minutes apart at the usual RSSI, each rejoined in 9 s with no heal
or reset; the heap went back to 54 032, so the step belonged to the
association. The user took it as the recovery working as designed;
the cause is open (the second fell during another board's ping test
on the bench).

### Thermometer offset: default −2.5 °C — DONE, verified on the bench 2026-11-09

The user's clocks typically need −2.5 °C, so
`config::DEFAULT_TEMP_OFFSET_DC` is −25 tenths, with no record
change. It reaches a clock with no record or a record of version 7 or
older; version 8 or later keeps what it stores, 0 included, since a
stored 0 cannot be told from an untouched one and a reading that
moved by itself after an update would be worse.

Bench: a board flashed with `nvs` erased logged `temp_offset_dc:
-25`; a stored 0 stayed 0 across a reflash. The version 7 case is a
host test only. (That board has no NTC or pull-up: GPIO0 floats at
842 mV and reads 48.8 °C instead of no sensor.)

### Sampled sound on the buzzer: PCM in flash, modulated on the chip — DONE, heard and accepted by the user on white-clock 2026-11-12

The user's idea: the buzzer line is a 1-bit speaker. The clip is raw
PCM (8 kHz, 8 bits, unsigned, mono; 7.8 KB a second) modulated on
the chip rather than a stored bitstream: the C3's GDMA reads internal
RAM only, so a RAM window is needed either way, and PCM is a quarter
of the flash.

- I2S0's transmitter in TDM mode, data line only, sends the buffer's
  bytes MSB first; GDMA channel 1 loops over a ring of 1 KB
  descriptors refilled ahead of it (`core::clip::Pacer`).
- GPIO5 is a released open-drain `Output` turned into a signal with
  the GPIO matrix's inverter, connected to `I2SO_SD` after the
  transfer starts on zeros; ending takes it back as a plain output,
  which clears the inverter LEDC must not inherit.
- Modulator (`core/src/clip.rs`): second-order sigma-delta, full
  scale 0.7, 30 ms ramps against a thump. Simulated in-band SNR on
  the test clip at 256 kbit/s: 35 dB second order, 22 first; 27 at
  128. Default 256 (`CLIP_KBPS`).
- `ALARM_CLIP=<file>` adds the clip as a melody; `scripts/wav2pcm.sh`
  converts with sox. The test clip (Futurama's theme) stays in the
  ignored `wav/`.

Hearings on white-clock, 2026-11-12: recognisable but very quiet,
with phantom button presses. A pulse-width version (one centred pulse
per carrier period, `CLIP_CARRIER_KHZ`) was louder, still quiet, and
an 8 kHz carrier whines (the user), so 16 kHz; a harder limiter
(20 dB) and a 900 Hz high-pass changed nothing audible. The user's
scope showed 16 kHz pulses to the full 5 V at the buzzer, as wide as
on GPIO5, with a −10 to −15 V kick at turn-off: the drive is right
and the volume is the buzzer's, a resonator near 2 kHz that the beeps
hit as a square wave. The phantom presses were a short between
DIP-16 pins 9 and 10 (the two buttons) under the module on that
adapter board; both theories of interference (edge coupling, the
10 ms poll aliasing with the carrier) were wrong, and a ten-read
button check built on the second was reverted. After the user's
rework three plays logged `clip: 1733 chunks, 0 underruns` and no
setting moved. Not checked: that each button still does its own
thing.

### Beeper clips: a ZX Spectrum tune as it was played — DONE, heard and accepted by the user on white-clock 2026-11-12

The PCM clip was too quiet and not good on a resonant buzzer (the
user), so next came music written for a 1-bit speaker: Trantor's
title tune. The game's player was read in disassembly and not ported
(its data format would have taken days); the tune was captured.
`scripts/zxbeeper.py tape out.bpr --seconds N [--skip N] [--stretch
F]` runs the tape in SkoolKit's simulator and writes the speaker's
flips as run lengths: a magic, the total in ticks, then LEB128
counts, off and on alternately (`clip::Beeper`). Trantor's pulses are
47 µs at the median, on 3.6 % of the time. `clip::Stream` picks the
kind by the magic, so `ALARM_CLIP` takes either and `src/clip.rs`
plays both through one ring.

### Two-channel tunes: the "Wham! The Music Box" player, re-implemented — DONE, heard and accepted by the user on white-clock 2026-11-12

**The Wham player (removed).** The user disliked Trantor on the
unmodified buzzer and asked for the five tunes of `WHAM.TAP` (Mark
Alexander's editor, 1985). `clip::Wham` re-implemented its player
from the disassembly (two square waves sharing the line by time at
36 kHz, three drums) and matched the emulator within 0.4 ms over
20 s; its 48 T-state slots crackled at 256 kbit/s (3.5 bits), so it
ran at 512. Once it played right the user found the arrangements not
to his taste: the player and `scripts/wham2clip.py` were removed
(commit b6ff2c6 has them); `Runs` and `RunSource` stay for the beeper
clip.

**Several clips.** `ALARM_CLIP` is a comma-separated list.
`core/build.rs` writes `clips.rs` (`CLIP_COUNT`, `CLIP_NAMES`,
`CLIP_DATA`); `melody.rs` appends a melody a clip and refuses one
under a second, over 1 MB or with a taken name. The API name is the
file's name in letters, the label the same in words unless
`ALARM_CLIP_LABEL` gives one at the same place in its list.

**The changed circuit.** The user fitted a 1N4148 across the buzzer
as a flyback diode and lowered R5, the base resistor, from 10 k to
3.2 k; on it sigma-delta Futurama at 256 kbit/s (`CLIP_MODULATOR=sd`,
beside the default `pwm`) and Trantor sounded fine. The README has it
as an optional mod with R5 at 5.1 k (the user's value): the diode
removes the negative kick, the lower R5 makes the collector's signal
squarer and louder. The diode's orientation there is derived from
the circuit, not read off the board.

**The clip task.** The user heard skips under Wi-Fi activity and, of
three cures offered, chose moving the refill off the thread executor.
The buzzer task cannot move (LEDC's driver is not `Send`), so
`clip::clip_task` on the scan's interrupt executor owns I2S0, the DMA
channel and the ring; the buzzer task calls `clip::play(index,
cancel)`; the pin is shared by turns through a `clone_unchecked`
handle in `main.rs`. It modulates for a millisecond or two at
priority 2 every 16 to 32 ms: if the probe or the association
suffers while a clip plays, look here. The ring grew to 16 chunks
(`SLOTS`; main stack 51.5 KB with clips). Neither cured the skips;
both were kept.

**The real bug: `out_dscr` reads two ahead.** A skipping clip logged
0 underruns, and a trace showed the GDMA's `out_dscr` naming the
descriptor two on from the one being read. The pacer had taken it
for the one being read, so each chunk was overwritten, as the DMA
entered it, by the one a lap later: the skips, the stream overlapping
itself at the start, part of the crackle and probably some of the PCM
clip's poor first hearing. Now the clock paces alone
(`Pacer::played`; the bit clock and the timer share a crystal) and
the register is only checked: a slot not `Pacer::DSCR_AHEAD` (2) on
from the clock's chunk, give or take one, is counted in `clip: N
chunks, U underruns, S stray positions`. A host test runs the
playback loop against a model of the DMA; white-clock logged 0 and 0
with the page fetched meanwhile. To look at: `ring::position`, the
scan heartbeat's figure, reads the same register of channel 0 and is
by this two on; only the log line depends on it.

**Accepted, 2026-11-12,** by the user on white-clock with the diode
and R5 at 3.2 k.

**Trantor in the repository.** Advised that neither Futurama (a
recording of a television theme) nor Trantor (a capture of a game's
music) is licensed, the user decided: Futurama stays in `wav/`,
Trantor goes in as `clips/trantor.bpr`, with `clips/README.md` naming
David Whittaker and saying the file is not under the project's
licence. The tune restarts at 126.5 s (a first cut at 141.5 s ended
on its own beginning, as the user heard), hence `--seconds 126.5`.

**Chronos, tried and dropped.** Tim Follin's title music, from a
cracked tape that needed `--settle` and `--start` in
`scripts/zxbeeper.py`. Its pulse widths step by 14 T-states against
the clip's tick of 27 (1/128 000 s), so the format became `BPR2` at
1/512 000 s, a beeper clip running at its own
`clip::BEEPER_BIT_RATE` (512 kbit/s, `Stream::bit_rate`); `melody.rs`
refuses a `BPR1` file, which would be taken for PCM. The user
disliked it still, and as PCM from his own emulator recording too
(90 s of the 180 fitted, 98.9 % of the slot), so it was removed in
both forms. Kept: `BPR2`, the two options, and a skip argument and
end fade in `scripts/wav2pcm.sh`.

The clocks' set at the end of the day:
`ALARM_CLIP=clips/trantor.bpr,wav/futurama.pcm` with
`CLIP_MODULATOR=sd CLIP_KBPS=256`, Futurama as the loud conversion
(900 Hz cut, 20 dB into the limiter).

### Tracker modules (XM) — player built 2026-11-12, settings chosen by ear on white-clock 2026-11-12

The user's next source of tunes: XM modules, Orion's "halftime"
(4 channels, 12 642 bytes, 43.5 s) and "keygentune2003 orn" (57.6 s).
A trial render (`openmpt123`, then `scripts/wav2pcm.sh`, 349 KB) was
liked, so the module itself is played, at 28 times less flash.

**The player, `core/src/xm.rs`.** Not the `xmrsplayer` crate: 0.15
needs `alloc` and is 22 000 lines with serde. Ours reads the module
in place, eight channels at most. Played: effects 0 to 6, 9, A to D,
E1 E2 E9 EA EB EC ED, F, G, K, the volume column, envelopes with
sustain and loop, forward loops, the linear frequency table,
FastTracker's arpeggio tick order, an instrument's auto-vibrato (as a
sine). Not played: tremolo, pattern loop and delay, multi-retrigger,
the Amiga table, more than one sample an instrument. Channels are
mixed with linear interpolation above the output rate and averaged,
then high-passed and limited. `clip::Player` takes PCM or a `Tune`.

**The clip, `XMC2`** (`scripts/xm2clip.py`; `XMC1` is refused at
build time): a 32-byte header, tables of the patterns and samples,
samples as plain bytes, ping-pong loops unrolled; the converter lists
what the player lacks. Per clip in the header: the limiter's drive
and release (bytes 10 and 11, 0 for `xm::DRIVE` and `RELEASE_SHIFT`),
the high-pass's passes and coefficients (the chip has no tangent),
the restart order, the rate (byte 17 bit 0). `--raise
INSTRUMENT:OCTAVES` moves an instrument in the clip.

**Checked on the host.** `CLIP_PCM_DUMP=file cargo host-test
dump_the_tune -- --ignored` writes what a built-in module gives the
modulator; `CLIP_RAW=1` the mix before the shaping (a quarter of it).
Against openmpt's render the raw mix is within 1 dB in every band,
bar a droop of 1 to 2.5 dB above 2 kHz from the interpolation.

**Chosen by ear, each as an A/B in one firmware.**
- High-pass 300 Hz, one pass. The 900 Hz twice that suited Halftime
  removed Keygen's bass line (44 to 148 Hz); lower cuts lost because
  the limiter works on the sum (above 300 Hz −4.7 dB with the cut,
  −8.0 dB with none).
- 16 kHz, for modules and then PCM (a `PCM1` header with the rate,
  `clip::PCM_MAGIC`; bare PCM stays at `PCM_HZ`).
- Limiter: `DRIVE` 127, `RELEASE_SHIFT` 7 (16 ms, RMS 0.42). The
  louder `--drive 160 --release 5` (RMS 0.59, 16.5 % of samples at
  the rails) won at 8 kHz and lost at 16.

**In the repository and every build** (the user's decision): the
`.xm` files in `mods/`, `clips/halftime.xmc` and `clips/keygen.xmc`,
credited in `clips/README.md` (a credit, not a licence).
`core/build.rs` takes `DEFAULT_CLIPS` when `ALARM_CLIP` is not set;
set to nothing is a build with none. Default image 1 036 784 bytes,
main stack 50.6 KB (67.2 KB with no clips). Futurama stays local.

**The alarm loops a clip with no gap — built 2026-11-12, not yet
heard.** `clip::play(index, looping, cancel)`: a beeper clip starts
over at once, a module follows its jump back or the restart order,
PCM goes last sample to first; a preview plays once. `CLIP_LOOPS`
from `core/build.rs` is true for a beeper clip or a module, false
for PCM (`ALARM_CLIP=file:loop` or `file:once` overrides); a clip
that does not loop rings with `repeat_gap_ms` between passes.

### Sixteen-bit samples from a module to the modulator — done, heard on white-clock 2026-11-13

The user asked whether I2S's PDM mode could replace the software
sigma-delta, with 16-bit samples. Taken cheapest first: wider samples
in software, the line at the rate PDM needs, then the hardware.

A module lost width only where `shape` rounded to 127 steps, so width
costs no flash (16-bit PCM would double it). `Tune::next_sample`
gives `i16` in 256ths of the 8-bit step (`xm::STEP`); a clip asks
with bit 1 of header byte 17 (`xm2clip.py --bits 16`), which an older
firmware masks and plays narrow.

Heard on white-clock (diode, R5 at 5.1 k; first recorded as 3.2 k,
corrected by the user): at 256 kbit/s the wide clips were smoother,
at `CLIP_KBPS=512` (1.024 Mbit/s for a 16 kHz clip) much better,
while the 8-bit ones hardly changed: the 8-bit step was the floor,
the modulator's noise the rest. Both became defaults (`--bits 8` for
the other). Never read: the processor's share and the underrun count
at 512.

### Hardware PDM for a module: a spike to hear — done, heard on white-clock 2026-11-13 (128 bits a sample kept, 64 is noise)

I2S0's PCM-to-PDM converter (esp-hal 1.2.2, `I2s::new_pdm`) in place
of the software loop: no modulating on the processor, a ring four
times as deep in time, 2.048 Mbit/s; at the cost of a second path in
`src/clip.rs` (beeper clips are bits and stay on TDM) and a modulator
no host test sees. `Player::for_converter` and `fill_samples` write
the levels, ramps included, as `i16`, with −32768 (the released line,
`clip::SAMPLE_REST`) after the end; `Stream::bit_rate` is the ring's
rate. DAC line mode, mono, the driver's high-pass off (it would
remove the ramps and the rest level).

Learned:
- esp-hal 1.2.2's `pdm::regs_v2::configure_tx` clears
  `tx_pcm_bypass`, which the IDF leaves set, putting the A-law
  compander before the converter: white noise. `src/clip.rs` sets the
  bit and latches it with `tx_update`, through `I2S0::regs()`.
- In PDM mode the driver makes the master clock's divider whole, so
  rates are 160 MHz over integers; 16 kHz came out at 16 447 Hz.
  `clip::converter_clock` picks the bit-clock divider (8 to 20)
  nearest and gives the ring's true rate, 16 026 Hz, 3 cents sharp.
- The converter does not work at fp/fs of one (64 bits a sample):
  noise. At two (128 bits) the user found it cleaner than software.

On test-clock: 0 underruns, 0 stray positions, a beeper clip after a
PDM one unharmed. Decision: PDM for modules and PCM, the bit path for
beeper clips.

### PDM is the modulator for PCM clips and modules — done, heard and accepted by the user 2026-11-13

The build's choice, not the clip's: `CLIP_MODULATOR=pdm` is the
default beside `sd` and `pwm` (`CLIP_KBPS` and `CLIP_CARRIER_KHZ`
matter to those only). `config::CLIP_PDM`; `clip::CONVERTER`
`Some(128)`; `Stream::new` gives every PCM clip and module to the
converter. The ring stays 16 KB, half a second of samples.

**Level.** Full scale had been the software loop's stability limit,
0.7 of the rail. An A/B of 0.70, 0.85, 0.95 and 1.00 on the modules,
heard on white-clock and on martin-clock's stock buzzer circuit: the
user chose 0.85; 0.95 overloaded in places, 1.00 clipped.
`clip::CONVERTER_LEVEL` is 438. Futurama through the converter was
heard and accepted.

Owed: a scope on the line (the rest level around a clip, the pulse
rate); esp-hal's `tx_pcm_bypass` reported upstream.

### Commodore 64 tunes: a SID on the chip — built 2026-11-13, heard on martin-clock, 4 steps a sample chosen by the user

First tune: "Donkey Kong" by Sascha Zeidler (Linus), a PSID v2 for
the 8580. A `.sid` is the C64's player routine, not notes.

**The 6502 on the host, the SID on the chip.** `scripts/sid2clip.py`
runs the routine in its own 6502 and records each frame's register
writes; it hashes the machine's memory after every call and marks the
loop where a state returns. A 6502 on the chip was rejected: 64 KB of
address space and a routine of unknown length on the interrupt
executor. PSID with a play address only. A tune with no loop is cut
at `--seconds` and ends on a frame of zeroed registers.

**The clip, `SID1`.** A 40-byte header (model, the shaping, the C64's
clock, cycles a frame, frame count, loop offset, steps a sample in
byte 38), then pairs of register and value and a byte with the top
bit set for the frames to wait. A frame's writes apply at its start.

**`core/src/sid.rs`.** From the chip's published behaviour:
oscillators stepped at 64 kHz and four averaged to a sample (a
point-sampled saw or pulse aliases), envelopes once a sample; ring
modulation, sync, the test bit, combined waveforms as an AND, the
rate counter's wrap at 32 768 (the late note hard restart exists
for), the filter with an 8580's linear cutoff or a 6581's table. Not
played: samples through the volume register, paddles, reads of `OSC3`
and `ENV3` (the converter says when a routine reads them). The
high-pass and limiter moved to `core/src/shape.rs` (`Shaper`), shared
by every player.

**Measured.** 83.0 s looping from 37.1 s, 62 KB, 748 bytes a second,
mostly pulse-width sweeps. Against libsidplayfp 3.1.2 the registers
match frame for frame; band energies within 1 dB up to 2.4 kHz. On
test-clock the `clip:` line, which now ends with the time spent
filling the ring: a module 12 % of the processor, the SID 26 % at 4
steps a sample, 17 % at 2; no underruns.

**Heard, 2026-11-13.** The user preferred 4 steps to 2 (`--steps 2`
remains). A click at the start was the tune's own (voices gated in
frame 0, closed in frame 1), made loud by the limiter at full gain:
`--mute 2` removes it. Donkey Kong stays local (the user's decision),
in the ignored `sid/`.

Owed: a 6581 tune, and one with ring modulation or sync, against the
reference.

### ZX Spectrum tunes: an AY-3-8910 and a PT3 player on the chip — built 2026-11-13, heard on white-clock; the register comparison with a reference player owed

The tunes, in the ignored `pt3/`: "Hibernation" by MmcM (96.7 s),
"synchronization" by nq (11 147 bytes, 513.8 s), then "resurection"
(PPK) by Nik-O (172.9 s, a real loop from position 14).

**Player and chip both on the clock.** A PT3 is notes, its player one
fixed routine, the module 6 to 11 KB where a register record would
be 100 KB or more. The clip, `PT31`, is the module untouched behind a
40-byte header like `SID1`'s: flags (bit 0 a YM2149's 32 levels,
bit 1 start afresh at the end), the chip's clock (1 750 000 Pentagon,
1 773 400 Spectrum 128), cycles a frame, frames, loop frame.

- `pt3::Tune`: Pro Tracker 3.x and Vortex Tracker II, one chip, from
  the format's behaviour, with joric/libpt3 (no licence stated) read
  and not copied; unlike it, a sample line's envelope offset is
  sign-extended, as the Spectrum's routine does. Reads are
  bounds-checked and answer 0. TurboSound is refused.
- `ay::Chip`, by itself so another AY format is only a second
  player. Exact: the output is constant between ticks of clock/8, so
  a sample is its mean, found change to change. Checked against a
  tick-by-tick model, which caught a counter clamped to a half-written
  period at the register write (clamp at the next tick).
- `scripts/pt32clip.py` walks the patterns for length and loop; the
  machine defaults by the tone table. Hibernation and Synchronization
  loop only to their last position, wrong for an alarm, hence
  `--restart`.

**A stack overflow, found on the clocks.** With a fourth player
`clip::Stream::new`, inlined into the clip task's `poll`, had a frame
of 6016 bytes, and that task's frame lands on whatever stack the
interrupt found: a panic on every tune. `Stream::new` is now never
inlined and builds in place through `Player::take_*`: 2.8 KB at worst.

On test-clock: module 12 %, SID 29 %, AY 13 % of the processor; the
shaper and the ring's samples are a floor all share. Hibernation as
AY against YM: the user kept AY (`--chip ym` remains).

Owed: the register comparison with libpt3 (`CLIP_REG_DUMP=file
CLIP_INDEX=n cargo host-test dump_the_registers -- --ignored` writes
the clock's side, fourteen bytes a frame); building the downloaded
reference was declined by the permission check, the user's to run or
waive. Not recorded: the user's judgment of pitch and tempo, and
whether any tune becomes a default clip.

### A Stop button beside Play — built 2026-11-13, checked on test-clock by curl

`POST /stop` silences the buzzer (`buzzer::stop`) and ends a ringing
alarm as a press would (`display::silence`). No token, as `/play`
has none; a body is ignored. The page's `Stop` shares Play's script
handler through the button's `formaction`: a second handler would not
fit (5865 of 6016 bytes at worst). On test-clock `curl -X POST /stop`
cut a preview within 2 ms. Owed: the page's button and a ringing
alarm, the user's to try.

### Quieter noise for the AY's drums — built 2026-11-13, 11/16 chosen by ear on white-clock

The user found the drums too aggressive on the buzzer. A channel with
its noise bit open is mixed at a fraction of its level,
`ay::Chip::set_noise_level` in sixteenths, header byte 38,
`pt32clip.py --noise N`; scaled, not filtered, so `run` stays exact.

**The level table was wrong.** Against Ayumi and MAME's `ay8910.cpp`
the logic agreed, the levels did not: `ay.rs` had the data sheet's
even 3 dB a step (as openMSX), while measured chips are uneven (level
12 at −6.2 dB, not −9.0), so notes at 10 to 13 were 3 to 4.4 dB too
quiet beside drums at 15. `ay.rs` now has `LEVELS_AY` and
`LEVELS_YM`, Ayumi's measurements. MAME's tied-pins model was not
taken (Pentagon tunes; the limiter does much the same).

On the measured levels the user chose 11/16 (−3 dB) over full and
8/16. Open: measured against even was never compared by ear.

### A low-pass for the AY's noise — built 2026-11-13, heard on white-clock and not adopted

The user's alternative. `ay::Chip::run_parts` answers `ay::Parts`
(`plain`, `noisy`, `open`, `noise`); with header byte 39 set,
`pt3::Tune` low-passes the fraction of ticks the noise was open (one
pole, `--noise-lowpass HZ`) and gates the listening channels by it,
so the tone under the noise is not filtered, as the user asked.

At 3 kHz and 1.5 kHz against 11/16 unfiltered, the user kept the
latter: `--noise 11` is the converter's default, `--noise-lowpass` an
option no clip uses.

### Bass cut for a speaker: an A/B on headphones — heard on test-clock 2026-11-13

Waiting for 32 Ω speakers, the user listened to test-clock's GPIO5 on
headphones through the clock's stage on a breadboard
(`bench/buzzer-sim.png`: a BC557, 5.1 k into its base, 100 Ω load).
The clips, cut for a buzzer, were thin there. Futurama cut at 60 Hz
once was good; every Halftime sounded overdriven whatever its cut
(next section). Futurama's slight hiss was first blamed on the 20 dB
of drive; variants at 10 dB and none were built, no judgment
recorded.

### Where a module's overdrive comes from: the limiter's release — found 2026-11-13 to be the breadboard's stage; 256 bits a sample and 31 250 Hz made the defaults

**The limiter.** Halftime's mix matches openmpt123's render in bands
and peaks (58 % of its energy under 150 Hz; its drum samples are
recorded clipped). The knee of 3000 against a mix RMS of 18 238
limits always, by 19 dB, and at 16 ms rides the bass. The knee became
the clip's: header byte 29, times 1024, 0 for 3000 (`Shaper::new`,
`xm2clip.py --knee N`; SID and PT3 pass 0). A knee above the peak is
a plain gain: RMS 0.16 against 0.50.

**It was the stage.** Halftime stayed distorted with the limiter off
and as PCM of openmpt's render, though clean on the PC. A 2.2 k
resistor from base to emitter nearly cured it: the BC557 had three to
four times the base current it needed and no path for the charge, so
it turned off late and stretched every pulse. Everything heard of
Halftime on the breadboard before that is void.

**256 bits a sample.** PCM silence hissed at 128 bits and was quiet
at 256. `CLIP_PDM_BITS` (128, 256, 384; fp/fs two, four, six) is what
`clip::CONVERTER` carries; the user made 256 the default, 4.096 MHz
at 16 kHz. Heard before the resistor, so the hiss at 128 may have
been the stage's.

**31 250 Hz.** The user asked for 8 MHz with a sample rate to match;
whole dividers give 8 MHz exactly at 31 250 Hz and cannot make
32 000, hence `clip::FAST_HZ`. The rate is the clip's: a module's
byte 17 bit 2, a SID's byte 39 (then 2 steps a sample), a PT3's flags
bit 2, `PCM1`'s rate. A build at 384 bits refuses such a clip. Heard
on the breadboard and white-clock and kept: the three converters
default to it (`--rate 16000` otherwise), `clips/` was re-made;
`scripts/wav2pcm.sh` stays at 16 000 (31 KB a second otherwise).

Owed:
- a stock S8550 and buzzer at 256 bits a sample;
- the sixth build (Silence at 128 bits with the resistor, four
  limiter settings at a 60 Hz cut), built and not flashed: the
  limiter for a speaker is not settled;
- a scope on a stock board's collector with the 32 Ω speaker; if the
  S8550 saturates, 2.2 k base to emitter is a third part of the mod;
- the processor's share for a module at 31 250.

### PT3 tunes flat, for headphones — built 2026-11-13, heard on headphones and white-clock's buzzer

The player's shaping (noise at 11 sixteenths, 300 Hz high-pass,
limiter with 20 dB of drive) was for the buzzer; the user asked to
hear the chip flat. A clip opts out with flags bit 3, `pt3::FLAT`: the
mean is removed by a 5 Hz one-pole tracker in `pt3::Tune::next_sample`
(the shaper's Q14 high-pass cannot go under about 100 Hz at 31 250
samples a second), and the knee is `FLAT_KNEE` (96 in the shaper's
1024s, three channels at top level), so the header's drive is a plain
gain (127 unity, 255 twice). `PT31` had no spare byte for a knee.

Flat was 7 to 10 dB quieter than limited and, to the user, too bassy
(53 % of Synchronization's energy under 300 Hz against 9 %). After
A/B rounds the user found the 16 ms limiter hurt PT3 tunes and chose
flat with a 100 Hz high-pass once and drive 180 (1.4 times; RMS 0.21,
0.1 % of samples at the rails), on headphones and then on
white-clock's buzzer. That is `scripts/pt32clip.py`'s default;
`--limiter --highpass 300 --noise 11` gives the old clip byte for
byte. Only Synchronization was heard so. `CLIP_WIDE=1` with
`CLIP_PCM_DUMP` dumps the player's samples as `i16`. The buzzer's
choice was taken back on 2026-11-16 (the last section here).

### The station picks its access point: the strongest of each scan — built 2026-11-13, not on a clock yet

The bench board kept joining a far access point: the driver under
`ScanMethod::AllChannels` avoids the one that last dropped it. At the
user's request `net::wifi_task` now connects with the BSSID and
channel of `strongest_candidate` (`station_config(creds, Some(ap))`).
The unpinned config is set first (the scan runs under it), so a pin
never outlives its connect; a failed or empty scan leaves the driver
to choose. No second choice within a turn: a refusing access point
ends in the connect budget's reset.

Owed: on a clock, the log's `candidate` lines, `connecting to … at`
naming the loudest, `connected to` with that BSSID; with the near
access point off the next turn names the other.

### The page split in four: clock, settings, Home Assistant, system — built 2026-11-14, run on test-clock; white-clock, a phone and the access point owed

The user found the one page overloaded. Now `/` (alarm, display,
status), `/settings`, `/ha` (the broker) and `/system` (upload, token,
reboot, access point), each with the same line of links; on the
access point `/settings` and `/ha` are not served and `/system` has no
access-point button.

- A form post became a partial update: `checkboxes_absent_off` went,
  and each checkbox is preceded by a hidden input of its name with
  `value=0` (the later pair wins), or saving the alarm would clear 12
  hour on another page.
- A script posts a changed colour or brightness to `/api` at once;
  the clock page stays one form, Save under both fieldsets (the
  user's).
- `GET /update` and `GET /token` redirect to `/system`.
- `POST /reload` and `POST /ap` take `auth::token_or_pin`, since a
  page has no token; the PIN means someone who can see the panel.
- The token page is linked, the user's decision.

Run on test-clock 2026-11-14 by curl and headless Chromium (DevTools
protocol, real clicks). Owed: the Access point button, the pages on
the access point, the no-script Save, white-clock and a phone.

### Cross-site requests refused: `Host`, `Origin`, no framing — built and run on test-clock 2026-11-14

The user pointed out that any site could make a LAN browser post to
the clock (a form post needs no preflight), and that DNS rebinding
reached even the `Authorization` routes. The rule is `http::admit` in
`core`:

- `Host`: absent, the clock's IPv4 address, or on the station the
  hostname bare or with `.lan` or `.local` (the user's list, for now);
  on the access point the address only. Another `Host`: `GET` is a
  `302` to `http://<ip>/` (also the captive portal), `POST` a `403`.
- `Origin` on a `POST`: absent allowed (`curl`, the scripts), else
  `http://` and a name the `Host` rule accepts; `null` is refused.
- `X-Frame-Options: DENY` on every response.
- `GET /system` issues no PIN (an `<img>` could take a PIN in use
  away); the page's script posts `/pin`.

`parse_head` refuses a head with `Host` or `Origin` twice.
`Sec-Fetch-Site` is not used: browsers send it only to HTTPS and
localhost. A refused request does not count as served (it confirms no
new image) and is refused before the credentials, so the PIN stays
unspent. Run on test-clock: `scripts/verify-csrf.sh`, 26 checks, and
a foreign page in headless Chromium (posts, `no-cors` fetches, an
`<iframe>`, an `<img>`). Owed: the access point, where a phone's
sign-in probe naming another host now gets a `302`.

### Clips in a partition of their own: the clip pack — DONE, run on test-clock and verified by the user on white-clock 2026-11-14; the access point and the release workflow not yet run

The user asked for the clips in a partition carved out of the app
slots, an upload page, and the firmware still running on the old
table.

- **Table**: `ota_0` and `ota_1` 0x160000 each, `clips` at 0x2E0000,
  0x120000 (to the flash's end, grown from 1 MB the same day at the
  user's review, before any release), data subtype `undefined` (0x06),
  found by label. A custom subtype would panic every image that walks
  the table: esp-bootloader-esp-idf 0.6's `partition_type()` unwraps.
  An OTA leaves the old table (`ota_1` at 0x200000).
- **Pack** (`core/src/pack.rs`): `CLPK`, version 1, up to
  `pack::MAX_CLIPS` (16) entries of 64 bytes (offset, length, flags,
  API name, label), the data, SHA-256 at the end; `Pack::parse`
  checks all of it.
- **Reading**: the players read a clip in place, so `src/pack.rs`
  maps the partition into free flash-MMU entries at boot (the entry,
  then `Cache_Invalidate_Addr`, as esp-storage does), before the
  settings load, since a melody index is clamped to the table. The
  software SHA runs at about 1.4 MB/s (482 ms for 700 KB); the chip's
  accelerator was not tried.
- **Melody table at run time**: note melodies, then the pack's clips,
  else the built-in ones (`melody::install`, `count`, `LIMIT`;
  `RANDOM` a fixed value). A stored melody is an index, so another
  pack changes its meaning.
- **No clips built in by default** (the user's change): `ALARM_CLIP`
  is empty, a melody past the table reads as index 0, the beeps; the
  web installer writes the default pack as a fifth part, and the CI
  runs the host tests again with clips built in. An `.env` that
  exports `ALARM_CLIP` with Futurama makes an image too big for the
  slot.
- **Upload**: `POST /clips` (`Bearer` or `PIN`), page `/clips`.
  Credentials, partition, size and header are checked before anything
  is erased; then the pack leaves the table, the scan stops, sectors
  are written with read-back, the pack parsed in place. A refused pack
  has its first sector erased; success reboots (`heal::reboot`).
  `/api` has read-only `clips`: `pack` or `builtin`.
  `scripts/clips.sh`, `scripts/mkpack.py`; a host test parses the
  file `CLIP_PACK_FILE` names.

Run on test-clock (old table by OTA, new over USB, uploads of 10 to
53 s, every refusal, a flipped bit refused at boot as `Hash`); the
user uploaded the local pack from the page on white-clock
(`v0.4-34-gd24c81b`) and heard the clips. Owed: the access point, the
installer's fifth part, the release workflow.

### The thermometer held while Q1 is hot — measured on white-clock 2026-11-14, the hold built and run there 2026-11-15; a short preview and the serial log's lines not yet seen

With the buzzer mod (flyback diode, R5 5.1 k, 32 Ω coil) Q1, next to
the NTC, heats while a clip plays through the PDM converter; not on
beeps or a beeper clip. By reasoning, not scope: at 125 to 250 ns a
bit the transistor sits half on, about 0.2 W at worst (V²/4R), within
an S8550's rating; a 2.2 kΩ base-emitter resistor changed nothing.
Measured: Halftime for the alarm's five minutes raised `temp_c` from
24.1 to 33.3 °C, still rising at the end, back within 0.2 °C after 20
minutes.

`sensors::Hold` is told each second whether `clip::converting()`: the
published reading is held for four times what was played, two minutes
at least and twenty at most, plays adding up; the NTC's average runs
on underneath, and a reset loses the hold. Only the five-minute case
was measured: the reading stayed 24.1 until 1200 s after the ring
(`v0.4-36-g7b0e0c6-dirty`). Owed: the log's `held` and `released`
lines, a short preview's hold, a reset during a hold, whether a stock
board heats. Not explained: more sample-to-sample jitter (0.1 to 0.3)
than in the first run.

### The pages after a UX review: PIN at the click, a saved notice, the bad field named — built 2026-11-15, run on test-clock in headless Chromium; a real upload from the page run 2026-11-17

A review of the pages gave twenty findings; the user took these first.

1. Enter in the alarm form played the tune (Play was the first
   submit): the script makes Play and Stop `type=button`.
2. A stale tab put the old colour back: the script takes the names
   off the display's selects, so Save posts the alarm only.
3. A failed post looked done: now `The clock did not answer; nothing
   was changed.`, and the select goes back.
4. The PIN is drawn at the action's click on `/system` and `/clips`,
   not at the load (it expired before a file was chosen, and any
   answer disabled the buttons until a reload); a PIN form appears
   under the button and the buttons come back after any answer.
5. `Saved.` via `?saved=1` (`form::Note`).
6. The bad field named: `form::apply_naming`, `?error=value&key=tz`,
   the label from `form::FIELDS`, never the query's text.
7. `Show the password` on the setup form.

Run on test-clock (`v0.4-37-g61c15f6-dirty`). Not tried: an upload
from the page, the access-point button, a phone. The user's: upload
labels name the release's file, not `scripts/ota.sh`.

### The rest of the UX review — built 2026-11-15, run on test-clock in headless Chromium; the setup form the user's, 2026-11-15

The user asked for all the rest: a settings form posted by `fetch`,
so a refusal keeps what was typed; the token field as text; `/ha`
polling `/api` (5 s, two minutes) and reloading when `mqtt` changes;
`color-scheme:light dark`; titles `<hostname>: <page>`; seven day
checkboxes all named `days`, a repeated key read by `apply` as a union
(none ticked changes nothing), with the presets back as an unnamed
select at the user's request; a `Next alarm:` line; whether a broker
password is stored (password to none on the same host still needs the
host cleared first); `role=status` and `role=alert`; Rescan keeping
typed fields. Also the user's: no `Scriptable:` line on the pages (it
is the documentation's), one link colour (`a{color:LinkText}`), the
heading at 1.6 em, links, Save and notices centred, a warning that a
pack replaces all clips.

`RESPONSE_BUF` went from 6 to 8 KB (the user's figure): `PAGE_MAX`
8032, main stack 43.7 KB from 49.9; the clock page 7205 at its worst.
Run on test-clock (`v0.4-38-g368c53a-dirty`); the user reported the
setup form working on the access point. Not tried: a post with the
clock out of reach, a phone's width. Open: after one upload
test-clock was gone three and a half minutes and came back by a
self-heal, with no serial log; watch uploads with the monitor on.


### A clip pack for the speaker — DONE: the second pack (limiter's release 128 ms) heard and kept by the user 2026-11-15

The buzzer's clips are thin and compressed on a speaker. A first pack
with a plain gain and no limiter (RMS 0.14 to 0.25) was much too
quiet for the user. The kept pack is the buzzer's knee and drive with
the high-pass at 100 Hz once and the limiter's release at 128 ms
(`--release 10`; the buzzer's is 16 ms): RMS 0.31 to 0.37, 2 dB under
the buzzer's clips. Modules and Donkey Kong `--highpass 100 --passes 1
--release 10`; PT3 the same with `--limiter --noise 16`; Futurama
`CLIP_HIGHPASS=100 CLIP_DRIVE=15` (new variables of
`scripts/wav2pcm.sh`); Trantor as it is. `scripts/speaker-clips.sh`
makes `packs/clips-default-speaker.bin` under the default pack's names
and order; `packs/clips-local-speaker.bin` is made by hand. Left over:
`sid::FLAT` (flags bit 1, `sid2clip.py --flat`), used by no pack.

### Synchronization in the default pack — built 2026-11-16, heard by the user 2026-11-17

The user got n1k-o's permission for this project.
`mods/nq-synchronization.pt3` is the module and
`clips/synchronization.pt3c` the fourth and last clip of the default
pack, so stored melodies keep their tune; `clips/README.md` credits
him. Hibernation and PPK stay in `pt3/`: nobody was asked. Heard on a
modded buzzer and on a speaker, both good. Owed: a stock buzzer.

### Wi-Fi transmit power as a setting — DONE, verified by the user on the bench board 2026-11-15

The bench board's poor antenna needs less than 16 dBm; the user chose
a setting over a build variable. `Settings::tx_power_qdbm` is one of
`settings::TX_POWER_CHOICES` (2 to 16 dBm, the driver's levels),
capped by `config::WIFI_MAX_TX_POWER_QDBM` (68); record version 12;
the default 68 or the build's `WIFI_TX_POWER_QDBM`. Key `tx_power`, a
select on `/settings`, no token, not in `mqtt::SET_KEYS`.
`net::wifi_task` applies it before every connect and every
`RSSI_PERIOD`. The user's decision: the access point always transmits
at `config::AP_TX_POWER_QDBM` (11 dBm) and its setup form has the
select, so a clock set too low is recovered by the button hold.

### A SID tune on the other machine: `sid2clip.py --system` — DONE, heard by the user on test-clock's speaker 2026-11-15

Donkey Kong sounded slow to the user: its file says PAL (50.12 calls
a second) and the converter writes the machine's clock and frame
length into the header. `--system pal|ntsc` overrides the file; as
NTSC it is 19 % faster and two thirds of a semitone up, which the
user recognised. The local packs have the NTSC clip under the old
name; white-clock and martin-clock have PAL until a pack is uploaded.

### Beeper clips packed: `BPR3`, repeats as references into the clip; SID tunes the same, `SID2` — DONE, accepted by the user on a clock 2026-11-16

Trantor was 145 KB of the default pack. Its runs do not repeat bar
for bar (pulse trains of unrelated periods), and gzip or xz need a RAM
window; a Z80 snapshot of the player would be a few KB, but the user
chose packed runs as more generic. `BPR3` adds reference tokens: the
next n runs are those decoded from the token so many bytes back, in
the packed bytes, so the clip is read in place with only a stack of
`(return, left)`, `BEEPER_DEPTH` (16) deep; a reference may reach
itself. A bad file ends the clip. Trantor is 67 559 bytes, the pack
105 KB; Huffman over the unrepeated runs was not worth its table.
`clip::Beeper` still plays `BPR2`; `zxbeeper.py --plain` writes it.
Heard 2026-11-17: at its pitch, no stutter.

`SID2` is the same for a SID record (reference tokens `0x40`–`0x5F`
and `0x60`, an item a write or a wait; no reference crosses the loop
offset): Donkey Kong 62 216 → 11 742 bytes. The stack is
`core/src/refs.rs` (`Refs`), the packer `scripts/packrefs.py`; `SID1`
still plays. PT3 and XM modules are not packed: the player walks
pointers and would need the module in RAM. A firmware older than
these plays such a clip as noise.

### PCM clips at half the flash: IMA ADPCM, `ADP1` — built 2026-11-16, heard by the user on white-clock and a speaker; the buzzer through the clock's own decoder not yet

References do nothing for PCM (lz4 under 1 %, xz 6 %): the samples
are near noise. IMA ADPCM at 4 bits is 2:1 with the error about 18 dB
under the signal; the user found it audible but acceptable, and far
better than 8 kHz PCM at the same size, so it is for a clip that must
be smaller, not a replacement. `ADP1`: magic, rate as `u32`, then
sox's raw `ima` nibbles, high first (`tail -c +9 clip.adp | play -t
ima -r 16000 -c 1 -`). `core/src/adpcm.rs` decodes;
`scripts/adpcm.py` encodes, keeping four paths (+0.9 dB over the plain
coder; sox's own is 4 dB worse); `wav2pcm.sh` writes `.adp`. Futurama
is ADPCM in the local packs (221 109 bytes for 442 209). On
test-clock the decoder cost 0.6 % of the processor, no underruns. An
older firmware plays it as noise. Owed: heard on a buzzer.

### The PT3 clips' limiter: the buzzer's as the speaker's, the speaker's to be made gentler — built 2026-11-16, `--knee 37 --drive 150` chosen by the user on test-clock's speaker; heard 2026-11-17

On test-clock's speaker the user decided the buzzer's clip should not
be flat but what the speaker had, and the speaker's limiter gentler
(too noticeable in PPK). So `clips/synchronization.pt3c` is `--limiter
--noise 16 --highpass 100 --passes 1 --release 10 --restart`, while
`pt32clip.py`'s default stays the plain gain. Longer releases were
measured but not heard; instead `PT31`'s four spare flag bits became
a knee, `pt3::KNEES` (0 the shaper's own, each step 2 dB more;
`--knee N`), which an older firmware ignores, limiting hard. The user
chose `--knee 37 --drive 150` for the speaker (PPK's RMS 0.267 from
0.366); `scripts/speaker-clips.sh` and the local speaker pack use it.
Heard 2026-11-17: the default pack on a modded buzzer and on a
speaker, both good. Owed: a stock buzzer circuit.

### The project renamed: onebitdawn — 2026-11-17, verified in Home Assistant and on white-clock the same day

The user found `esp32clock` too generic; the name is the 1-bit buzzer
line and the morning. This file keeps the old name where it is
history.

- Crates `onebitdawn` and `onebitdawn-core`. `mqtt::PREFIX` is
  `onebitdawn`, so the topics moved; the device id and unique ids did
  not, and discovery gives Home Assistant the new topics. The old
  retained `status` and `state` stay on the broker. Not explicitly
  checked: that the entities kept their history and none is doubled.
- `config::AP_PASSWORD` defaults to `onebitdawn`; release files are
  `onebitdawn-*.bin`. GitHub redirects the old repository address but
  not the old Pages address.
- Pages: heading `onebitdawn: <hostname>`, title `<hostname>: <page>
  - onebitdawn` (hostname first, for a narrow tab).

### A clip of a version no player reads is refused — built 2026-11-17, tested on the host; no such upload tried on a clock

The fourth byte of every kind's magic is its version (`PCM1`, `ADP1`,
`BPR2`/`BPR3`, `XMC2`, `SID1`/`SID2`, `PT31`). Only `BPR1` and `XMC1`
were refused, by name; any other digit was played as headerless PCM,
noise. No format changed.

- `clip::is_unsupported`: the clip starts with a kind's three letters
  (`FAMILIES`) and no player takes it.
- `Pack::parse` refuses such a pack (`PackError::Clip`). On an upload
  that is after the write, like the length and hash checks, so the
  clock has the note melodies until the next upload: checking the
  clips before the erase would need the pack in RAM. The `ALARM_CLIP`
  assert fails the build.
- Not done: a `PCM1` clip at an unknown rate still plays at 8 kHz;
  unknown flag bits in a tune's header are ignored.

### Forward compatibility before the first public release — built 2026-11-17, tested on the host and on test-clock

Asked by the user before the release after v0.4, the first others may
flash. What cannot be fixed later is what *this* firmware does with a
later one's data after a rollback. An audit found three faults and
one decision.

- **An unknown value cost the whole record.** `Settings::from_record`
  answered `None` for any field out of range, leaving a rolled-back
  clock on defaults with no network. Now each field falls back alone;
  `None` only for a record shorter than its version says.
- **A longer record erased the settings.** `storage::load` erased the
  map on any error, `BufferTooSmall` among them. The scratch is 1024
  bytes and `storage::ITEM_MAX` (1000) is the promise. Leaving a
  too-long item alone was not enough: `sequential-storage` reads
  every item it passes, so with a 1086-byte item present a record
  saved beside it was not read at the next boot. An image that meets
  one runs on defaults (with the build's seeds) and leaves it until
  it has something to save, then erases the map first.
- **A carry layout change dropped the boot count**, so such an update
  could not be rolled back. From layout 4 (v0.4's too) the record is
  append-only and `carry::readable` takes a later layout by the words
  it knows.
- **The prebuilt token.** Release images had a token printed in the
  manual, and `seed` stores it at the first save for good. The user
  decided to ship without one: the panel PIN does everything from the
  pages; scripts work once a token is set on `/system`.
- Not changed: the partition table (520 KB of slot to grow into), the
  pack format, the OTA checks. `AGENTS.md` got "Compatibility".

On test-clock, throwaway builds stood in for a later firmware (record
version 13 with unknown values, carry layout 5, records of 426 and
1086 bytes): each case behaved as designed, and the token was cleared
and set again with the PIN. Main stack 42.2 KB.

Not tested: a clock that never had a token on a tokenless build; the
access point's setup on such an image; the release workflow itself.
Still owed from before: the default pack on a stock buzzer circuit.

### AGENTS.md cut to the rules, the references moved to `agents/` — 2026-11-17

`AGENTS.md` had grown to 90 KB loaded into every session. It is 24 KB
of rules; the rest moved to `agents/behaviour.md`, `modules.md`,
`clips.md`, `hardware.md` and `sizes.md`. The move was made verbatim
and checked by script, then restructured with every backticked name
kept. New in `AGENTS.md`: "Reference files", "Behaviour rules", a
one-line-per-file layout table.

## The display saved with Save — built and verified 2026-11-17

The user found colour and brightness applying as chosen, on a page
where everything else waits for Save, not obvious; this takes back
"The display is applied as chosen" of the page split. `CLOCK_JS` no
longer posts a changed select to `/api`; the two are ordinary form
fields counted by the unsaved mark. Cost: a stale tab's Save puts
back what was stepped on the clock since (removed by the next
section). Verified by the user on white-clock.

## A form posts only what was changed — built and verified 2026-11-17

Asked by the user to remove the stale-tab cost. In `UNSAVED_JS`
(shared by `/`, `/settings`, `/ha`) `G` collects the names of
controls that differ from what was served; the mark is `G().size` and
the submit posts only those names' pairs. A name goes with every pair
under it: a checkbox with its hidden `0`, the days all that are
ticked. Nothing changed posts an empty body, answered `Saved`. The
server is unchanged; the setup form has no such script. Measured on
test-clock with headless Chromium: only changed fields posted, and a
stale tab left another's colour alone.

A review finding: on a failed `fetch` the script fell back to
`f.submit()`, the whole form. The fallback is gone: `SAVE` shows `The
clock did not answer; try again.` and the form stays as typed.

Not seen: `/ha` with a field changed. Verified by the user on
white-clock (`v0.4-79-gc258300`).

## A clip at half its pitch: the transmitter's sample width — fixed and verified 2026-11-17

Report: now and then a clip played at half pitch and stuttered, and
another clip did not clear it. On test-clock it began and ended
straight after a preview of Trantor, the one beeper clip, which ran
the transmitter in TDM mode at 8 bits.

Measured: a slow clip is read at exactly half the rate with every
register as in a good one; it followed about one Trantor in sixteen
(59 of some 1000 rounds) and stayed until the next. Not the divider;
`tx_update` after start, resets at stop, waiting for `tx_idle` and
rewriting the width changed nothing. Of thirteen TDM values forced
into a converter clip only `tx_bits_mod` at 7 (8-bit samples) gave
the half rate, every time. So the transmitter, once run at 8 bits,
sometimes keeps taking one byte a sample in a state no register shows
and no reset reaches.

**The fix** is never to run at 8 bits: a beeper clip goes out as two
16-bit TDM channels at the same bit rate, and `play` swaps each byte
pair in the ring by hand (not the driver's big-endian setting, never
seen on a scope). 0 strays in 440 rounds; the user heard Trantor at
its pitch.

## GPIO5's drive strength: tried as a setting and dropped — 2026-11-17

At the user's request `fun_drv` was built as a setting `buzzer_drive`
(5 to 40 mA, record version 13) and run on white-clock at 40. The
user saw no difference on the scope or by ear. Dropped whole, record
back at 12. Reasoned, not measured: the open-drain pad sinks only
Q1's base current through R5, under 1 mA.

For a second try: esp-hal 1.2.2's LEDC `Channel::configure` puts the
pad back to 20 mA for every tone (the trial rewrote `fun_drv` through
`IO_MUX::regs().gpio(5)`); the strength was never read back.

## A 16 kHz clip pack: tried and dropped — 2026-11-17

The user's idea: modules at 16 kHz halve the converter's bit rate (8
Mbit/s to 4.1) and might keep Q1 cooler. Built as
`scripts/clips-16k.sh` and a second release pack. The user's result:
the transistor is about as hot, and on the speaker it sounds bad.
Dropped whole. Reasoned, not measured: Q1 does not switch at either
rate and sits in its linear region.

## Trantor's stray positions: the I2S clock's fraction — reproduced, fixed and verified 2026-11-18

The soak's Trantor counted about 1890 stray positions per 30 s
preview, except the first after boot. On `another-bench` the count
grew with play time (0 at 10 s, 5589 at 60 s): a drift of one chunk
(16 ms) in about 9.7 s, 0.165 %. The beeper's 512 kbit/s needs an
MCLK of 160 MHz by 39 1/16, and the sixteenth is 0.16 % of the
divider: on every play but the first after a reset the fraction is
not what the registers say. Which way the line was off was not
measured.

The IDF writes other coefficients first ("workaround for the double
division issue", `i2s_ll_tx_set_raw_clk_div`, v6.0.1): whole part 2,
`yn1` 0, y 1, z 0, x 0, then the target's `yn1`, z, y, x and the
whole part last, a field a write. esp-hal 1.2.2 does this on the PCR
chips only. `settle_clock` in `src/clip.rs` does it for every clip
before the transfer starts. On `another-bench`: 0 strays for Trantor
in any order and rung five minutes as the alarm (the soak: 31329);
the converter clips' filling unchanged.

A stopgap: esp-hal `main` (d6caef0, `i2s/clocks/v2v3.rs`) writes the
sequence for the C3, so `settle_clock` goes with a release that has
it. Heard by the user on white-clock. Not tried: whether the 8-bit
half-rate fault survives this sequence.

## Sixteen channels for the module player — built, measured on test-clock, heard and kept 2026-11-18

For the user's S3M "Children - Classic Version" (16 channels;
somebody else's music, local packs only, kept in the ignored `xm/`).

- `scripts/s3m2xm.py` (later the pack maker's `s3mxm`) makes an XM:
  effects A, B, C, D, O, T converted, others named and left out, no
  panning. Matched libopenmpt's playing of the S3M.
- `xm::MAX_CHANNELS` 8 to 16; still `XMC2`, existing clips byte for
  byte the same. An older firmware plays a clip of more than eight
  channels as nonsense; none is released. Stack 39.9 to 39.3 KB.

On test-clock: Halftime (four channels, 31 250) 22 % of the
processor; Children at 31 250 49 %, no underruns or strays, `/api`
answering (worst 360 ms), the watchdog's longest feed gap unchanged.
The user kept the 31 250 variant over a 16 kHz one. `packs/local.sh`
makes the local packs.

Not done: the clip as an alarm ringing five minutes at half the
processor; the version line lacking `-dirty` for a changed tree.

## A pack maker for users: one command and one web page — built and verified by the user on white-clock 2026-11-19; the release workflow not run

Before the project is announced the user wanted one command taking
`.pt3`, `.sid`, `.wav`, `.mod`, `.xm`, `.s3m` and a web page doing
the same, in place of nine scripts.

- `tools/pack/`, `onebitdawn-pack`: a `std` library over `core` (the
  converters ported from Python, the writer; constants are `core`'s)
  and a binary. Every pack goes through `Pack::parse` and the
  partition check. Aliases `cargo pack`, `pack-test`, `pack-wasm`.
- `onebitdawn-pack -o mine.bin [--sound-output buzzer|speaker]
  file[:option,…][=Label] …`. Kind by magic, then extension; a ready
  clip goes in as it is. Also `name=`, `--clip`, `flat`.
- `.wav` without sox: RIFF reader, mono mix, windowed-sinc resampler,
  `core::shape`, ADPCM at 16 kHz by default. Not sox's bytes; rms
  within 0.2 %.
- The page, `docs/pack/`: the library as a `cdylib` for
  `wasm32-unknown-unknown` with a hand-written C ABI and plain
  JavaScript, no wasm-bindgen or npm; `core` builds for wasm as it
  is; 298 KB. The preview plays through the clock's own players (not
  `clip::Stream`, whose rest is the lower rail and clicks). It cannot
  upload: https cannot post to `http://<ip>` and `http::admit`
  refuses a foreign `Origin`. Options sit behind an "Advanced" box
  after the user's first runs.
- Releases carry the command for Linux, Windows and macOS.

**Decided by the user.** The Python converters went once the Rust
gave the same bytes; `zxbeeper.py` (with `packrefs.py`) stays, as
`.tap` needs a Z80 and SkoolKit; mp3, ogg, flac through the browser's
decoder later.

**Measured.** Byte for byte the Python's output for both packs and
every converter option; the 6502 against the Python one over 4000
random programs; the page's pack identical to the command's.

**Learned.** A speaker's defaults for a SID tune (a module's limiter)
clipped on white-clock; by the user's ear they are `flat`
(`sid::FLAT`) with `highpass=150,passes=2`.

Verified by the user on white-clock, speaker and buzzer. Owed: the
release workflow has not run (`PLAN.md`, "Checks still owed").

## The alarm sound is kept by its name, and an older record written again at the first boot — built 2026-11-19, run on test-clock, verified by the user on white-clock the same day

The record kept the alarm sound as an index into the melody table, so
a new pack could silently change the alarm. The user wanted it kept
by API name.

- Record version 14 appends the name (a length byte and
  `pack::NAME_MAX` bytes; `RECORD_MELODY_NAME`), 408 bytes.
  `to_record` writes length 0 for an index with no melody in the
  table in force (the pack is out of the table during an upload).
- `from_record` with a name: that melody wherever it is; not found is
  the beeps. No name: the index. Byte 35 is still written for a
  rolled-back image. `Settings::melody` stays an index.
- At the user's word, `settings::record_is_older`: `Store::load`
  notes it and the persist task saves once as it starts, so the name
  is there before a first pack upload. One flash write at that boot.

Run on test-clock with packs in three orders and a rollback to
`v0.4-105`. Not covered: once saved with the beeps, a clip that comes
back later stays the beeps. Verified by the user on white-clock
(`v0.4-109-gb651296`).

## "Suspended Seconds", a module of the user's own piano tune — built 2026-11-19, in the default packs, verified on a buzzer and a speaker the same day

**The licence.** The tune, its MIDI file, the module and the clip are
the user's copyright, not under the project's licence. The final
terms (widened twice the same day from no distribution apart from
the clip pack): may be copied and distributed, unmodified or as converted
from its module by onebitdawn's tools, as part of onebitdawn clip
packs, including commercially; all other rights reserved. In
`clips/README.md`, `AGENTS.md`, `agents/clips.md`, the script's
docstring.

**Built**, `mods/src/suspended_seconds.py` and `.mid` over
`scripts/mkxm.py`; nothing in `xm.rs` changed. One piano track, 48 s,
132 notes, looping.

- `mkxm.read_midi(pedal=True)`: a note released under the pedal lasts
  until the pedal comes up or its key is struck again;
  `grid=(0.5, 0.125)` quantizes (the user asked for it).
- A struck string: `Sample(struck=)`, `Instrument(damp=)`: the start
  of a recording and a loop, the volume envelope the decay with no
  sustain point, key-off the damper (125 ms). Also `Sample(onset=)`,
  `Sample(bright=)`.
- Velocity is volume, `64 (v / 93)^1.5`. Twelve ticks a row, tempo
  240; ten channels, no note cut.

**The piano, by the user's ear.** Seven kits were tried (MuseScore
General's grand, Iowa's Steinway B, VSCO 2 CE's upright, with
lifts). Short loops after half a second sounded like an organ;
recordings are now kept 0.7 to 1.5 s before the loop. The dullness is
the recordings', not the sample rates. Chosen: VSCO's upright at mf,
no lift (CC0); the others are a command away (`scripts/mkxm.py
mods/src/suspended_seconds.py out.xm <kit>`).

**The clicks were the limiter**, which flattens a piano's attack. One
that releases over seconds rides the passage and leaves attacks
alone. Chosen by the user: `knee=12,release=13` for the speaker
(-15.5 dBFS rms, no steps) and `knee=12,release=10` for the buzzer.

**In the default packs.** `mods/suspended-seconds.xm`;
`clips/suspended_seconds.xmc` (256 361 bytes) the buzzer's, eighth
clip; `scripts/speaker-clips.sh` makes the speaker's. The pack is
727 891 bytes, 61 % of the partition. The clip needs the player's
volume ramp, which `v0.4-109` lacked. Heard by the user on
white-clock (speaker) and martin-clock (buzzer). Not measured: the
processor's share on a clock.

## "Starlight & Aurora", a module of the user's second tune — built 2026-11-20, verified on white-clock's speaker; the speaker's pack's ninth clip only

A clip and nothing in the firmware, so on `master` in the freeze. The
user's tune, under the terms of "Suspended Seconds" by his word.

**Built**, `mods/src/starlight_aurora.py` and `.mid`,
`mods/starlight-aurora.xm` (225 687 bytes). Four parts, 111 s,
looping: piano, a bell pad, a sine lead, a sine bass (F2 to A2).

- `scripts/surgenotes.py` renders single notes of a Surge XT patch
  offline: `pedalboard` hosts the VST3 and the patch goes in as the
  plugin's state (an `.fxp` is a 60-byte header and Surge's `sub3`
  chunk). The first note after a patch is loaded is not the patch's.
  Patch: `patches_3rdparty/LinnStrument MPE/Keys/Acoustic Grand.fxp`.
  Renders go in the ignored `xm/`.
- Piano: two velocity layers split at 66, at 11 025 Hz (the stem has
  nothing over 2 kHz). Bell: synthesised from measured partials, its
  room a 2.5 s release; a held bell falls 4 dB a second (`FADE`), at
  the user's request. The bass has harmonics to the fifth, moved into
  the one tune by the user. Nineteen channels. The user dropped a
  quantized kit.
- An instrument's `volume=` does nothing where every note has a
  volume column.

**Where it ended.** The user found it does not sound on a buzzer: the
buzzer's pack is the eight again, and `scripts/speaker-clips.sh`
makes it as the speaker pack's ninth and last clip (949 750 bytes,
80 % of the partition). Speaker settings chosen by the user on
white-clock: high-pass 100 Hz once, `knee=12,release=13`;
`highpass=60` was not taken, the bass being next to unheard there and
left so. The user read the credits in `clips/README.md` and found
them right.

**The label.** The module's title is "Starlight and Aurora" (twenty
characters, an XM title's limit; the name `starlight_and_aurora` is
`NAME_MAX`). A pack label may not hold `&` (`check_names`), and the
pack maker makes "Starlight Aurora" of the title. Open: what the
clock's pages should show is the user's to say.

## The pack maker's page asks which firmware the pack is for — built 2026-11-24 on `next`, seen by the user in a browser the same day

A feature for after v0.5, asked by the user: a choice of v0.5 or
"v0.6 or later", with clips held to what that firmware plays.

- `tools/pack`'s `Firmware` (`V0_5`, `V0_6`, default the latest) and
  `convert_for(source, output, firmware)`.
- For v0.5: a module's 16-bit samples cut to 8 (`XMC2`), a sound file
  as `ADP1`. An option asking for more (`samples=16`,
  `samples=packed`, `format=packed`, `format=pcm16`) is refused with
  the reason, and so is a ready clip v0.5 does not play (`XMC3`,
  `XMC4`, `ADP2`, `PCM2`), by `Firmware::plays`.
- `obd_convert` takes the minor version (5; anything else the
  latest). The page has a drop-down, "The clock's firmware is"; the
  command `--firmware 0.5|0.6`.

## The software modulators taken out — 2026-11-24 on `next`, verified on white-clock with the next section

The user's decision: nobody will use them with the PDM converter
there, which every build has used since 2026-11-13.

Gone: `Player`'s two modulators and their tests, `BIT_RATE`, `OSR`,
`CARRIER_HZ`, `config::CLIP_KBPS`, `CLIP_CARRIER_KHZ`, `CLIP_PDM`,
`CLIP_SIGMA_DELTA`, and the build variables `CLIP_MODULATOR`,
`CLIP_KBPS`, `CLIP_CARRIER_KHZ` (now ignored). `clip::CONVERTER` is
`CONVERTER_BITS`; `Player::fill` is what `fill_samples` was.
`CLIP_PDM_BITS` and the TDM mode for a beeper clip's bits stay. A
default build plays the same. `.rwtext` 0x223c, `.stack` 0x8038.

## Beeps and beeper clips on the line output through the converter, at a level — built 2026-11-24 on `next`, verified by the user on white-clock the same day

The user asked to limit how loud the note melodies and Trantor are on
the speaker: on GPIO20 both were the pin rail to rail, while the
amplifier pack's modules are near 16 dB under full scale. The user
chose the PDM converter with a ceiling over thinning the bits.

- `clip::Mean` makes samples at `FAST_HZ` from a square wave given as
  levels and ticks of 1/512 000 s: each sample is the wave's mean
  over its 16.384 ticks, counted in 125ths so it is exact, times an
  amplitude. Pulse widths and pitch are kept.
- A beeper clip for the line is `Beeper` through `Mean`; a melody is
  `clip::Notes` through `Mean`, a plain square wave about the middle.
  `Stream::of(sound, line)`. The clip task plays notes too
  (`play_notes`), handed over by `buzzer::play_once` when the tones'
  pin is the line. On the buzzer nothing changed.
- The setting, by the user's word, is `beep_volume`, "Beeps volume on
  amplifier", `settings::BeepVolume`: `quietest`, `quiet`, `loud`,
  `loudest` at 12, 20, 32, 50 hundredths of a module's full scale,
  default `quiet` (`clip::LINE_PCT`). Record version 15 appends its
  byte (409 bytes). A select, disabled while the output is `buzzer`;
  in `/api`; not a `set/` key or an entity.
- The ceiling of 50 is the user's rule: no louder than a PT3 tune's
  three channels at top level with no limiter. A `const` assert holds
  `LINE_PCT_MAX`.
- With the clips under suspicion notes are LEDC's on the line too.
  The user pointed out a square wave there could damage the speaker,
  so it is a pulse, `clip::line_duty`: the longest, in 1024 steps,
  whose power (d (1 - d)) is no more than the setting's square wave:
  2, 7, 19, 48 steps.

Sizes: `.rwtext` 0x227c, `.stack` 0x7fc8; settings page 4804 of 8032
bytes, its form's body 294 of 640.

On white-clock: the fallback run with `--features bench-hang` (`POST
/hang/clip`), then the plain build at `line`; the user found all
good. Not recorded: whether a plain mean is enough for Trantor (it
leaves 15.6 to 31 kHz folded into the band), and the clip task's
processor share for Trantor through `Mean`.
