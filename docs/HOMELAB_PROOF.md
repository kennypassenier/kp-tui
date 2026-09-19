# The proof: homelab's own screen, rebuilt on this crate

`docs/HOMELAB_INVENTORY.md` counted what homelab writes by hand and this
crate answered all eleven of them. That is an argument, not evidence. So
one real screen was rebuilt with nothing but this crate's widgets:
`client/src/tui/view/stacks.rs`, the fleet registry beside the manifest and
the app grid, which is the screen homelab opens on.

It lives in `crates/kp-tui/examples/demo/fleet.rs` and runs as
`cargo run --example demo -- --screen fleet --theme cyberpunk`.

## What it cost

| | homelab's `stacks.rs` | the rebuild |
| --- | --- | --- |
| Drawing code, non-blank lines | 177 | 183 |
| References to a theme constant | 34 `THEME.*` | 0 — every colour comes from `&Theme` |
| Colour literals | 1 (`Color::Rgb`) | 0 |
| Themes it renders in | 1 | 22 |

**The rebuild is not shorter.** Six lines longer, in fact, and that is the
finding worth having: the widgets that existed (`Panel`, `SelectList`,
`source_colour`, the reveal, `Surface`'s sweep over the app grid) cost
almost nothing to use, and the three things that did not exist cost their
full length in the demo. What the rebuild buys is not brevity — it is that
the same screen reads as a phosphor CRT in terminal, a HUD in cyberpunk and
a printed page in formal, without a second palette anywhere.

## The three gaps, as measured

Each one is marked `GAP n` in `fleet.rs`, so the shortfall is code a reader
can count rather than a claim in a document.

| Gap | What it is | Lines it cost here | Where it also sits |
| --- | --- | --- | --- |
| 1 | **A state dot and a tag** — `●`/`○` for up and down, `[UPD]`, `[OFF]` | 22 | homelab writes these in the stacks list, the fleet table and the settings screen; the web package has `.kp-badge` |
| 2 | **A facts list** — `label value   label value`, aligned, with a success, warning or destructive ink per value | 36 | homelab's manifest, doctor and settings screens; the web package has `.kp-definition` |
| 3 | **A themed table** — a header, a rule and a row rhythm the register's own way | 45 | homelab's app grid and fleet table; the web package has `.kp-table` with a register rule in eighteen of the twenty-two |

103 of the rebuild's 183 lines are those three. Everything else — the
panel, the list, the row in hand, the source hue, the reveal, the sweep —
was one call each.

Two smaller things the rebuild did not need but homelab has: an
indeterminate gauge (the deploy window) and a tree (`tui-tree-widget`).
Neither belongs to this screen; both are named here so the next proof knows
where to look.

## The second pass, 2026-09-18

All three gaps were closed and the screen rebuilt on them.

| | homelab | first pass | second pass |
| --- | --- | --- | --- |
| Drawing code, non-blank lines | 177 | 183 | 169 |
| Lines of hand-chosen style | — | 103 | 0 |
| Theme constants | 34 | 0 | 0 |
| Themes | 1 | 22 | 22 |

Fourteen lines shorter, and the 103 that were hand-rolled style are gone —
what is left is data (the fixture rows) and calls. The screen also gained
something it did not have: the load history as a braille chart, which is
twelve levels in three rows where a block sparkline gives eight in one.

What answered each gap:

- **`Badge`** — a chip that ends the way its register ends a button (half
  blocks where the theme is soft, brackets where it brackets), plus
  `Badge::dot` for a status list. Also the `[ RUN ]` in terminal, which is
  that register's brackets and not a decision of the screen's.
- **`Facts`** — label and value in columns, the labels of a column padded
  to one width, each value carrying its own tone.
- **`DataTable`** — the header in the register's own plate, case and
  tracking; the rule under it 1px, 2px or 3px per register — and a ramp
  from `--primary` to `--accent` for synthwave, the one register that
  draws that rule as a gradient. The tracking comes off a heading that
  does not fit its column, the same rule a button's label follows.

One thing deliberately not carried over: the 1px rule the base draws
between body rows. On a page that is a pixel; in a cell grid it is a whole
row, and it would halve how many records fit. The grid is the rule there.

## The second screen, 2026-09-19

One screen proves that the widgets fit the screen they were drawn from.
A second one, picked because it leans on different widgets, is what tells
you whether the crate generalises. So homelab's own dashboard —
`client/src/tui/view/dashboard.rs`, the screen it opens on after the
splash — was rebuilt the same way, in
`crates/kp-tui/examples/demo/ops.rs`:

```sh
cargo run --example demo -- --screen ops --theme cyberpunk
```

| | homelab's `dashboard.rs` | first pass | with the gaps closed |
| --- | --- | --- | --- |
| Drawing code, non-blank lines | 319 | 264 | 242 |
| Of those, comments | 4 | 18 | 14 |
| So: code | 315 | 246 | 228 |
| Lines of hand-chosen style | — | 18 | 0 |
| References to a theme constant | 48 `THEME.*` | 0 | 0 — every colour comes from `&Theme` |
| Colour literals | 1 (`Color::Rgb`) | 0 | 0 |
| Themes it renders in | 1 | 22 | 22 |

**Eighty-seven lines shorter, 28 %**, and this time the direction is the one
the first proof predicted: the first rebuild was six lines longer because
three widgets did not exist yet; with those three in the crate, a screen
that uses all of them comes out shorter than the hand-written original.
Counted with `awk 'NR>=12 && NF'` on homelab's file (its drawing code
starts at line 12) and `awk '/^pub fn draw\(/{f=1} f&&NF'` on the rebuild,
so the fixtures at the top of the demo file are not counted as drawing.

### What it found, and what was done about it

Three things. All three were closed the same day Kenny judged them, so
nothing on this screen is marked `GAP` any more and no colour on it is
chosen by hand.

**1 · A table cell was painted in one colour.** `DataTable` flattened every
cell to its first span's style, so the `▎` hue bar before a node's name
painted the whole name that hue, and a row carrying both an `off` and a
`noenv` badge drew them in one. A defect, not a gap: fixed in `pad_spans`,
which pads a cell to its column with every span intact and cuts at the
column's edge. The test
`a_table_cell_keeps_the_colour_of_every_span_it_is_built_from` fails
against the old code with `left: Rgb(163, 41, 41), right: Rgb(23, 30, 43)`
— the node name wearing the bar's hue — and passes against the new.
Recorded as `fix-2` in `docs/CORRECTIONS.md`.

**2 · Two columns touched.** `DataTable` put nothing between columns: the
width of a column was its slot, gutter included. A left-aligned column
carries its own gutter in its padding; a **right-aligned** one does not, so
its last character sat flush against the next column's first — `3/4UPD`,
`0/1OFFNOENV`, and a spaced header reading `A P P SF L A G S`. Now
`DataTable::spacing` puts a cell between two columns, one by default, the
way ratatui's own `Table` has a `column_spacing`; `spacing(0)` is the old
behaviour and is what drives
`a_right_aligned_column_does_not_end_against_the_next_one` red. A column's
width is its content again, which is why the four callers lost a cell each.

**3 · An indeterminate stream was hand-written.** homelab marches a mark
along the row for a transfer whose size nobody knows, and `Meter` wants a
fraction there is none of, so the rebuild wrote eighteen lines with a
colour chosen on the spot — the only hand-chosen style left on the screen.
The first proof had named this exact thing as what the next proof would
need, and it was right.

The answer was not invented here. The package had already decided what an
indeterminate bar looks like, at `gap-11` in `css/components.css`: *an
indeterminate bar is not a full one* — the track wears diagonal stripes in
the accent, the bar steps aside, and the stripes drift slowly with no
change of light [DI5], because a full bar and a flashing one both lie about
something nobody has measured. `Stream` is that decision in a cell grid:
the diagonal is `╱` (`/` for the two registers whose spinner is plain
ASCII, which would not draw a box-drawing diagonal either), the stripe
repeats every four cells, and the web's `kp-progress-stripes 1200ms linear
infinite` moves one stripe per 1200 ms — four cells, so 300 ms a cell.
`an_indeterminate_stream_drifts_without_changing_its_light` reads the
picture at 0, 300 and 600 ms, asserts the two colours never move, and holds
still under reduced motion.

That is the second proof's real finding: the first screen's three gaps were
widgets the crate lacked, and this screen's three were a defect, a trap and
a value that already existed on the web and had never been carried over.
