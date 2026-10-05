<img width="971" height="539" alt="Screenshot 2026-10-04 at 7 08 56 PM" src="https://github.com/user-attachments/assets/8a58cf5a-001f-4120-af2c-a90c73a7bf24" />
# CLERGY

A fast, disciplined terminal utility for system hygiene, live metrics, and controlled purge operations.

CLERGY is a Rust-based TUI designed to feel *instrumental*, not decorative — inspired by tools like `btop`, but focused on clarity, safety, and intent.

---

## Overview

CLERGY provides:

- Live system metrics (CPU, memory, disk, swap, load)
- Compact grid-based telemetry with historical context
- Safe, explicit purge operations with confirmation
- Persistent last-purge results
- Configurable theming with terminal-first defaults

No background daemons.  
No telemetry.  
No gimmicks.

---

## Design Philosophy

CLERGY follows a few strict principles:

- **Deterministic** — no hidden behavior
- **Readable at a glance** — dense, meaningful visuals
- **Terminal-native** — respects your terminal colors by default
- **Explicit actions** — nothing destructive without confirmation
- **Minimal state** — only what matters, nothing more

If something is shown, it exists for a reason.

---

## Interface

- Grid-based metrics inspired by `btop`
- Braille-style intensity dots for compact history
- Pressure-aware coloring (normal / warning / danger)
- Clear separation between telemetry and actions
- Zero UI noise

The interface refreshes smoothly without flicker and avoids unnecessary animation.

---

## Theming

CLERGY supports theming via a small, explicit palette system.

### Default
- Red accent for identity
- Cyan/blue for information
- Amber for attention
- Green for success
- Gray for silence

### Mellow theme
A blue / cyan / turquoise-forward palette designed for long sessions and low eye fatigue.

Themes are defined in code and can later be made user-configurable via `settings.rs`.

---

## Installation

### Homebrew (recommended)

```bash
brew trust --formula joshfisidi/tap/clergy
brew tap joshfisidi/tap
brew install clergy
```

Run `clergy` to get started.

### Build the latest source

On macOS (Apple Silicon or Intel), install Rust and the backend's JSON encoder:

```bash
brew install rust jq
git clone https://github.com/joshfisidi/clergy.git
cd clergy
cargo build --release --locked
./target/release/clergy
```

Homebrew installs the version selected by the tap; pushing a source fix to `main`
does not update an existing Homebrew installation.

## Purge authentication

Opening the dashboard does not require administrator authentication. Select
**Purge**, then **Confirm**. CLERGY temporarily returns to the normal terminal for
sudo's password prompt. Password input stays hidden and is handled by sudo.
Press Ctrl+C at that prompt to cancel and return to CLERGY without starting a purge.

After authentication, the dashboard resumes and shows the running operation while
metrics continue to refresh. Wait for the result before starting another purge or
quitting. Authentication and startup failures return to an error panel. The
backend never opens another password prompt inside the TUI.

## Run reports

Every purge produces a report with:

- Start/end timestamps, host, user, and total duration.
- Each command's status, exit code, milliseconds, and captured output.
- Before/after disk availability, RAM pages, free/compressed RAM, and used/free swap.
- Exact disk availability in KiB, plus a readable signed change.
- Before/after Time Machine snapshot inventories and names that disappeared.

If a command fails, the remaining actions are skipped and the partial report is
retained. The latest report can be reopened under **Last Session**, including runs
from the command line. Older saved reports remain readable; unavailable command
timings and snapshot counts are shown as unknown.

Use **↑/↓**, **PgUp/PgDn**, or **Home/End** to scroll long reports. Smaller terminals
use a compact layout. The telemetry's **Purge** tile shows the latest run time.

Measurements describe changes observed during the run. They include other system
activity and do not prove bytes reclaimed by a specific action. A successful cache
flush does not provide a cache-entry count, and successful snapshot thinning may
remove no snapshots. Reports distinguish these outcomes.

`clergy purge --json` includes the same journal and observations. Partial runs
produce structured JSON and a nonzero exit status; `--headless` prints the readable
report. Saving failures are shown rather than silently discarding the report.

For command-line use:

```bash
clergy purge --headless
clergy purge --json
```

Without terminal input, authentication is noninteractive and fails if cached sudo
credentials are unavailable. Authenticate with `sudo -v` in that terminal first.

## Regression tests

```bash
cargo test --locked
cargo build --locked
python3 tests/purge_terminal.py
```

The macOS pseudo-terminal tests replace sudo and all purge actions with mocks.
They check hidden password input, cancellation and failure recovery, continuing UI
updates, duplicate-purge prevention, JSON output, and 16 KiB/4 KiB memory pages.
They do not authenticate with real credentials or change system caches/snapshots.

