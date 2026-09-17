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
| `braille_spark(data, width)` | already `dashboard`'s own chart | — |
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
