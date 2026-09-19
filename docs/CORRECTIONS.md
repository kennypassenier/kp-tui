# Corrections — kp-tui

Live-found faults and the measures approved for them. One entry per fault,
nine fields, in the order `~/Projects/dev-procedure/FORM_PROTOCOL.md` §8
sets.

## fix-1 · A state colour is a plate, not an ink

Found by Kenny on 2026-09-18, looking at the fleet screen: *"in fleet2.html
is de groene tekst in synthwave (wat doet groen daar uberhaupt?) bijna
onleesbaar"*.

1. **What went wrong.** `--success`, `--warning` and `--destructive` were
   painted as text. They are plates. Measured across the generated palette:
   `--success` on `--card` reads **1.19:1** in synthwave, 1.16:1 in
   titanium, 1.21:1 in light — **43 of the 66 state/card pairs in the set
   are under 4.5:1**, ten of them under 1.5:1.
2. **Which gate let it through.** None existed. The crate had 44 tests and
   not one of them took a contrast reading; the package's own
   `check-site.mjs` measures the site, not this crate.
3. **Where else the same fault sits.** The fault is "a token used on the
   wrong side of the paint". Searched with
   `grep -n "\.fg(" src/*.rs examples/demo/*.rs | grep -E "success|warning|destructive|info"`
   — seven sites: the severity tag, the severity message, the paused
   state, the danger frame, the stepper's finished steps, the status dot,
   the meter's reading. All seven are repaired.
4. **How we prevent recurrence.** `Theme::ink(tone, surface)` measures both
   of a state's tokens against the surface it will be painted on, takes the
   better, and lifts it towards `--foreground` a twentieth at a time until
   it reads at 4.5:1. `Theme::on_plate` does the other direction, with
   black and white in the running last — shade-dark's warning plate tops
   out at 4.17:1 against every colour its own register declares.
5. **What the remedy costs.** A lifted ink is not the register's exact
   colour. In eighteen registers nothing moves (the pair already reads); in
   the other four the hue survives and the lightness travels.
6. **Who enforces it.** Code: a test walks 22 themes × 4 tones × 3 surfaces
   plus the four plates and fails under 4.5:1.
7. **How we measure that it works, and when.** At the next screen this
   crate draws: the test is green, and Kenny reads the state words on the
   shot without asking what they say. Queued as `fix-1-M1`.
8. **The fallback if it fails.** If a lifted ink reads wrong to Kenny in
   some theme, the state word moves onto its plate instead — a plated
   badge, which is what the web does — and the ink lift is kept only for
   text that cannot carry a plate.
9. **When we review the measure.** At the first release of this crate: if
   no state ink has needed a hand-written exception by then, the lift stays
   as it is.

## fix-2 · A table cell was painted in the colour of its first part

Found on 2026-09-19, rebuilding homelab's dashboard as the second proof
[docs/HOMELAB_PROOF.md].

1. **What went wrong.** `DataTable` flattened a cell to one span: it joined
   the cell's text and took the style of the first part. A cell is built
   out of parts that each carry a colour — the `▎` hue bar before a node's
   name, the `●` before its status, two badges side by side — so the whole
   name wore the bar's hue, and a row carrying both an `off` and a `noenv`
   badge drew them in one colour. Measured: `media` came out
   `Rgb(163, 41, 41)` where the name's own ink is `Rgb(23, 30, 43)`.
2. **Which gate let it through.** None could. Every table test so far
   asserted a width, an alignment or the rule under the header — the shape
   of the table — and none asserted a colour inside a row, so the widget
   could throw a style away and stay green.
3. **Where else the same fault sits.** The fault is "a widget that
   rebuilds a `Line` instead of padding it". **Searched with**
   `grep -n 'spans.iter().map(|s| s.content' crates/kp-tui/src` — after the
   fix there is no other place: `SelectList` and `Facts` render the lines
   they are given.
4. **How we prevent recurrence.** `pad_spans` pads a cell to its column
   with every span intact and cuts at the column's edge, and the widget
   calls it for every cell.
5. **What the remedy costs.** A cell is now several spans instead of one,
   so a row is a slightly longer `Line`. Nothing in the drawn output moves
   except the colours that were being lost.
6. **Who enforces it.** Code:
   `a_table_cell_keeps_the_colour_of_every_span_it_is_built_from` builds a
   cell out of a hue bar and a name and another out of two badges, and
   reads the four colours back out of the buffer.
7. **How we measure that it works, and when.** At this commit the test
   fails against the old code (`left: Rgb(163, 41, 41), right: Rgb(23, 30,
   43)`) and passes against the new, and the ops shot shows five hue bars
   and two badge colours. Again at the third screen this crate draws.
   Queued as `fix-2-M1`.
8. **The fallback if it fails.** If padding around spans turns out to cut
   a wide glyph in half at a column edge, the cut moves to a width-aware
   one (`unicode-width`) rather than back to flattening.
9. **When we review the measure.** At the first release of this crate,
   with fix-1.

## fix-64 · A bar began where the word in front of it ended

Kenny, on the twenty-five pictures, 2026-09-19: *"Die balken moeten op
hetzelfde startpunt beginnen, nu is de balk achter "memory" veel later
omdat memory langer is dan cpu. … Dus aparte kolommen basically. Elementen
beginnen op vaste punten, niet afhankelijk van de lengte van andere
elementen."*

1. **What went wrong.** `Meter` wrote its label, one space, then the bar.
   Three meters under each other therefore started their bars in three
   different columns, because `cpu`, `memory` and `disk` are three
   different lengths. The same fault sat in the command picture (the `·`
   after a stack name), in the log pane (the message after the unit name),
   in the retention row (the second stepper after the first), and in the
   deploy question (the consequence after the word).
2. **Why it happened.** Every one of those lines was built by
   concatenating spans, and a concatenation has no idea what the line
   above it did. Nothing in the crate offered a column to share, so
   nothing shared one.
3. **Where else the same fault sits.** **Gezocht met:**
   `grep -rn 'format!("{' crates/kp-tui/src crates/kp-tui/examples` over
   every span that writes a label — the five above, plus the two the
   register already padded (`Choice::label_spans`, `Severity::tag`), which
   is where the fix was copied from.
4. **How we prevent recurrence.** `label_column(theme, labels)` measures
   what a group of labels needs, once, in the register's own dress;
   `Meter::label_width`, `Field::label_width` and `Choice::label_spans`
   take that number. `LogBuffer::columns` does the same for the host and
   unit columns of a log line. A caller that draws a group now passes one
   width to all of it.
5. **What the remedy costs.** A cell or two of empty space on the shortest
   label of every group, and one call per group. Nothing at runtime: it is
   a `max` over a handful of strings.
6. **Who enforces it.** Code:
   `meters_in_one_group_start_their_bars_in_the_same_column` renders `cpu`,
   `memory` and `disk` in all 22 registers and reads the first painted
   plate cell out of the buffer for each; the three columns must be equal.
7. **How we measure that it works, and when.** At this commit the test is
   green over 22 themes, and the stacks shot shows five cards whose bars
   all begin in column 9. Again at the next screen built from a direction,
   where the question is whether the caller reached for `label_column`
   without being told. Queued as `fix-64-M1`.
8. **The fallback if it fails.** If callers keep forgetting, the column
   moves into the widget group rather than the call: a `Meters` widget
   that takes the rows and measures them itself.
9. **When we review the measure.** At the first release of this crate.

## fix-65 · A rebuild quietly dropped what the screen could do

Kenny, on the log direction, 2026-09-19: *"Maar kijk bij homelab rust
welke features het logboek moet hebben. … Ik wil geen features kwijt
geraken. Een logscherm is niet enkel esthetisch, het heeft ook deze
functies en kijk welke ik nog vergeten ben."*

1. **What went wrong.** The log rebuild's own header claimed it "found
   nothing missing". Two things were missing. The source selector painted
   itself and filtered nothing — `LogBuffer::visible` knew about the
   severity filter and not about the selected unit. And scrolling back
   down to the last line never let go of the pause, where homelab resumes
   following the moment the view reaches the tail.
2. **Why it happened.** The rebuild was read against homelab's *drawing*
   code, `view/logs.rs`, and both behaviours live in its key handler,
   `model.rs`. A screen was compared with a screen; a screen is also what
   its keys do.
3. **Where else the same fault sits.** **Gezocht met:**
   `grep -n 'Tab::\w* => match key.code' -A 40 client/src/tui/model.rs`
   in homelab, read against each rebuilt screen. The stacks screen lost
   nothing (its list became cards and the manifest stayed); the dashboard
   lost nothing; settings lost nothing; the deploy window lost nothing.
   The log screen lost those two, and gained a level filter homelab does
   not have.
4. **How we prevent recurrence.** A rebuilt screen is read against both
   files — the view and the key handler — and the demo binds every key the
   original binds, so the footer can name them. This screen's footer now
   lists source, scroll, follow, level, tail.
5. **What the remedy costs.** `LogBuffer` carries one more field and the
   demo one more index; `scroll_down` has one more branch.
6. **Who enforces it.** Code: `a_selected_source_really_hides_the_others`
   and `scrolling_back_down_to_the_tail_follows_again`.
7. **How we measure that it works, and when.** At this commit both tests
   are green, and the log shot draws the selector, the band, the level in
   the title and the six keys in the footer. Again when a sixth screen is
   rebuilt: the question is whether its key handler was read at all.
   Queued as `fix-65-M1`.
8. **The fallback if it fails.** If reading two files by hand keeps
   missing one, the inventory becomes a list in `docs/HOMELAB_PROOF.md`
   per screen — every key the original binds, ticked off — and the rebuild
   is not called done until every row is ticked.
9. **When we review the measure.** At the first release of this crate.

## The queue

| ID | What | Status |
| --- | --- | --- |
| fix-1-M1 | Does a state word read where it is painted? Measured at the ops screen, 2026-09-19: the contrast test walks 22 themes × 4 tones × 3 surfaces plus the four plates and is green, and Kenny read `up`, `down`, `upd`, `off`, `noenv` and `alloc 134%` on the shot in six registers without asking what they said. | closed |
| fix-2-M1 | Does a table cell keep the colour of every part it is built from? Measured at this commit (2026-09-19): the test fails against the old code with left Rgb(163, 41, 41), right Rgb(23, 30, 43), and passes against the new; the ops shot draws five hue bars and two badge colours in one row. Again at the third screen this crate draws. | open |
| fix-64-M1 | Do elements begin at fixed columns? Measured at this commit (2026-09-19): `meters_in_one_group_start_their_bars_in_the_same_column` is green over 22 registers, and the stacks shot shows every card's two bars beginning in the same column. Again at the next screen built from a direction. | open |
| fix-65-M1 | Does a rebuilt screen still do everything the original did? Measured at this commit (2026-09-19): the two log tests are green, and the log screen binds all five keys homelab binds plus the level filter it lacks. Again at the sixth rebuilt screen. | open |
