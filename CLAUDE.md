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
| Current phase | **Building, after 0.1.0** (released 2026-09-20 with kp-themes 7.1.0). kp-themes' research is the scope [kp-themes scope-127]; the proof against homelab is `docs/HOMELAB_PROOF.md` |
| Last completed gate | **The 0.1.0 release, 2026-09-20.** The widgets form was answered 2026-09-17 in the kp-themes session ("alles in een keer", kp-themes scope-129) |
| Next gate | the keys form of 2026-09-26: who wins where a demo key and a homelab key collide, how far the rebuild follows homelab's keys [fix-68], the small-font read of the bars [fix-66-M1], when 0.1.1 is cut |
| Next action | waiting on Kenny: the keys form of 2026-09-26 (Hearth thread "kp-tui"); the bar page is https://claude.ai/artifact/DGoWLfpNk1fV7meN75et3E |
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
