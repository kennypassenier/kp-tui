//! Twenty-five design directions, five per rebuilt screen.
//!
//! Kenny, 2026-09-19, after the five rebuilds: "ik heb het idee dat we de
//! UI nog naar een hoger niveau kunnen tillen. Het kan nog cooler/moderner
//! zijn. Dus ik verwacht van elk scherm vijf ontwerpen waarin je
//! nieuwe/modernere/coolere features of aanpakken toont om zo een scherm
//! aan te pakken." He then chose the shape: five pictures per screen, one
//! paragraph each, and the one he picks gets built for real.
//!
//! So each of these is a picture, not a screen: it shows the idea at the
//! size the idea needs and takes its fixtures from the rebuilds beside it.
//! None of them is wired to a key, and that is the point — the cost of a
//! direction nobody chooses is one draw function.
//!
//! Every one of them still names no colour of its own.

use kp_tui::{
    Badge, Choice, Column, DataTable, Facts, Field, Meter, SelectList, Spark, Stage, Stepper,
    Stream, Surface, Theme, Tone, fx::Motion, source_colour, spinner, widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::fleet::FLEET;

pub struct Design {
    /// The rebuilt screen this is a direction for.
    pub screen: &'static str,
    pub name: &'static str,
    /// What the direction is, in one paragraph.
    pub idea: &'static str,
}

pub const DESIGNS: [Design; 25] = [
    // ── the stacks screen ───────────────────────────────────────────────
    Design {
        screen: "stacks",
        name: "Card grid",
        idea: "Every stack becomes a four-row card instead of a table row: its own hue down the leading edge, the state and the flags on the first line, the app count and the last deploy under it, and a braille strip of the last ten minutes at the foot. A table makes you read across; a card makes you read one thing. The card in hand arrives with the stagger the crate already has, so moving through the fleet has motion instead of a moving highlight.",
    },
    Design {
        screen: "stacks",
        name: "Deploy rail",
        idea: "A timeline instead of a registry. Each stack keeps one row, and the row is a rail of the last twenty deploys — a cell per deploy, in the tone of its outcome, newest at the right. You see at a glance which stack is churning and which has been still for a week, which is the question a registry never answers. The rail is the theme's own rule glyph, so a register that draws a gradient rule draws a gradient rail.",
    },
    Design {
        screen: "stacks",
        name: "Split preview",
        idea: "A narrow list on the left and everything else on the right, redrawn as you move. No manifest panel, no app grid, no fixed boxes — one preview that changes shape per stack: an app grid for a stack with apps, a single large status for one that is down, a diff for one that has drifted. The screen has one job at a time instead of four panels that are mostly empty.",
    },
    Design {
        screen: "stacks",
        name: "Tile wall",
        idea: "One tile per app rather than per stack, packed as densely as the terminal allows: a plate in the state's tone, the name inside it, nothing else. Forty containers fit where five rows used to. It answers the only question that matters at a glance — is anything red — and the fleet's shape is visible as a shape rather than a list. Moving the cursor over a tile opens the detail in a strip at the foot.",
    },
    Design {
        screen: "stacks",
        name: "Command first",
        idea: "The screen opens as a command field with the fleet under it, and typing filters and acts in one motion: `web res` restarts web, `med log` opens media's log. The list is the result, not the interface. This is homelab's own palette promoted from an overlay to the screen, and it is the direction that trades browsing for speed — best for someone who knows the fleet by heart and worst for someone who does not.",
    },
    // ── the dashboard ───────────────────────────────────────────────────
    Design {
        screen: "dashboard",
        name: "HUD corners",
        idea: "The readouts move to the four corners and the middle is left empty for the one thing that is happening now. Corner brackets rather than full frames, the numbers hung off them, and the centre carrying either nothing or the running deploy. It reads as an instrument rather than a report, and it is the only layout here that gets quieter as the host gets calmer.",
    },
    Design {
        screen: "dashboard",
        name: "Sparks behind",
        idea: "Every number gets its own history behind it: load, memory, disk and network each drawn as a braille sparkline with the current figure sitting on top of it. No separate chart panel. A number without its last ten minutes is a number you cannot judge, and this is the cheapest way in a terminal to put the two in the same glance.",
    },
    Design {
        screen: "dashboard",
        name: "Big figures",
        idea: "One column, four figures, each three rows tall, drawn in half blocks so they read from across the room. Everything else — the fleet, the transfers — drops to a single line at the foot. This is the wall-display direction: the screen is not being worked at, it is being watched from a desk on the other side of it.",
    },
    Design {
        screen: "dashboard",
        name: "Rings",
        idea: "Capacity as braille rings instead of bars. A ring carries the same fraction in a quarter of the width, so memory, disk, swap and load sit side by side on one row and the panel that used to hold four bars holds the fleet instead. The ring's gap is the theme's own dot glyph and its fill takes the same thresholds the bar had.",
    },
    Design {
        screen: "dashboard",
        name: "Alarms first",
        idea: "The screen sorts itself. What needs attention is at the top at full size, what is healthy shrinks to a single line, and a host with nothing wrong shows four quiet lines and a lot of ground. The layout is the alarm: you never read the dashboard to find the problem, the problem has already moved to where your eye lands.",
    },
    // ── the settings ────────────────────────────────────────────────────
    Design {
        screen: "settings",
        name: "Two columns",
        idea: "The setting on the left, its explanation on the right, aligned row for row. The rebuild puts the explanation after the value on the same line, which is why those lines run long; splitting them lets the values line up in a column you can scan and gives each explanation as much room as it needs.",
    },
    Design {
        screen: "settings",
        name: "Live preview",
        idea: "Changing a value shows what it does, under it, immediately: stepping the retention tiers redraws a strip of the next sixty days with the snapshots that would be kept. A settings screen that only names a policy makes you simulate it in your head; this one simulates it for you, and it is the direction with the most to build behind it.",
    },
    Design {
        screen: "settings",
        name: "Grouped cards",
        idea: "Each group of settings becomes its own panel — schedule, retention, notification — with the panel in hand focused and the rest at rest. The screen gains the frames the rebuild deliberately does without, and in exchange the eye has somewhere to land: three cards instead of fourteen rows.",
    },
    Design {
        screen: "settings",
        name: "Diff",
        idea: "Two columns: what the host is running, and what you have changed. Unsaved values show in the warning tone beside the saved ones they would replace, and the save key says how many. The dirty dot at the foot of the rebuild says something is unsaved; this says exactly what, which is the difference between trusting the screen and checking it.",
    },
    Design {
        screen: "settings",
        name: "One at a time",
        idea: "The wizard direction: one setting on screen, large, with its explanation and its consequence, and the rest as a breadcrumb. Slow to walk through and impossible to get wrong — right for a first run, wrong for changing one hour on a Tuesday. It is in the list because a settings screen that is used twice a year is a different problem from one used daily.",
    },
    // ── the log stream ──────────────────────────────────────────────────
    Design {
        screen: "logs",
        name: "Density band",
        idea: "A braille band above the rows showing how much arrived when, over the last ten minutes, with the errors in their own tone on the same band. You see the burst before you read it, and you can aim at it. A log pane without a shape makes you scroll to discover there was a storm at 09:41.",
    },
    Design {
        screen: "logs",
        name: "Grouped sources",
        idea: "The stream folds by source: one row per source with its count and its worst severity, and the source you open expands in place. Five sources talking at once is the state a multiplexed feed is always in, and reading it interleaved is a choice rather than a given.",
    },
    Design {
        screen: "logs",
        name: "Severity focus",
        idea: "Errors get two rows and their own plate, warnings one row in full ink, info a dimmed line, debug a single character in the gutter. The pane stops pretending every line is worth the same width. What it costs is the even rhythm a log has; what it buys is that an error cannot scroll past unnoticed.",
    },
    Design {
        screen: "logs",
        name: "Time gutter",
        idea: "The timestamps move into a rail down the left edge, drawn as a continuous line with a tick where a second starts and a gap where nothing arrived. Time becomes a shape rather than a column of digits, and the quiet minutes become visible as quiet rather than as absence.",
    },
    Design {
        screen: "logs",
        name: "Stream and detail",
        idea: "The pane splits when a line is selected: the stream above, the line's full record below — every field the journal carries, the unit's last state, and the lines either side of it in its own source. A log line is a summary of a record, and this is the direction that admits it.",
    },
    // ── the deploy window ───────────────────────────────────────────────
    Design {
        screen: "deploy",
        name: "Step rail",
        idea: "The deploy as its steps rather than as its transcript: sync, gates, pull, up, health — each a row with its own state and its own duration, the running one carrying the spinner. The transcript moves to a strip under it. You stop reading output to work out where it is.",
    },
    Design {
        screen: "deploy",
        name: "Two panes",
        idea: "Steps on the left, transcript on the right, both live. It is the step rail without the loss: the shape of the deploy and the detail of it at once, at the cost of half the width for each. Best on a wide terminal and unusable on a narrow one, which is exactly the trade to put in front of Kenny.",
    },
    Design {
        screen: "deploy",
        name: "Gate cards",
        idea: "Every safety gate becomes a card that turns as it passes: whitelist, hostname guard, fail-closed, digests pinned. A deploy that is blocked shows which card did not turn, in the destructive tone, with what it read. The gates are the part of a deploy nobody sees until one fails, and this direction makes them the screen.",
    },
    Design {
        screen: "deploy",
        name: "Full status",
        idea: "One word across the middle in half blocks — LIVE, PASSED, FAILED — with the transcript behind it at a tenth of its weight and the keys at the foot. A deploy finishing is the moment the operator looks up from something else, and this is the direction that can be read from the door.",
    },
    Design {
        screen: "deploy",
        name: "Docked bar",
        idea: "The deploy stops being a window. It docks as a two-row bar at the foot of whatever screen you are on: the step, the stream and the elapsed time, expanding to the full window on a key. Backgrounding a deploy is the normal case, and an overlay that has to be dismissed is the interface arguing with that.",
    },
];

pub fn draw(frame: &mut Frame, th: &Theme, n: usize, reveal_ms: u32, motion: Motion) {
    let area = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    match n {
        0 => card_grid(frame, th, area, stage, reveal_ms, motion),
        1 => deploy_rail(frame, th, area, stage, reveal_ms, motion),
        2 => split_preview(frame, th, area, stage, reveal_ms, motion),
        3 => tile_wall(frame, th, area, stage, reveal_ms, motion),
        4 => command_first(frame, th, area, stage, reveal_ms, motion),
        5 => hud_corners(frame, th, area, reveal_ms, motion),
        6 => sparks_behind(frame, th, area, stage, reveal_ms, motion),
        7 => big_figures(frame, th, area),
        8 => rings(frame, th, area, stage, reveal_ms, motion),
        9 => alarms_first(frame, th, area, stage, reveal_ms, motion),
        10 => two_columns(frame, th, area, stage, reveal_ms, motion),
        11 => live_preview(frame, th, area, stage, reveal_ms, motion),
        12 => grouped_cards(frame, th, area, stage, reveal_ms, motion),
        13 => diff(frame, th, area, stage, reveal_ms, motion),
        14 => one_at_a_time(frame, th, area, stage, reveal_ms, motion),
        15 => density_band(frame, th, area, stage, reveal_ms, motion),
        16 => grouped_sources(frame, th, area, stage, reveal_ms, motion),
        17 => severity_focus(frame, th, area, stage, reveal_ms, motion),
        18 => time_gutter(frame, th, area, stage, reveal_ms, motion),
        19 => stream_and_detail(frame, th, area, stage, reveal_ms, motion),
        20 => step_rail(frame, th, area, stage, reveal_ms, motion),
        21 => two_panes(frame, th, area, stage, reveal_ms, motion),
        22 => gate_cards(frame, th, area, stage, reveal_ms, motion),
        23 => full_status(frame, th, area, reveal_ms, motion),
        _ => docked_bar(frame, th, area, stage, reveal_ms, motion),
    }
}

// ── the parts every direction borrows ───────────────────────────────────

fn panel<'a>(th: &'a Theme, title: &'a str, stage: Stage, ms: u32, motion: Motion) -> Panel<'a> {
    Panel::new(th, title)
        .focused(true)
        .stage(stage)
        .reveal(ms, motion)
}

fn put(frame: &mut Frame, th: &Theme, area: Rect, lines: Vec<Line<'static>>) {
    Paragraph::new(lines)
        .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
        .render(area, frame.buffer_mut());
}

fn ink(th: &Theme, tone: Tone) -> Style {
    Style::new().fg(th.ink(tone, th.id.palette().card))
}

fn muted(th: &Theme) -> Style {
    Style::new().fg(th.c.muted_foreground)
}

/// A braille strip of made-up history, so every direction that wants one
/// draws the same shape.
fn history(seed: f64) -> Vec<f64> {
    (0..48)
        .map(|i| 2.4 + ((i as f64 + seed) / 4.0).sin() * 1.6 + (i as f64) * 0.02)
        .collect()
}

// ── stacks ──────────────────────────────────────────────────────────────

fn card_grid(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(area);
    for (i, stack) in FLEET.iter().enumerate() {
        let col = cols[i % 3];
        let row = Rect {
            y: col.y + (i / 3) as u16 * 6,
            height: 6.min(col.height),
            ..col
        };
        if row.y + row.height > area.y + area.height {
            break;
        }
        let p = panel(th, stack.name, stage, ms, motion);
        let inner = p.block().inner(row);
        frame.render_widget(p, row);
        let running = stack.apps.iter().filter(|a| a.1).count();
        let mut head = vec![
            Span::styled("▎", Style::new().fg(source_colour(th, stack.name))),
            Badge::dot(th, stack.online),
            Span::styled(if stack.online { " up" } else { " down" }, muted(th)),
            Span::raw("  "),
        ];
        if stack.drift {
            head.extend(Badge::new(th, "upd", Tone::Warning).spans());
        }
        put(
            frame,
            th,
            inner,
            vec![
                Line::from(head),
                Line::from(vec![
                    Span::styled(
                        format!("{running}/{} apps", stack.apps.len()),
                        ink(th, Tone::None),
                    ),
                    Span::styled("   deployed 4h ago", muted(th)),
                ]),
            ],
        );
        if inner.height > 2 {
            let strip = Rect {
                y: inner.y + inner.height - 1,
                height: 1,
                ..inner
            };
            Spark::new(th, &history(i as f64 * 3.0))
                .max(6.0)
                .render(strip, frame.buffer_mut());
        }
    }
}

fn deploy_rail(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Deploy history", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let mut lines = vec![Line::from(vec![
        Span::styled(format!("{:<12}", "stack"), muted(th)),
        Span::styled("last twenty deploys", muted(th)),
    ])];
    for (i, stack) in FLEET.iter().enumerate() {
        let mut rail = vec![Span::styled(
            format!("{:<12}", stack.name),
            ink(th, Tone::None),
        )];
        for step in 0..20 {
            let tone = match (i + step) % 7 {
                0 if step > 12 => Tone::Danger,
                1 => Tone::Warning,
                _ => Tone::Success,
            };
            rail.push(Span::styled("▆", ink(th, tone)));
        }
        rail.push(Span::styled("  4h", muted(th)));
        lines.push(Line::from(rail));
    }
    put(frame, th, inner, lines);
}

fn split_preview(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [list, preview] =
        Layout::horizontal([Constraint::Length(20), Constraint::Min(30)]).areas(area);
    let lp = panel(th, "Fleet", stage, ms, motion);
    let li = lp.block().inner(list);
    frame.render_widget(lp, list);
    let items: Vec<Line<'static>> = FLEET
        .iter()
        .map(|s| Line::from(s.name.to_string()))
        .collect();
    SelectList::new(th, &items, 1).render(li, frame.buffer_mut());

    let pp = panel(th, "web · what it is doing", stage, ms, motion);
    let pi = pp.block().inner(preview);
    frame.render_widget(pp, preview);
    let [facts, bars] = Layout::vertical([Constraint::Length(3), Constraint::Min(2)]).areas(pi);
    Facts::new(
        th,
        &[
            ("state", "running".into(), Tone::Success),
            ("apps", "3 of 3".into(), Tone::None),
            ("drift", "none".into(), Tone::Success),
            ("deployed", "4h ago".into(), Tone::None),
        ],
    )
    .columns(2)
    .render(facts, frame.buffer_mut());
    let rows = Layout::vertical([Constraint::Length(1); 3]).split(bars);
    // One column for all three bars: "memory" may not push its own bar
    // further right than "cpu" pushes its [fix-64].
    let column = kp_tui::label_column(th, &["cpu", "memory", "disk"]);
    for (i, (label, value)) in [("cpu", 0.22f32), ("memory", 0.64), ("disk", 0.41)]
        .into_iter()
        .enumerate()
    {
        if let Some(r) = rows.get(i) {
            Meter::new(th, label, value)
                .label_width(column)
                .render(*r, frame.buffer_mut());
        }
    }
}

fn tile_wall(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Every container", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let names: Vec<(&str, bool)> = FLEET
        .iter()
        .flat_map(|s| s.apps.iter().map(|a| (a.0, a.1)))
        .collect();
    let per_row = (inner.width / 14).max(1) as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    for chunk in names.chunks(per_row) {
        let mut row = Vec::new();
        for (name, up) in chunk {
            let tone = if *up { Tone::Success } else { Tone::Danger };
            let plate = th.tone(tone).unwrap_or(th.c.muted);
            row.push(Span::styled(
                format!(" {:<11} ", name.chars().take(11).collect::<String>()),
                Style::new()
                    .fg(th.on_plate(match tone {
                        Tone::Success => th.id.palette().success,
                        _ => th.id.palette().destructive,
                    }))
                    .bg(plate),
            ));
            row.push(Span::raw(" "));
        }
        lines.push(Line::from(row));
        lines.push(Line::default());
    }
    put(frame, th, inner, lines);
}

fn command_first(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [field, list] = Layout::vertical([Constraint::Length(3), Constraint::Min(4)]).areas(area);
    let fp = panel(th, "Do something", stage, ms, motion);
    let fi = fp.block().inner(field);
    frame.render_widget(fp, field);
    Field::new(th, "", "web res")
        .focused(true)
        .blink(ms / 16)
        .render(fi, frame.buffer_mut());
    // The stack name is a column of its own, so every `·` stands under
    // the one above it [fix-64].
    let commands = [
        ("web", "restart the stack"),
        ("web", "redeploy from the manifest"),
        ("web", "open the log"),
        ("media", "restart the stack"),
    ];
    let column = kp_tui::label_column(th, &["web", "media"]);
    let lp = panel(th, "3 matches", stage, ms, motion);
    let li = lp.block().inner(list);
    frame.render_widget(lp, list);
    let matched: Vec<Line<'static>> = commands
        .iter()
        .map(|(stack, what)| Line::from(format!("{stack:<column$} · {what}")))
        .collect();
    SelectList::new(th, &matched, 0).render(li, frame.buffer_mut());
}

// ── the dashboard ───────────────────────────────────────────────────────

fn hud_corners(frame: &mut Frame, th: &Theme, area: Rect, ms: u32, motion: Motion) {
    Surface::new(th, th.id.palette().background)
        .at(ms, motion)
        .paint(area, frame.buffer_mut());
    let corner = |frame: &mut Frame, x: u16, y: u16, mark: &str, lines: Vec<Line<'static>>| {
        let w = 26u16.min(area.width / 2);
        let at = Rect {
            x,
            y,
            width: w,
            height: 3.min(area.height),
        };
        Paragraph::new(
            [
                vec![Line::from(Span::styled(
                    mark.to_string(),
                    Style::new().fg(th.c.ring),
                ))],
                lines,
            ]
            .concat(),
        )
        .style(Style::new().fg(th.c.foreground).bg(th.c.background))
        .render(at, frame.buffer_mut());
    };
    let right = area.x + area.width.saturating_sub(26);
    let bottom = area.y + area.height.saturating_sub(3);
    corner(
        frame,
        area.x,
        area.y,
        "⌜",
        vec![
            Line::from(Span::styled("pve-01", ink(th, Tone::Success))),
            Line::from(Span::styled("online 41 days", muted(th))),
        ],
    );
    corner(
        frame,
        right,
        area.y,
        "⌝",
        vec![
            Line::from(Span::styled("64% memory", ink(th, Tone::Warning))),
            Line::from(Span::styled("3.40 of 8 load", muted(th))),
        ],
    );
    corner(
        frame,
        area.x,
        bottom,
        "⌞",
        vec![Line::from(Span::styled(
            "5 containers, 1 down",
            ink(th, Tone::Danger),
        ))],
    );
    corner(
        frame,
        right,
        bottom,
        "⌟",
        vec![Line::from(Span::styled("2 transfers running", muted(th)))],
    );
    // The middle carries what is happening, and nothing when nothing is.
    let mid = Rect {
        x: area.x + area.width / 4,
        y: area.y + area.height / 2 - 1,
        width: area.width / 2,
        height: 2.min(area.height),
    };
    Paragraph::new(vec![
        Line::from(Span::styled(
            format!("{} deploying media", spinner(th, ms, motion)),
            Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("gates passed, pulling images", muted(th))),
    ])
    .style(Style::new().fg(th.c.foreground).bg(th.c.background))
    .render(mid, frame.buffer_mut());
}

fn sparks_behind(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Host", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let rows = Layout::vertical([Constraint::Length(3); 4]).split(inner);
    for (i, (label, value, tone)) in [
        ("load", "3.40 of 8", Tone::None),
        ("memory", "64%", Tone::Warning),
        ("disk", "61%", Tone::None),
        ("network", "41 MB/s", Tone::None),
    ]
    .into_iter()
    .enumerate()
    {
        let Some(r) = rows.get(i) else { break };
        let [head, strip] = Layout::vertical([Constraint::Length(1); 2]).areas(*r);
        put(
            frame,
            th,
            head,
            vec![Line::from(vec![
                Span::styled(format!("{label:<9}"), muted(th)),
                Span::styled(
                    value.to_string(),
                    ink(th, tone).add_modifier(Modifier::BOLD),
                ),
            ])],
        );
        Spark::new(th, &history(i as f64 * 5.0))
            .max(6.0)
            .render(strip, frame.buffer_mut());
    }
}

fn big_figures(frame: &mut Frame, th: &Theme, area: Rect) {
    // Half blocks, three rows tall: a figure that reads from a desk away.
    const GLYPHS: [(&str, [&str; 3]); 5] = [
        ("6", ["▛▀▀", "▙▄▖", "▙▄▟"]),
        ("4", ["▌ ▌", "▙▄▟", "  ▌"]),
        ("%", ["▛▖▗", "  ▘ ", "▗▖▟"]),
        ("1", [" ▛", "  ▌", "  ▌"]),
        (".", ["   ", "   ", " ▖ "]),
    ];
    let glyph = |c: char| -> [&'static str; 3] {
        GLYPHS
            .iter()
            .find(|(g, _)| g.starts_with(c))
            .map(|(_, rows)| *rows)
            .unwrap_or(["   ", "   ", "   "])
    };
    let rows = Layout::vertical([Constraint::Length(4); 3]).split(area);
    for (i, (text, label, tone)) in [
        ("64%", "memory", Tone::Warning),
        ("61%", "disk", Tone::None),
        ("4", "containers up", Tone::Success),
    ]
    .into_iter()
    .enumerate()
    {
        let Some(r) = rows.get(i) else { break };
        let mut lines: Vec<Line<'static>> = Vec::new();
        for line in 0..3 {
            let mut spans = Vec::new();
            for c in text.chars() {
                spans.push(Span::styled(
                    format!("{} ", glyph(c)[line]),
                    ink(th, tone).add_modifier(Modifier::BOLD),
                ));
            }
            if line == 1 {
                spans.push(Span::styled(format!("  {label}"), muted(th)));
            }
            lines.push(Line::from(spans));
        }
        put(frame, th, *r, lines);
    }
}

fn rings(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Capacity", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    // A ring in braille: eight positions around a circle, filled clockwise.
    const AROUND: [&str; 8] = ["⠁", "⠈", "⠐", "⠠", "⢀", "⡀", "⠄", "⠂"];
    let cols = Layout::horizontal([Constraint::Ratio(1, 4); 4]).split(inner);
    for (i, (label, value, tone)) in [
        ("memory", 0.64f32, Tone::Warning),
        ("disk", 0.61, Tone::None),
        ("swap", 0.08, Tone::Success),
        ("load", 0.42, Tone::None),
    ]
    .into_iter()
    .enumerate()
    {
        let Some(c) = cols.get(i) else { break };
        let lit = (value * 8.0).round() as usize;
        let ring: Vec<Span<'static>> = AROUND
            .iter()
            .enumerate()
            .map(|(n, g)| {
                Span::styled(
                    g.to_string(),
                    if n < lit { ink(th, tone) } else { muted(th) },
                )
            })
            .collect();
        put(
            frame,
            th,
            *c,
            vec![
                Line::from(ring),
                Line::from(Span::styled(
                    format!("{:>3.0}%", value * 100.0),
                    ink(th, tone).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(label.to_string(), muted(th))),
            ],
        );
    }
}

fn alarms_first(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [loud, quiet] = Layout::vertical([Constraint::Length(7), Constraint::Min(3)]).areas(area);
    let p = panel(th, "Needs you", stage, ms, motion);
    let inner = p.block().inner(loud);
    frame.render_widget(p, loud);
    put(
        frame,
        th,
        inner,
        vec![
            Line::from(vec![
                Badge::state(th, Tone::Danger),
                Span::styled(
                    " backup is down".to_string(),
                    ink(th, Tone::Danger).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "  restic: snapshot failed, lock is 41 minutes old",
                muted(th),
            )),
            Line::default(),
            Line::from(vec![
                Badge::state(th, Tone::Warning),
                Span::styled(
                    " memory at 64%, allocation at 134%".to_string(),
                    ink(th, Tone::Warning),
                ),
            ]),
        ],
    );
    put(
        frame,
        th,
        quiet,
        vec![
            Line::from(Span::styled(
                "  media · web · monitoring · dns all up",
                muted(th),
            )),
            Line::from(Span::styled(
                "  disk 61%, load 3.40 of 8, 2 transfers",
                muted(th),
            )),
        ],
    );
}

// ── the settings ────────────────────────────────────────────────────────

fn two_columns(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Host settings", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let [left, right] =
        Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).areas(inner);
    let settings = [
        (
            "nightly run",
            "03:00",
            "backup and auto-updates for every managed stack at this hour",
        ),
        (
            "keep daily",
            "for 7 days",
            "one snapshot a day for the first week",
        ),
        (
            "keep fortnightly",
            "for 60 days",
            "then one every two weeks, for two months",
        ),
        (
            "keep bimonthly",
            "forever",
            "and one every two months, kept for good",
        ),
        (
            "webhook",
            "off",
            "one POST per finished operation, to a Home Assistant hook",
        ),
    ];
    let mut names = Vec::new();
    let mut helps = Vec::new();
    for (i, (label, value, help)) in settings.into_iter().enumerate() {
        let c = Choice::new(th, label, value).focused(i == 0);
        names.push(Line::from([c.label_spans(18), c.value_spans()].concat()));
        helps.push(Line::from(Span::styled(help.to_string(), muted(th))));
    }
    put(frame, th, left, names);
    put(frame, th, right, helps);
}

fn live_preview(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Retention", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let c = Choice::new(th, "keep daily", "for 7 days").focused(true);
    let mut lines = vec![
        Line::from([c.label_spans(18), c.value_spans()].concat()),
        Line::default(),
        Line::from(Span::styled(
            "  the next sixty days, as this policy keeps them:",
            muted(th),
        )),
        Line::default(),
    ];
    // A strip of days, a mark where a snapshot survives.
    let mut strip = vec![Span::raw("  ")];
    for day in 0..60u32 {
        let kept = day < 7 || (day < 30 && day.is_multiple_of(14)) || day.is_multiple_of(30);
        strip.push(Span::styled(
            if kept { "▊" } else { "·" }.to_string(),
            if kept {
                ink(th, Tone::Success)
            } else {
                muted(th)
            },
        ));
    }
    lines.push(Line::from(strip));
    lines.push(Line::from(Span::styled(
        "  today                                                     in sixty days",
        muted(th),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  11 snapshots kept, 49 pruned, 2.1 GB freed",
        ink(th, Tone::None),
    )));
    put(frame, th, inner, lines);
}

fn grouped_cards(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(area);
    type Group = (&'static str, [(&'static str, &'static str); 2], bool);
    let groups: [Group; 3] = [
        (
            "Schedule",
            [("nightly run", "03:00"), ("window", "2 hours")],
            true,
        ),
        (
            "Retention",
            [("daily", "7 days"), ("fortnightly", "60 days")],
            false,
        ),
        (
            "Notify",
            [("webhook", "off"), ("on failure", "always")],
            false,
        ),
    ];
    for (i, (title, rows, focused)) in groups.into_iter().enumerate() {
        let Some(c) = cols.get(i) else { break };
        let p = Panel::new(th, title)
            .focused(focused)
            .stage(stage)
            .reveal(ms, motion);
        let inner = p.block().inner(*c);
        frame.render_widget(p, *c);
        let lines: Vec<Line<'static>> = rows
            .into_iter()
            .enumerate()
            .map(|(n, (label, value))| {
                let ch = Choice::new(th, label, value).focused(focused && n == 0);
                Line::from([ch.label_spans(13), ch.value_spans()].concat())
            })
            .collect();
        put(frame, th, inner, lines);
    }
}

fn diff(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "2 unsaved changes", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let rows: [(&str, &str, &str); 4] = [
        ("nightly run", "04:00", "03:00"),
        ("keep daily", "7 days", "7 days"),
        ("keep fortnightly", "60 days", "90 days"),
        ("webhook", "off", "off"),
    ];
    let mut lines = vec![Line::from(vec![
        Span::styled(format!("{:<20}", ""), muted(th)),
        Span::styled(format!("{:<16}", "on the host"), muted(th)),
        Span::styled("would become", muted(th)),
    ])];
    for (label, saved, wanted) in rows {
        let changed = saved != wanted;
        lines.push(Line::from(vec![
            Span::styled(format!("{label:<20}"), muted(th)),
            Span::styled(format!("{saved:<16}"), ink(th, Tone::None)),
            Span::styled(
                if changed { wanted } else { "—" }.to_string(),
                if changed {
                    ink(th, Tone::Warning).add_modifier(Modifier::BOLD)
                } else {
                    muted(th)
                },
            ),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "S applies both; R reloads the host's own values",
        muted(th),
    )));
    put(frame, th, inner, lines);
}

fn one_at_a_time(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [crumbs, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(4)]).areas(area);
    Paragraph::new(Stepper::new(th, &["schedule", "retention", "notify", "review"], 1).line())
        .style(Style::new().fg(th.c.foreground).bg(th.c.background))
        .render(crumbs, frame.buffer_mut());
    let p = panel(th, "How long do you keep a snapshot?", stage, ms, motion);
    let inner = p.block().inner(body);
    frame.render_widget(p, body);
    let c = Choice::new(th, "", "for 7 days").focused(true);
    put(
        frame,
        th,
        inner,
        vec![
            Line::default(),
            Line::from(
                [
                    vec![Span::raw("   ")],
                    c.value_spans(),
                    vec![Span::styled("   of the daily snapshots", muted(th))],
                ]
                .concat(),
            ),
            Line::default(),
            Line::from(Span::styled(
                "   A week of dailies costs about 2.1 GB on this host.",
                muted(th),
            )),
            Line::from(Span::styled(
                "   Shorter frees space; longer lets you go further back.",
                muted(th),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("   ←→ ", Style::new().fg(th.c.primary)),
                Span::styled("change   ", muted(th)),
                Span::styled("enter ", Style::new().fg(th.c.primary)),
                Span::styled("next", muted(th)),
            ]),
        ],
    );
}

// ── the log stream ──────────────────────────────────────────────────────

/// The fixture every log direction reads: time, source, severity, message.
const LINES: [(&str, &str, Tone, &str); 8] = [
    (
        "09:41:02",
        "media",
        Tone::Info,
        "jellyfin: transcode worker ready",
    ),
    (
        "09:41:02",
        "web",
        Tone::Info,
        "caddy: certificate renewed, 89 days left",
    ),
    (
        "09:41:03",
        "backup",
        Tone::Warning,
        "restic: repository locked by another process",
    ),
    (
        "09:41:03",
        "monitoring",
        Tone::Info,
        "prometheus: scrape took 412 ms",
    ),
    (
        "09:41:04",
        "dns",
        Tone::Info,
        "blocky: 1284 queries, 19 % blocked",
    ),
    (
        "09:41:05",
        "backup",
        Tone::Danger,
        "restic: snapshot failed, lock is 41 minutes old",
    ),
    (
        "09:41:05",
        "media",
        Tone::MutedInk,
        "jellyfin: cache hit for /Items/1a2b",
    ),
    (
        "09:41:06",
        "web",
        Tone::Info,
        "caddy: 200 GET / 2.1 kB in 3 ms",
    ),
];

fn log_line(th: &Theme, i: usize) -> Line<'static> {
    let (time, source, tone, message) = LINES[i % LINES.len()];
    Line::from(vec![
        Span::styled(format!("{time} "), muted(th)),
        Span::styled(
            format!("{source:<11}"),
            Style::new().fg(source_colour(th, source)),
        ),
        Span::styled(message.to_string(), ink(th, tone)),
    ])
}

fn density_band(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Log stream", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let [band, rows] = Layout::vertical([Constraint::Length(3), Constraint::Min(2)]).areas(inner);
    let [chart, axis] =
        Layout::vertical([Constraint::Length(2), Constraint::Length(1)]).areas(band);
    Spark::new(th, &history(9.0))
        .max(6.0)
        .render(chart, frame.buffer_mut());
    // The errors on the same band, in their own tone.
    let mut marks = Vec::new();
    for x in 0..axis.width {
        let err = x.is_multiple_of(17) && x > 0;
        marks.push(Span::styled(
            if err { "▲" } else { "─" }.to_string(),
            if err {
                ink(th, Tone::Danger)
            } else {
                muted(th)
            },
        ));
    }
    put(frame, th, axis, vec![Line::from(marks)]);
    put(
        frame,
        th,
        rows,
        (0..rows.height as usize).map(|i| log_line(th, i)).collect(),
    );
}

fn grouped_sources(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    ms: u32,
    motion: Motion,
) {
    let p = panel(th, "Log stream · by source", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let mut lines = Vec::new();
    for (name, count, worst, open) in [
        ("backup", 41u32, Tone::Danger, true),
        ("media", 118, Tone::Info, false),
        ("web", 96, Tone::Info, false),
        ("monitoring", 12, Tone::Warning, false),
        ("dns", 204, Tone::Info, false),
    ] {
        lines.push(Line::from(vec![
            Span::styled(
                if open { "▾ " } else { "▸ " }.to_string(),
                Style::new().fg(th.c.border_strong),
            ),
            Span::styled("▎", Style::new().fg(source_colour(th, name))),
            Span::styled(format!("{name:<12}"), ink(th, Tone::None)),
            Span::styled(format!("{count:>4} lines  "), muted(th)),
            Badge::state(th, worst),
        ]));
        if open {
            for i in 2..4 {
                lines.push(Line::from(
                    [vec![Span::raw("    ")], log_line(th, i).spans].concat(),
                ));
            }
        }
    }
    put(frame, th, inner, lines);
}

fn severity_focus(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    ms: u32,
    motion: Motion,
) {
    let p = panel(th, "Log stream · by weight", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let mut lines: Vec<Line<'static>> = Vec::new();
    for (time, source, tone, message) in LINES {
        match tone {
            Tone::Danger => {
                let plate = th.tone(Tone::Danger).unwrap_or(th.c.destructive);
                lines.push(Line::from(Span::styled(
                    format!(" {source} · {time} "),
                    Style::new()
                        .fg(th.on_plate(th.id.palette().destructive))
                        .bg(plate)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("   {message}"),
                    ink(th, Tone::Danger),
                )));
            }
            Tone::Warning => lines.push(Line::from(vec![
                Span::styled("  ! ", ink(th, Tone::Warning)),
                Span::styled(message.to_string(), ink(th, Tone::Warning)),
            ])),
            Tone::MutedInk => {
                lines.push(Line::from(Span::styled(format!("  · {source}"), muted(th))))
            }
            _ => lines.push(Line::from(vec![
                Span::styled("    ", muted(th)),
                Span::styled(message.to_string(), ink(th, Tone::None)),
            ])),
        }
    }
    put(frame, th, inner, lines);
}

fn time_gutter(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Log stream", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let mut lines = Vec::new();
    for i in 0..inner.height as usize {
        // A tick where a second starts, a gap where nothing arrived.
        let (mark, quiet) = match i % 5 {
            0 => ("├", false),
            3 => ("╎", true),
            _ => ("│", false),
        };
        let (_, source, tone, message) = LINES[i % LINES.len()];
        lines.push(Line::from(vec![
            Span::styled(
                format!("{mark} "),
                if quiet {
                    muted(th)
                } else {
                    Style::new().fg(th.c.border_strong)
                },
            ),
            Span::styled(
                format!("{source:<11}"),
                Style::new().fg(source_colour(th, source)),
            ),
            Span::styled(message.to_string(), ink(th, tone)),
        ]));
    }
    put(frame, th, inner, lines);
}

fn stream_and_detail(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    ms: u32,
    motion: Motion,
) {
    let [stream, detail] =
        Layout::vertical([Constraint::Min(4), Constraint::Length(9)]).areas(area);
    let sp = panel(th, "Log stream", stage, ms, motion);
    let si = sp.block().inner(stream);
    frame.render_widget(sp, stream);
    let items: Vec<Line<'static>> = (0..si.height as usize).map(|i| log_line(th, i)).collect();
    SelectList::new(th, &items, 5).render(si, frame.buffer_mut());

    let dp = panel(th, "restic: snapshot failed", stage, ms, motion);
    let di = dp.block().inner(detail);
    frame.render_widget(dp, detail);
    Facts::new(
        th,
        &[
            ("unit", "restic-backup.service".into(), Tone::None),
            ("severity", "error".into(), Tone::Danger),
            ("pid", "18422".into(), Tone::None),
            ("exit", "1".into(), Tone::Danger),
            ("since", "41 minutes".into(), Tone::Warning),
            ("retries", "3 of 3".into(), Tone::None),
        ],
    )
    .columns(2)
    .render(di, frame.buffer_mut());
}

// ── the deploy window ───────────────────────────────────────────────────

const STEPS: [(&str, &str, Tone); 5] = [
    ("sync", "4.2 s · 118 files", Tone::Success),
    ("gates", "6 of 6 passed", Tone::Success),
    ("pull", "2 images", Tone::Success),
    ("up", "recreating jellyfin", Tone::Info),
    ("health", "waiting", Tone::MutedInk),
];

fn step_line(th: &Theme, i: usize, ms: u32, motion: Motion) -> Line<'static> {
    let (name, note, tone) = STEPS[i];
    let mark = match tone {
        Tone::Success => "✓".to_string(),
        Tone::Info => spinner(th, ms, motion).to_string(),
        _ => "·".to_string(),
    };
    Line::from(vec![
        Span::styled(format!(" {mark} "), ink(th, tone)),
        Span::styled(
            format!("{name:<9}"),
            ink(th, Tone::None).add_modifier(Modifier::BOLD),
        ),
        Span::styled(note.to_string(), muted(th)),
    ])
}

fn step_rail(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let p = panel(th, "Deploy media", stage, ms, motion);
    let inner = p.block().inner(area);
    frame.render_widget(p, area);
    let [steps, feed] = Layout::vertical([Constraint::Length(6), Constraint::Min(2)]).areas(inner);
    put(
        frame,
        th,
        steps,
        (0..STEPS.len())
            .map(|i| step_line(th, i, ms, motion))
            .collect(),
    );
    put(
        frame,
        th,
        feed,
        vec![
            Line::from(Span::styled("  jellyfin recreated", muted(th))),
            Line::from(Span::styled("  jellyfin-exporter recreated", muted(th))),
            Line::from(Span::styled("  waiting for health…", muted(th))),
        ],
    );
}

fn two_panes(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(34), Constraint::Min(30)]).areas(area);
    let lp = panel(th, "Steps", stage, ms, motion);
    let li = lp.block().inner(left);
    frame.render_widget(lp, left);
    put(
        frame,
        th,
        li,
        (0..STEPS.len())
            .map(|i| step_line(th, i, ms, motion))
            .collect(),
    );
    let rp = panel(th, "Transcript", stage, ms, motion);
    let ri = rp.block().inner(right);
    frame.render_widget(rp, right);
    put(
        frame,
        th,
        ri,
        vec![
            Line::from(Span::styled(
                "[sync][run ] rsync -a --delete ./stacks/media/",
                ink(th, Tone::Info),
            )),
            Line::from(Span::styled(
                "[sync][exit] 0 in 4.2 s, 118 files",
                ink(th, Tone::Success),
            )),
            Line::from(Span::styled(
                "[gate] compose config — valid",
                ink(th, Tone::Success),
            )),
            Line::from(Span::styled(
                "[run ] docker compose up -d",
                ink(th, Tone::Info),
            )),
            Line::from(Span::styled("jellyfin recreated", muted(th))),
        ],
    );
}

fn gate_cards(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let gates: [(&str, &str, Tone); 4] = [
        ("whitelist", "3 of 3 images allowed", Tone::Success),
        ("hostname guard", "pve-01 matches", Tone::Success),
        ("fail closed", "rollback armed", Tone::Success),
        ("digests pinned", "1 of 6 floating", Tone::Danger),
    ];
    let cols = Layout::horizontal([Constraint::Ratio(1, 2); 2]).split(area);
    for (i, (name, note, tone)) in gates.into_iter().enumerate() {
        let col = cols[i % 2];
        let card = Rect {
            y: col.y + (i / 2) as u16 * 5,
            height: 5.min(col.height),
            ..col
        };
        if card.y + card.height > area.y + area.height {
            break;
        }
        let p = Panel::new(th, name)
            .focused(tone == Tone::Danger)
            .stage(stage)
            .reveal(ms, motion);
        let inner = p.block().inner(card);
        frame.render_widget(p, card);
        put(
            frame,
            th,
            inner,
            vec![
                Line::from(vec![
                    Badge::state(th, tone),
                    Span::styled(
                        if tone == Tone::Danger {
                            " blocked"
                        } else {
                            " passed"
                        }
                        .to_string(),
                        ink(th, tone).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(Span::styled(format!("  {note}"), muted(th))),
            ],
        );
    }
}

fn full_status(frame: &mut Frame, th: &Theme, area: Rect, ms: u32, motion: Motion) {
    // The transcript behind, at a tenth of its weight.
    put(
        frame,
        th,
        area,
        (0..area.height)
            .map(|i| {
                Line::from(Span::styled(
                    format!("  [sync][exit] step {i} :: ok"),
                    Style::new().fg(th.c.border),
                ))
            })
            .collect(),
    );
    let word = "LIVE";
    let mid = Rect {
        x: area.x + area.width / 2 - 12,
        y: area.y + area.height / 2 - 2,
        width: 26.min(area.width),
        height: 3.min(area.height),
    };
    Paragraph::new(vec![
        Line::from(Span::styled(
            format!("{} {word}", spinner(th, ms, motion)),
            Style::new()
                .fg(th.c.primary)
                .bg(th.c.card)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("deploy media", muted(th).bg(th.c.card))),
    ])
    .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
    .render(mid, frame.buffer_mut());
}

fn docked_bar(frame: &mut Frame, th: &Theme, area: Rect, stage: Stage, ms: u32, motion: Motion) {
    let [screen, bar] = Layout::vertical([Constraint::Min(4), Constraint::Length(2)]).areas(area);
    // Whatever you were on keeps the screen.
    let p = panel(th, "Lxc mesh · 5 nodes", stage, ms, motion);
    let inner = p.block().inner(screen);
    frame.render_widget(p, screen);
    let columns = [
        Column::new("node", 16),
        Column::new("status", 8),
        Column::new("apps", 5).right(),
    ];
    let rows: Vec<Vec<Line<'static>>> = FLEET
        .iter()
        .map(|s| {
            vec![
                Line::from(vec![
                    Span::styled("▎", Style::new().fg(source_colour(th, s.name))),
                    Span::styled(s.name.to_string(), ink(th, Tone::None)),
                ]),
                Line::from(vec![
                    Badge::dot(th, s.online),
                    Span::styled(if s.online { " up" } else { " down" }, muted(th)),
                ]),
                Line::from(Span::styled(
                    format!("{}/{}", s.apps.iter().filter(|a| a.1).count(), s.apps.len()),
                    ink(th, Tone::None),
                )),
            ]
        })
        .collect();
    DataTable::new(th, &columns, &rows).render(inner, frame.buffer_mut());

    let [line, flow] = Layout::vertical([Constraint::Length(1); 2]).areas(bar);
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spinner(th, ms, motion)),
            Style::new().fg(th.c.primary),
        ),
        Span::styled(
            "deploy media",
            ink(th, Tone::None).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  up · recreating jellyfin", muted(th)),
        Span::styled("   1m 12s", muted(th)),
        Span::styled("   enter to open", Style::new().fg(th.c.primary)),
    ]))
    .style(Style::new().fg(th.c.foreground).bg(th.c.secondary))
    .render(line, frame.buffer_mut());
    Stream::new(th, ms, motion).render(flow, frame.buffer_mut());
}
