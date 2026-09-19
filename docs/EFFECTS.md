# The effects: what homelab writes by hand, and what the theme answers

homelab's `client/src/tui/fx.rs` is 190 lines and eight functions, keyed on
`(tick, element id)` so nothing has to be stored between frames. That rule
is worth keeping and this crate keeps it. What is not worth keeping is the
rest: every colour in it is one of eighteen literals in
`client/src/tui/theme.rs`, every period is counted in 30 fps ticks, and the
set is tuned for one look.

kp-themes already decided these questions once, per theme, in the twenty-two
registers — which texture the ground carries (DI9), what the alarm does,
whether anything is allowed to loop at all (DI5). So each effect below is
the same mechanic with the theme holding the knobs.

## The mapping

| homelab (`client/src/tui/fx.rs`) | kp-tui | What the theme decides |
| --- | --- | --- |
| `glitch(text, id, tick, level)` — 35 % of characters swapped from a fixed `GLITCH_CHARS`, in random 2-tick windows | `effects::Glitch` (`Alarm::glitch`) | Whether it glitches at all (cyberpunk alone), how often, how long, and how much. The glyphs are `fx::GLYPHS`, the package's own set — no block glyphs [AR40] |
| `decrypt(text, progress, id, tick)` — block characters resolving left to right | already `fx::Reveal::Decipher`, since the first round | Every theme's reveal routine: `Arrive`, `Decipher`, `Type` or `Words`, at its own `--fx-duration` |
| `pulse_bg(tick, level)` — sine between two hard-coded cyans, 2.6 s | `effects::pulse` (`Alarm::glow_ms`) | The two colours (a role pair out of the palette) and the period; `None` for a register whose glow is at alpha 0 |
| `scanline(h, tick, id, level)` — a lit row every 10 s | `effects::Sweep` (`Fx::sweep`) | Whether a row crosses at all. **One register declares it**: cyberpunk's `kp-alarm-sweep` at 6000 ms. Terminal's raster is static — DI5 measured a moving one and refused it |
| — (homelab has no static texture) | `Texture` + `components::Surface` | The ground itself: scanlines, drafting grid, halftone dots, twill, or nothing. Eleven of the twenty-two carry one a cell grid can hold |
| `ticker_text(segments, width, tick)` — joined with `  ::  ` | `components::Ticker` | The divider is the theme's own `tab_divider`; the speed is one knob |
| `flicker_phase(ticks_left)` — a few dark ticks, one bright flash | `effects::Strike` (`Alarm::strike`) | The stops come from `@keyframes kp-alarm-cyberpunk-flicker` [scope-100]: 0 → 0.6 → 0.52 → 1 over 600 ms, one cell sideways, **once**. Only the register that declares it |
| `spinner(tick)` — ten braille frames | `effects::Spinner` (`Fx::spinner`) | Braille, quadrant, half-block, bar or ASCII, per theme, at the 900 ms `--kp-spinner-duration` |
| `progress_marks(tick, width)` — a mark marching along the row for a transfer of unknown size | `components::Stream` | The package's own answer at `gap-11`: an indeterminate bar is not a full one, so the muted track wears the accent as diagonal stripes that drift with no change of light [DI5]. The diagonal is `╱`, or `/` in the two registers whose spinner is plain ASCII; the stripe repeats every four cells and `kp-progress-stripes 1200ms` moves one stripe, so 300 ms a cell |
| `braille_spark(data, width)` | already `dashboard`'s own chart | — |
| `◂ value ▸` written inline in the settings tab, with a hand-picked cyan on the row in hand | `components::Choice` | The marks are the package's own `var(--kp-glyph-closed, '▸')` and its mirror — no register overrides that token, measured across all 22 registers. What is per-theme is the plate the value in hand wears, and that is `Selection`, the same one a list row stands on |
| `●` drawn by hand beside a state word | `Badge::state` | The tone is the caller's, the ink is the theme's: `Theme::ink` picks one that reads on the card it sits on [fix-1] |
| a progress bar drawn the same way in every theme | `Meter` + `anatomy::Track` | How the register closes the bar's ends, read from its own `.kp-progress` rule across all 22: eight draw no border and get none, seven set `border-radius: 0` and get `[ ]`, nostromo's pill radius gets `( )`, and six between get the thin rails `▏ ▕` |
| `load_color(pct)` | already `Meter::thresholds` | `--success`, `--warning`, `--destructive` |

Two homelab effects were already here before this step (`decrypt`,
`load_color`), two more had an equivalent (`braille_spark`, the ticker),
and the four that were genuinely missing — glitch, pulse, sweep, strike —
are the ones the registers have the most to say about.

## What the registers say about looping

DI5 is the reason this is not a free-for-all. Seventeen of the twenty-two
registers state in a comment that nothing on the page loops or flickers,
and terminal's says why: *"flicker at .15s infinite was measured and
refused (DI5)"*. So:

- A **strike** happens once, on arrival, and then never again.
- A **glitch burst** returns on a period only where the register declares a
  looping keyframe; `every_ms: 0` is the single-burst form.
- The **glow** is the package's own `kp-alarm-pulse` at 1400 ms
  `infinite alternate`; the registers that set its alpha to 0 get
  `glow_ms: None` and the loop disappears with it.
- The **texture** does not move. A sweep is a separate declaration, and one
  register makes it.

## Reduced motion

`Motion::Reduced` is not a dimmer: every effect returns its resting state
on the first frame. No substitution, no sweep, no pulse, the spinner holds
its first frame and the ticker shows the head of the line. Sixteen-colour
terminals get no texture at all, because a 4.5 % tint cannot be expressed
there and a wrong one is worse than none.

## Looking at it

```
cargo run --example demo -- --screen effects --theme cyberpunk
cargo run --example demo -- --shot --colors truecolor --screen effects \
    --theme cyberpunk,terminal --at 5100 --size 112x20
```

`--shot` draws the screen headless and prints it as ANSI, through the same
`App::draw` the running demo uses, so a picture of it cannot drift from
what the demo shows.

## The components beside them

The effects are half of what makes a screen; the other half is the widgets
that carry them. `docs/HOMELAB_INVENTORY.md` counted what homelab writes by
hand, and all of it now lives here: `Popup`, `Field`, `Meter`, `KeyHints`,
`LogPane`, `SelectList`, `Stepper` and `CommandPalette`.

Two of those learned something on the way over:

- **`SelectList`** takes the row in hand from the register rather than from
  a constant. Measured on 2026-09-17: four registers plate it in
  `--primary`, one in `--muted`, one in `--secondary`, two in `--accent`,
  two drop it to `--background`, and nine leave the ground alone and speak
  with a bar, a bracket, a dot or a weight instead. terminal is the only
  one whose marker is a glyph — a literal `>` — and it is in the register.
- **`CommandPalette`** matches a subsequence, not a substring, so `dpl`
  finds "Deploy stack" where `label.to_lowercase().contains(&q)`
  (`client/src/tui/model.rs:1135`) finds nothing. The letters that hit are
  lifted in the theme's primary ink and underlined; the underline is what
  survives on the row in hand, whose plate is often that same primary.

## Depth

An overlay does two things to the page it covers, both one pass over the
buffer and neither with a colour of its own:

- **The scrim.** Every cell behind the overlay moves a third of the way to
  the theme's own `--background`, ink included, so the layer underneath
  reads as further away rather than as switched off.
- **The shadow.** One row under the overlay and one column beside it,
  darkened 45 % towards black in a dark theme and towards `--foreground` in
  a light one, so the shadow reads on both.

Sixteen-colour terminals get neither: a third of the way does not exist
there, and a wrong colour is worse than a flat page.

## The arrival, the rail, the corners and the roll

Four more, all measured from the registers rather than invented:

- **`Stage`** — the four beats `css/components.css` has declared since the
  alarm was built: the ground in over 240 ms, the panels settling over
  520 ms, the titles over 480 ms, the detail over 300 ms. `Panel::stage`
  brings a frame and its plate up out of `--background` on the panel beat
  and its title on the title beat, so a screen arrives instead of being
  there.
- **`Rail`** — one row across the top. Three registers paint a gradient
  onto a rule rather than a colour (`border-image`: synthwave eight times,
  terminal and retro three each) and those three ramp from `--primary` to
  `--accent`, cell by cell — 104 distinct colours across a 104-cell row.
  The other nineteen draw the plain rule they draw everywhere else.
- **HUD corners** — `⌜⌝⌞⌟` in `--ring` on the panel with the focus, and
  only for the eight registers that cut their corners in earnest
  (`clip-path` five times or more: cyberpunk 42, phantom 13, dark 12,
  retro 8, titanium 6, lapis 6, solstice 5, light 5). Six registers never
  cut a corner, and they never grow one here.
- **`roll`** — a number eased to its new reading over the theme's own
  `--fx-duration` instead of snapping to it. It is monotonic, it lands
  exactly on the target, and it is not a loop, so the flash reading stays
  at zero.

And the block sparklines on the dashboard are braille now, at two samples
to a column and four levels to a row; the percentage bar fills in eighths
of a cell rather than whole ones.
