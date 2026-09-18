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

## The queue

| ID | What | Status |
| --- | --- | --- |
| fix-1-M1 | Does a state word read where it is painted? Measured at the next screen this crate draws: the contrast test is green, and Kenny reads the state words on the shot without asking what they say. | open |
