# AOVR

<div align="center">

![AOVR demo](assets/demo.png)

**Terminal User Interface (TUI) for interactively managing Android Runtime Overlays.**

[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![UI Powered by Ratatui](https://img.shields.io/badge/UI-Ratatui-170126.svg?style=flat-square)](https://github.com/ratatui/ratatui)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](./LICENSE)

</div>

## Overview

**AOVR** brings an intuitive keyboard-driven terminal interface to Android's native cmd overlay system. Instead of manually parsing messy text streams from `cmd overlay list` and copying package names to run enable/disable commands, **AOVR** categorizes, groups, and lets you toggle runtime overlays seamlessly.

## Features

- **Electric Violet Palette:** Modern, borderless dark-mode panel layout powered by Ratatui.
- **Grouped Target View:** View all Android target packages alongside active overlay badge counts.
- **Interactive Toggle:** Enable or disable overlays instantly using `Space` and apply changes with `Enter`.
- **Vim & Arrow Navigation:** Native support for both `j`/`k` and `↑`/`↓` directional inputs.
- **Smart Search Bar:** Filter targets and overlays with smart-case matching (`/` or click the bar, `Enter` applies, `Esc` or `×` clears).
- **Fault-Tolerant:** Binder failures from `cmd` (e.g. `Failed transaction (2147483646)`) are surfaced as readable status messages — refresh the list with `r` instead of restarting.

## How It Works

Under the hood, **AOVR** wraps Android's native Overlay Manager Service (OMS) commands via `su -c`:

- **Listing Overlays:** Executes `su -c cmd overlay list` to parse targets, active states, and broken overlays.
- **Enabling Overlays:** Invokes `su -c cmd overlay enable <package>`.
- **Disabling Overlays:** Invokes `su -c cmd overlay disable <package>`.

It runs directly inside an Android terminal environment such as **Termux** with root access.

## Keyboard Reference

| Key | Action |
| :--- | :--- |
| `↑` / `↓` or `j` / `k` | Move cursor (Targets / Detail / About) |
| `g` / `G` or `Home` / `End` | Jump to first / last item |
| `PgUp` / `PgDn` or `Ctrl-u` / `Ctrl-d` | Page / half-page jump |
| `h` / `←` / `Esc` | Clear search, go back (Detail → Targets, dismiss About) |
| `l` / `→` / `Enter` | Open detail / apply changes and go back |
| `Space` | Open Targets / toggle overlay state in Detail |
| `/` | Smart-case search over target and overlay names (`Enter` applies, `Esc` clears; top bar is clickable) |
| Mouse wheel | Scroll / move selection (confirms search when matches exist) |
| Click | Focus search bar, clear filter with `×` |
| `r` | Refresh the overlay list (`cmd overlay list`) |
| `a` | Open the About / Keyboard Reference screen |
| `q` | Quit AOVR |

## Installation
### One-Line Quick Install (Termux / Android)

Run the following command in Termux or an ADB shell:

```bash
curl -fsSL https://raw.githubusercontent.com/sohan-f/aovr/master/install.sh | sh
```
## Building
### Prerequisites

1. **Rust Toolchain**: Latest stable Rust (`rustc >= 1.98`), installed via [rustup](https://rustup.rs).
2. **Android Environment** *(runtime only — not needed to build)*:
    - **Termux** installed on a rooted Android device.
    - Root access available via `su` (e.g. Magisk).

> **Note:** No Android NDK is required locally. The project builds and tests
> natively on any host (Linux/macOS); the Android (`aarch64-linux-android`)
> release binary is cross-compiled in CI, which is where the NDK is used.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sohan-f/aovr.git
cd aovr

# Build release binary (host triple, e.g. x86_64/aarch64-linux-gnu)
cargo build --release

# Run the test suite (parser & shell tests use fixtures recorded
# from a real Android device in tests/fixtures/)
cargo test

# The compiled binary will be available at target/release/aovr
```

### Virtual Mode (non-Android hosts)

On any host that is **not** Android, AOVR automatically starts in *virtual
mode*: it replays a bundled `cmd overlay list` capture from a real device
(31 targets / 126 overlays) so the UI can be reviewed and toggled manually
without hardware. A `virtual ·` badge in the breadcrumb marks this state, and
toggles update the in-memory list only — no device is ever touched.

```bash
cargo run --release                                # bundled real-device capture
AOVR_TARGETS_FILE=my_capture.txt cargo run --release   # replay your own capture
```

| Variable | Effect |
| :--- | :--- |
| `AOVR_TARGETS_FILE` | Replay any recorded `cmd overlay list` output (highest priority) |
| `AOVR_SU` | Force the real `su -c` path even off-device (e.g. a wrapper script for failure testing) |

Priority: `AOVR_TARGETS_FILE` → `AOVR_SU` → platform default
(Android: `/system/bin/su`, otherwise the bundled virtual list).

## Usage
### If installed via installation script
```bash
aovr
```

### Running inside Termux (On-Device)

```bash
# Move binary to path and run
cp target/release/aovr $PREFIX/bin/aovr
aovr
```

Root access is handled automatically; AOVR invokes `su -c` internally when executing overlay commands.

## Design Palette

AOVR uses a custom **Electric Violet** color system:

| Token | Preview | Purpose |
| :--- | :--- | :--- |
| `ACCENT` | ![#9D7AF0](https://img.shields.io/badge/-%239D7AF0-9D7AF0?style=flat-square) | Primary highlights & cursor focus bar |
| `PANEL_BG` | ![#161223](https://img.shields.io/badge/-%23161223-161223?style=flat-square) | Deep solid panel background |
| `SEL_BG` | ![#30205F](https://img.shields.io/badge/-%2330205F-30205F?style=flat-square) | Active row selection fill |
| `C_OK` | ![#50DC96](https://img.shields.io/badge/-%2350DC96-50DC96?style=flat-square) | Enabled status indicator |
| `C_WARN` | ![#F0B43C](https://img.shields.io/badge/-%23F0B43C-F0B43C?style=flat-square) | Disabled status / Warning |
| `C_ERR` | ![#F05A5A](https://img.shields.io/badge/-%23F05A5A-F05A5A?style=flat-square) | Broken / Invalid overlay package |

## License

This project is licensed under the [MIT License](LICENSE).
