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
brew trust --formula joshfisidi/tap/clergy` or `brew trust joshfisidi/tap` to trust it.
brew tap joshfisidi/tap
brew install clergy
```

Run ```clergy``` to get started.



