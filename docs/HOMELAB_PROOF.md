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

## What it says

The crate carries a screen of this shape today, and the next thing worth
building is whichever of the three gaps the next screen hits first. None of
them is a new idea: all three are components the web package already has,
which is the same road the log line and the source hue took in the other
direction (`gap-14` and `gap-15` in kp-themes).
