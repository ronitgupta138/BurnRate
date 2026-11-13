# Terminal ANSI HUD Render Loop & Rate Limiting Specification

## 1. Interactive HUD Architecture
BurnRate renders a persistent status HUD at the top or bottom of the developer's terminal, displaying real-time session spend, token velocity, and burn-rate meters.

## 2. Refresh Rate Limiting
- **FPS Ceiling:** Clamped to `15 Hz` (66ms minimum frame delta) to prevent terminal emulator buffer saturation and high CPU usage.
- **Double Buffering:** Compares the upcoming rendered ANSI frame buffer against the active frame. Only mutated lines are re-emitted via ANSI cursor positioning codes (`\x1b[{row};{col}H`).

## 3. Alternate Screen Buffer Support
When full terminal redirection is enabled, BurnRate switches to the xterm alternate screen buffer (`\x1b[?1049h`), cleanly restoring user shell scrollback history on exit (`\x1b[?1049l`).
