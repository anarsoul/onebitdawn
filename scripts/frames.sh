#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Vasily Khoruzhick
# The functions of the built firmware whose stack frame is a kilobyte
# or more, with the frame's bytes, largest first (agents/sizes.md).
# The Wi-Fi blob's and the main task's are there by nature; a
# function of the clip's start (`clip::Stream`, `clip::Player`) is a
# value built on the stack and moved, and is to be gone.
# Needs llvm-objdump. Usage: scripts/frames.sh [elf]
set -eu

cd "$(dirname "$0")/.."
elf=${1:-target/riscv32imc-unknown-none-elf/release/onebitdawn}
llvm-objdump -d --no-show-raw-insn -C "$elf" | awk '
    function hex(s) { return strtonum(s) }
    /^[0-9a-f]+ <.*>:$/ { if (frame >= 1024) print frame, name; name = $0; sub(/^[0-9a-f]+ </, "", name); sub(/>:$/, "", name); frame = 0; big = 0; next }
    $2 == "addi" && $3 == "sp," && $4 == "sp," && $5 ~ /^-/ { frame += hex(substr($5, 2)); next }
    # A frame too large for an addi: lui and addi into a register,
    # then sub sp, sp, it.
    $2 == "lui" { reg = $3; big = hex($4) * 4096; next }
    $2 == "addi" && $3 == reg && $4 == reg { big += ($5 ~ /^-/) ? -hex(substr($5, 2)) : hex($5); next }
    # (Not the Wi-Fi blob, which takes an address off the stack pointer.)
    $2 == "sub" && $3 == "sp," && $4 == "sp," && $5 "," == reg && big < 1048576 { frame += big; next }
    END { if (frame >= 1024) print frame, name }
' | sort -rn
