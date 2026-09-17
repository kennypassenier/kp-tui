# kp-tui 🖥

The kp-themes house themes in a terminal: generated palettes, a hand-written anatomy, and ratatui widgets.

This project follows the dev procedure in `~/Projects/dev-procedure/`
(`/project-flow`). Standing rules apply to every change:
`~/Projects/dev-procedure/STANDING_RULES.md`.
Enforcement is **git-native** (`.githooks/` via `core.hooksPath`), so
gates hold from any session or terminal. After a fresh clone, run:
`git config core.hooksPath .githooks`.

## Procedure status

| Field | Value |
|---|---|
| Current phase | **Building.** kp-themes' research is the scope [kp-themes scope-127]: `research/ratatui/README.md` and `ANATOMY_PROPOSAL.md` |
| Last completed gate | **The crates form, 2026-09-17** (in the kp-themes session): two crates, own repository, the research as scope |
| Next gate | the widgets: which of the demo's widgets move over, and in what shape |
| Next action | waiting on Kenny: the widgets form, asked in the kp-themes session |
| AFK mode | off |

<!-- Update this block after every completed gate. -->
<!-- `Next action` is read by hooks/may-i-stop.py, which refuses to end a
     turn unless it says "waiting on Kenny: <what>" [rule 48]. Anything
     else in that row — or an empty one — means the work continues. Keep
     it current: it is the one row that decides whether a turn may end. -->

## Project documents

| Doc | Purpose |
|---|---|
| docs/SCOPE.md | goals, non-goals, success criteria, constraints (Phase 0) |
| docs/FEATURES.md | rated feature list with permanent IDs (Phase 2) |
| docs/ARCHITECTURE_DECISIONS.md | frozen AR decisions incl. tech choice (Phases 3-4) |
| docs/REALIZATION_PLAN.md | milestones + status table (Phase 5) |
| docs/TEST_PLAN.md | what is proven where + accepted limitations (Phase 7) |

## Gates (enforced)

Commits are blocked by `.claude/hooks/check-commit.sh` unless
`.claude/hooks/gates.sh` passes and the message carries IDs in
brackets (`[W12]`, `[L4b]`, `[meta]`). CI re-runs the same gates on
every push; red blocks merge.
