# What homelab's TUI draws today, and what it hand-rolls

Kenny, 2026-09-17: *"Misschien moeten we pas componenten maken nadat we ze
nodig hebben. Kijk eens naar Homelab Rust en wat er daar in de TUI gebruikt
wordt, dat wordt voorlopig onze eerste consument."* So this is the
inventory that decides what kp-tui builds next. Measured in
`~/Projects/homelab` (workspace 3.51.0, branch `main`) on 2026-09-17.

## The stack

`ratatui 0.30.2` + `crossterm 0.28` with `event-stream`. The TUI is the
`homelab` binary (crate `homelab-client`), subcommand `homelab tui`;
`homelab-host` is the daemon and has none. `tui-preview` is a second app in
the same workspace: the cyberpunk mockup on simulated data, 4,499 lines.

`client/src/tui/` is **4,942 lines**, of which the view layer is 2,106
(`view/mod.rs` 861, `model.rs` 1,899, `backend.rs` 403, `view/dashboard.rs`
342). `client/tests/tui_snapshot_tests.rs` renders 2,458 lines of snapshots
through `TestBackend` at 120x30 — a regression net any refactor can lean on.

## Fifteen screens

Splash (ASCII logo, decrypt-reveal, POST-style boot log) · Dashboard (host
mesh, fleet table, capacity, transfers) · Stacks (registry list + manifest +
app grid) · Log stream (source selector, colour-coded body, follow) ·
Doctor · Settings (form) · Shell (scrollback + prompt) · Focus window
(deploy takeover with transcript and gauge) · Change-plan modal · Stack-forge
wizard (5 steps) · Typed confirm (red, type-the-name) · Help overlay ·
Command palette · Pending-ask box · Too-small-terminal notice (floor 80x24).

## What it uses of ratatui

Everything it draws comes from **eight widget types**: Paragraph (54 uses),
Block (30), Tabs (2), List/ListItem (4/6), Table/Row/Cell (4/10/9), Gauge
(2 real), Clear (9), Wrap (1). `TableState` at `view/dashboard.rs:256` is
the only stateful widget in the whole application.

**Not present at all:** Sparkline, Chart, BarChart, Canvas, Scrollbar,
LineGauge, tree, calendar. No `tui-input`, `tui-textarea`,
`throbber-widgets-tui`, `tui-logger`, `tui-tree-widget`, `tachyonfx`.

## Colours

One `pub const THEME: Theme` in `client/src/tui/theme.rs`: 13 named
`Color::Rgb` fields plus semantic helpers (`ok`, `warn`, `err`, `border_*`,
`title_*`, `muted_style`, `hint`) and a per-stack identity hue
(`stack_color`, `theme.rs:88`). No named ratatui colours anywhere — all
truecolor. **18 distinct literals**, 14 in `theme.rs` and 4 leaking into
views (the scanline highlight `#0D2226` appears three times).

**There is no theme switching and no light mode**: `THEME` is a `const`, not
a field on the model. The only look control is `FxLevel` (Off/Subtle/Full)
on F2.

## What it hand-rolls, with the count

| Hand-rolled | Copies | Where one lives |
| ----------- | ------ | ---------------- |
| Popup: centred `Rect` + `Clear` + bordered block + `.inner()` | 7 | `view/mod.rs:104` |
| Text input with a blinking block caret | 5 | `view/mod.rs:810` |
| Percentage bar from block characters instead of a gauge | 2 | `view/dashboard.rs:44` |
| Scroll window + anchored tail, no scrollbar | 3 | `view/logs.rs:280` |
| Severity to style mapping | 3 | `view/logs.rs:297` |
| Selected-row styling | 4 | `view/stacks.rs:65` |
| Key hints, and the help overlay repeating the same keymap | 2 | `view/mod.rs:668` and `:750` |
| Wizard breadcrumb | 1 | `view/mod.rs:242` |
| Fuzzy command palette | 1 | `view/mod.rs:830` |
| Marquee ticker | 1 | `fx.rs:131` |
| Form field row | 1 | `view/mod.rs:328` |

## What is already cool, and must survive

- A **deterministic FX engine** keyed on `(tick, element id)` — glitch
  scramble, decrypt-reveal, sinusoidal pulse, a scanline sweeping table
  rows, marquee, power-cycle flicker on tab switch, braille spinner — three
  intensity levels on F2, each effect O(text length) at 30 fps (`fx.rs`).
- The **splash decrypt-reveal** with a per-row colour gradient, its boot log
  tied to the real connection state.
- An **attention-first ticker**: actionable items in yellow, otherwise calm
  telemetry and "ALL SYSTEMS NOMINAL".
- A **per-stack identity hue** carried through table, list, source bar and
  log lines, so a stack is recognisable by colour alone.
- **Marching-dot transfers** for live byte streams.
- The **deploy focus window**: live transcript, right-aligned status title,
  indeterminate gauge.
- **AZERTY-correct tab keys** and a command palette, so nothing requires
  knowing a keybind first.

## What this says kp-tui should build

In the order the copies argue for: **Popup** (7), **Field** with the
theme's own caret (5), **Selection** as one style (4), **LogPane** with
severity and a source hue (3 + 3), **Meter** (2), **KeyHints** from one
keymap that both the footer and the help overlay read (2), then Stepper,
Palette and Ticker.

**All of them are built, as of 2026-09-17.** The last three landed
together: `SelectList` (the row in hand, painted the way each register
paints `[aria-current]` — measured across the twenty-two), `Stepper` (the
wizard's breadcrumb) and `CommandPalette`, which matches a subsequence
rather than a substring and lifts the letters that hit — `dpl` finds
"Deploy stack", which homelab's `label.contains(&q)` does not.

And the part that makes it worth switching at all: the FX. homelab's are
hand-tuned for one cyberpunk look; kp-themes already declares a reveal
routine, a motion duration and a set of marks **per theme**, so kp-tui can
offer the same effects as theme-aware widgets — the same screen in terminal
reads as a phosphor CRT and in cyberpunk as a HUD, without a second palette.

That part is now built and measured one effect at a time in
[EFFECTS.md](EFFECTS.md), and one of homelab's own screens has been rebuilt
on the crate to see what the inventory missed —
[HOMELAB_PROOF.md](HOMELAB_PROOF.md) has the three gaps it found.
