//! The four things homelab's TUI writes out by hand, most often.
//!
//! `docs/HOMELAB_INVENTORY.md` counted them in the first consumer: the
//! popup seven times, the text field five, the percentage bar twice, and a
//! key-hint footer whose keymap the help overlay repeats independently.
//! Each one here takes a `&Theme` and paints in that theme's own marks, so
//! the same screen reads as a phosphor CRT in terminal and as a HUD in
//! cyberpunk without a second palette.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::anatomy::{ButtonFace, Rule, Texture, Tone};
use crate::color::{ColorDepth, Rgb, Role};
use crate::effects::{self, mix};
use crate::fx::Motion;
use crate::logs::{LogBuffer, Severity};
use crate::theme::Theme;
use crate::widgets::Panel;
use ratatui::style::Color;

/// How much of the screen a popup takes, and what it means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupKind {
    /// A question or a form: the theme's focus frame.
    Normal,
    /// Something that destroys: the destructive colour on the frame, which
    /// is homelab's red double border at `view/mod.rs:104`.
    Danger,
}

/// A centred overlay: the ground cleared, the theme's frame around it, and
/// the inner rect for whatever goes inside.
///
/// Replaces the seven copies of "compute a centred Rect, render `Clear`,
/// draw a bordered block, take `.inner()`".
pub struct Popup<'a> {
    theme: &'a Theme,
    title: &'a str,
    kind: PopupKind,
    /// Width and height in cells, clamped to the area with a one-cell margin.
    size: (u16, u16),
}

impl<'a> Popup<'a> {
    pub fn new(theme: &'a Theme, title: &'a str, size: (u16, u16)) -> Self {
        Popup {
            theme,
            title,
            kind: PopupKind::Normal,
            size,
        }
    }

    pub fn kind(mut self, kind: PopupKind) -> Self {
        self.kind = kind;
        self
    }

    /// Where the popup lands, given the screen. Public because a caller
    /// needs it to know where its own content goes.
    pub fn area(&self, screen: Rect) -> Rect {
        let w = self.size.0.min(screen.width.saturating_sub(2));
        let h = self.size.1.min(screen.height.saturating_sub(2));
        Rect {
            x: screen.x + (screen.width.saturating_sub(w)) / 2,
            y: screen.y + (screen.height.saturating_sub(h)) / 2,
            width: w,
            height: h,
        }
    }

    /// The rect inside the frame, where the caller draws.
    pub fn inner(&self, screen: Rect) -> Rect {
        let area = self.area(screen);
        Panel::new(self.theme, self.title)
            .focused(true)
            .block()
            .inner(area)
    }

    /// Draw the overlay; returns the inner rect so the caller can fill it.
    pub fn render_over(self, screen: Rect, buf: &mut Buffer) -> Rect {
        let area = self.area(screen);
        // Depth, before anything is drawn: the page behind a dialog steps
        // back, and the dialog throws a shadow onto it. Both are one pass
        // over the buffer and neither needs a colour of its own.
        scrim(self.theme, screen, buf);
        Clear.render(area, buf);
        shadow(self.theme, area, screen, buf);
        let t = &self.theme.c;
        // The ground under a popup is the theme's popover surface, not the
        // page: a dialog that sits on the same colour as the page behind it
        // is a dialog nobody can see the edge of.
        let panel = Panel::new(self.theme, self.title).focused(true);
        let inner = panel.block().inner(area);
        panel.render(area, buf);
        // The panel paints a card; a dialog sits on `--popover`, which is
        // the surface the registers raise above a card. Repainted after the
        // frame so the frame keeps its own colours.
        let ground = Style::new().bg(t.popover).fg(t.popover_foreground);
        for y in inner.y..inner.bottom() {
            for x in inner.x..inner.right() {
                buf[(x, y)].set_symbol(" ");
                buf[(x, y)].set_style(ground);
            }
        }
        if self.kind == PopupKind::Danger {
            // The frame turns destructive, in place: same glyphs, the colour
            // that says what this dialog does.
            let danger = Style::new()
                .fg(self
                    .theme
                    .ink(Tone::Danger, self.theme.id.palette().popover))
                .bg(t.popover);
            for x in area.x..area.right() {
                buf[(x, area.y)].set_style(danger);
                buf[(x, area.bottom() - 1)].set_style(danger);
            }
            for y in area.y..area.bottom() {
                buf[(area.x, y)].set_style(danger);
                buf[(area.right() - 1, y)].set_style(danger);
            }
        }
        inner
    }
}

/// A single-line text field with the theme's own caret.
///
/// homelab draws this five times, each with its own blinking block. The
/// caret here is the theme's: a block where the anatomy asks for a block,
/// a bar where it asks for a bar, and it only blinks where the theme's
/// cursor blinks.
pub struct Field<'a> {
    theme: &'a Theme,
    label: &'a str,
    value: &'a str,
    focused: bool,
    /// Ticks since the caret's clock started; `None` holds it steady.
    blink: Option<u32>,
}

impl<'a> Field<'a> {
    pub fn new(theme: &'a Theme, label: &'a str, value: &'a str) -> Self {
        Field {
            theme,
            label,
            value,
            focused: false,
            blink: None,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn blink(mut self, ticks: u32) -> Self {
        self.blink = Some(ticks);
        self
    }

    /// Whether the caret is drawn this tick. A steady cursor is always on;
    /// a blinking one is on for half of each second, which is the rate
    /// `themes/retro/anatomy.md` states and the others inherit.
    fn caret_on(&self) -> bool {
        use crossterm::cursor::SetCursorStyle::{BlinkingBar, BlinkingBlock, BlinkingUnderScore};
        let blinking = matches!(
            self.theme.a.cursor,
            BlinkingBlock | BlinkingBar | BlinkingUnderScore
        );
        match (blinking, self.blink) {
            (false, _) | (true, None) => true,
            (true, Some(ticks)) => (ticks / 15) % 2 == 0,
        }
    }

    /// The caret's own glyph, from the theme's cursor style.
    fn caret(&self) -> &'static str {
        use crossterm::cursor::SetCursorStyle::*;
        match self.theme.a.cursor {
            BlinkingBlock | SteadyBlock | DefaultUserShape => "█",
            BlinkingUnderScore | SteadyUnderScore => "_",
            _ => "▏",
        }
    }
}

impl Widget for Field<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (t, a) = (&self.theme.c, self.theme.a);
        let label = if a.uppercase_labels {
            self.label.to_uppercase()
        } else {
            self.label.to_string()
        };
        let mut spans = vec![
            Span::styled(
                format!("{}{}: ", a.label_prefix, label),
                Style::new().fg(t.muted_foreground),
            ),
            Span::styled(self.value.to_string(), Style::new().fg(t.foreground)),
        ];
        if self.focused && self.caret_on() {
            spans.push(Span::styled(self.caret(), Style::new().fg(t.ring)));
        }
        let style = if self.focused {
            Style::new().bg(t.card)
        } else {
            Style::new().bg(t.background)
        };
        Paragraph::new(Line::from(spans))
            .style(style)
            .render(area, buf);
    }
}

/// A percentage bar in the theme's colours, with its own thresholds.
///
/// homelab builds this twice out of block characters, and colours it by
/// hand at 70 % and 90 %. Here the thresholds are the caller's and the
/// colours are the theme's.
pub struct Meter<'a> {
    theme: &'a Theme,
    label: &'a str,
    /// 0.0 to 1.0.
    value: f32,
    warn_at: f32,
    danger_at: f32,
}

impl<'a> Meter<'a> {
    pub fn new(theme: &'a Theme, label: &'a str, value: f32) -> Self {
        Meter {
            theme,
            label,
            value: value.clamp(0.0, 1.0),
            warn_at: 0.7,
            danger_at: 0.9,
        }
    }

    pub fn thresholds(mut self, warn_at: f32, danger_at: f32) -> Self {
        (self.warn_at, self.danger_at) = (warn_at, danger_at);
        self
    }

    /// The colour the fill takes at this value.
    pub fn colour(&self) -> ratatui::style::Color {
        let t = &self.theme.c;
        if self.value >= self.danger_at {
            t.destructive
        } else if self.value >= self.warn_at {
            t.warning
        } else {
            t.primary
        }
    }
}

impl Widget for Meter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height == 0 {
            return;
        }
        let (t, a) = (&self.theme.c, self.theme.a);
        let label = if a.uppercase_labels {
            self.label.to_uppercase()
        } else {
            self.label.to_string()
        };
        let reading = format!("{:>3.0}%", self.value * 100.0);
        // label … bar … reading, with the bar taking what is left.
        let text_w = label.chars().count() as u16 + reading.chars().count() as u16 + 2;
        let bar_w = area.width.saturating_sub(text_w);
        // An eighth of a cell at a time: the bar lands on the value it was
        // given instead of the nearest whole cell, which at forty cells is
        // eight times the precision for nothing.
        let exact = bar_w as f32 * self.value;
        let filled = exact.floor() as u16;
        let eighths = ((exact - filled as f32) * 8.0).round() as usize;
        const PART: [&str; 9] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];
        let fill = self.colour();
        let mut spans = vec![Span::styled(
            format!("{label} "),
            Style::new().fg(t.muted_foreground),
        )];
        // Painted as background rather than block glyphs: a solid bar reads
        // at any font, and the theme's own colour does the work.
        spans.push(Span::styled(
            " ".repeat(filled as usize),
            Style::new().bg(fill),
        ));
        let mut rest = bar_w.saturating_sub(filled) as usize;
        if eighths > 0 && rest > 0 {
            spans.push(Span::styled(
                PART[eighths].to_string(),
                Style::new().fg(fill).bg(t.muted),
            ));
            rest -= 1;
        }
        spans.push(Span::styled(" ".repeat(rest), Style::new().bg(t.muted)));
        spans.push(Span::styled(
            format!(" {reading}"),
            Style::new().fg(if self.value >= self.danger_at {
                self.theme.ink(Tone::Danger, self.theme.id.palette().card)
            } else {
                t.foreground
            }),
        ));
        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}

/// One keymap, drawn two ways.
///
/// homelab keeps its footer hints and its help overlay in two places that
/// have to agree by hand (`view/mod.rs:668` and `:750`). `KeyHints` takes
/// the list once: `footer` lays it in a row, `overlay` in a column.
pub struct KeyHints<'a> {
    theme: &'a Theme,
    keys: &'a [(&'a str, &'a str)],
}

impl<'a> KeyHints<'a> {
    pub fn new(theme: &'a Theme, keys: &'a [(&'a str, &'a str)]) -> Self {
        KeyHints { theme, keys }
    }

    /// The hints as one line, for the status bar.
    pub fn footer(&self) -> Line<'static> {
        let (t, a) = (&self.theme.c, self.theme.a);
        let key = Style::new().fg(t.primary).add_modifier(Modifier::BOLD);
        let what = Style::new().fg(t.secondary_foreground);
        let mut spans = Vec::new();
        for (i, (k, d)) in self.keys.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    a.tab_divider.to_string(),
                    Style::new().fg(t.border_strong),
                ));
            }
            spans.push(Span::styled((*k).to_string(), key));
            spans.push(Span::styled(format!(" {d}"), what));
        }
        Line::from(spans)
    }

    /// The same hints as rows, for the help overlay.
    pub fn overlay(&self) -> Vec<Line<'static>> {
        let (t, a) = (&self.theme.c, self.theme.a);
        let width = self
            .keys
            .iter()
            .map(|(k, _)| k.chars().count())
            .max()
            .unwrap_or(0);
        self.keys
            .iter()
            .map(|(k, d)| {
                let label = if a.uppercase_labels {
                    d.to_uppercase()
                } else {
                    (*d).to_string()
                };
                Line::from(vec![
                    Span::styled(
                        format!("{k:>width$}  "),
                        Style::new().fg(t.primary).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(label, Style::new().fg(t.foreground)),
                ])
            })
            .collect()
    }
}

impl Widget for KeyHints<'_> {
    /// The footer form: one line on the theme's secondary plate.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = &self.theme.c;
        Paragraph::new(self.footer())
            .alignment(Alignment::Left)
            .style(Style::new().bg(t.secondary).fg(t.secondary_foreground))
            .render(area, buf);
    }
}

// ── The log viewer ──────────────────────────────────────────────────────

/// A colour that belongs to a name, from the theme's own chart hues.
///
/// homelab gives every stack an identity hue so it is recognisable in the
/// table, the list, the source bar and its log lines alike
/// (`client/src/tui/theme.rs:88`). There it is a hand-written match on a
/// hash; here the five chart colours are what a theme already declares for
/// telling series apart, so the hue is the theme's rather than one more
/// literal.
pub fn source_colour(theme: &Theme, name: &str) -> ratatui::style::Color {
    // FNV-1a: stable across runs and machines, which a `DefaultHasher` is
    // not — a source that changed colour between sessions would be worse
    // than no colour at all.
    let mut hash: u32 = 0x811c_9dc5;
    for byte in name.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    let c = &theme.c;
    [c.chart_1, c.chart_2, c.chart_3, c.chart_4, c.chart_5][(hash % 5) as usize]
}

/// The log pane homelab draws three times, with what its three copies have
/// between them — and the scrollbar none of them has.
///
/// Carries: the source selector with each source in its own colour, the
/// severity filter and the follow state in the title, timestamp, host and
/// unit as their own colours, a severity tag, and a scrollbar that says
/// where in the buffer the view sits.
pub struct LogPane<'a> {
    theme: &'a Theme,
    title: &'a str,
    buffer: &'a LogBuffer,
    sources: &'a [&'a str],
    selected: usize,
    /// A live feed says so in the title; a generated one must say that too,
    /// because a demo stream that looks live is a lie a reader acts on.
    live: bool,
}

impl<'a> LogPane<'a> {
    pub fn new(theme: &'a Theme, title: &'a str, buffer: &'a LogBuffer) -> Self {
        LogPane {
            theme,
            title,
            buffer,
            sources: &[],
            selected: 0,
            live: true,
        }
    }

    /// The sources to offer, and which one is selected.
    pub fn sources(mut self, sources: &'a [&'a str], selected: usize) -> Self {
        (self.sources, self.selected) = (sources, selected);
        self
    }

    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    /// The pane's state, for the panel's own status corner: which levels
    /// are shown, and whether the view follows the tail.
    pub fn status(&self) -> Line<'static> {
        let c = &self.theme.c;
        let muted = Style::new().fg(c.muted_foreground);
        let l = self.buffer;
        let filter = if l.filter == Severity::Debug {
            "all levels".to_string()
        } else {
            format!("{} and up", l.filter.name())
        };
        let state = if l.paused() {
            format!("paused, {} new", l.unseen())
        } else {
            "following".to_string()
        };
        let live = if self.live {
            "live"
        } else {
            "synthetic, not real"
        };
        Line::from(vec![
            Span::styled(format!(" {live} · {filter} · "), muted),
            Span::styled(
                state,
                if l.paused() {
                    Style::new()
                        .fg(self.theme.ink(Tone::Warning, self.theme.id.palette().card))
                        .add_modifier(Modifier::BOLD)
                } else {
                    muted
                },
            ),
            Span::styled(" ", muted),
        ])
    }

    /// The source bar: every source in its own colour, the selected one on
    /// its plate. One line; empty when there is nothing to choose between.
    fn source_bar(&self) -> Line<'static> {
        let (c, a) = (&self.theme.c, self.theme.a);
        let mut spans = Vec::new();
        for (i, name) in self.sources.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    a.tab_divider.to_string(),
                    Style::new().fg(c.border_strong),
                ));
            }
            let hue = source_colour(self.theme, name);
            let label = if a.uppercase_labels {
                name.to_uppercase()
            } else {
                (*name).to_string()
            };
            let style = if i == self.selected {
                Style::new()
                    .fg(c.background)
                    .bg(hue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(hue)
            };
            spans.push(Span::styled(format!(" {label} "), style));
        }
        Line::from(spans)
    }
}

impl Widget for LogPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget};
        let (c, a) = (&self.theme.c, self.theme.a);
        let panel = Panel::new(self.theme, self.title).status(self.status());
        let inner = panel.block().inner(area);
        panel.render(area, buf);
        if inner.height == 0 || inner.width < 4 {
            return;
        }
        // The source bar takes the first row when there is more than one
        // source; a single source names itself in the title instead.
        let bar_rows = u16::from(self.sources.len() > 1);
        if bar_rows == 1 {
            let bar = Rect { height: 1, ..inner };
            Paragraph::new(self.source_bar())
                .style(Style::new().bg(c.card))
                .render(bar, buf);
        }
        let body = Rect {
            y: inner.y + bar_rows,
            height: inner.height - bar_rows,
            width: inner.width.saturating_sub(1),
            ..inner
        };
        if body.height == 0 {
            return;
        }
        let height = body.height as usize;
        let lines: Vec<Line> = self
            .buffer
            .visible(height)
            .into_iter()
            .map(|line| {
                let unit_hue = source_colour(self.theme, &line.unit);
                Line::from(vec![
                    Span::styled(
                        format!("{} ", line.time),
                        Style::new().fg(c.muted_foreground),
                    ),
                    Span::styled(format!("{} ", line.host), Style::new().fg(c.border_strong)),
                    Span::styled(format!("{} ", line.unit), Style::new().fg(unit_hue)),
                    Span::styled(
                        line.severity.tag().to_string(),
                        crate::dashboard::severity_style(self.theme, line.severity),
                    ),
                    Span::styled(" ", Style::new()),
                    Span::styled(
                        line.message.clone(),
                        crate::dashboard::message_style(self.theme, line.severity),
                    ),
                ])
            })
            .collect();
        Paragraph::new(lines)
            .style(Style::new().bg(c.card))
            .render(body, buf);

        // The scrollbar homelab's three copies do without: where the view
        // sits in what the filter shows, with the theme's own line colours.
        let total = self.buffer.shown_count();
        let track = Rect {
            x: inner.right() - 1,
            y: body.y,
            width: 1,
            height: body.height,
        };
        let mut state = ScrollbarState::new(total.saturating_sub(height)).position(
            total
                .saturating_sub(height)
                .saturating_sub(self.buffer.scroll().min(total.saturating_sub(height))),
        );
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some(" "))
                .thumb_symbol(if a.uppercase_labels { "█" } else { "▐" })
                .track_style(Style::new().bg(c.card).fg(c.border))
                .thumb_style(Style::new().bg(c.card).fg(c.border_strong)),
            track,
            buf,
            &mut state,
        );
    }
}

// ── The theme's own motion ──────────────────────────────────────────────

/// The ground a register textures, and what crosses it.
///
/// DI9 gives every theme a static texture layer — scanlines, a drafting
/// grid, halftone dots, carbon twill — between 2 % and 6 % alpha. A cell
/// grid cannot carry a 1px line, so the texture here is a tint on whole
/// cells: the same rhythm, at a little under the CSS alpha, in the only
/// resolution a terminal has. A cell row is fourteen pixels where the
/// register's line is one, so the same alpha over the same share of rows
/// reads louder here than in a browser; 4.5 % is where it stops reading as
/// stripes and starts reading as a ground. Sixteen-colour terminals get no texture at all rather
/// than a wrong one.
pub struct Surface<'a> {
    theme: &'a Theme,
    /// The colour the texture sits on — a page's `--background`, a card's
    /// `--card`.
    ground: Rgb,
    elapsed_ms: u32,
    motion: Motion,
    id: u64,
}

impl<'a> Surface<'a> {
    pub fn new(theme: &'a Theme, ground: Rgb) -> Self {
        Surface {
            theme,
            ground,
            elapsed_ms: 0,
            motion: Motion::Full,
            id: 0,
        }
    }

    pub fn at(mut self, elapsed_ms: u32, motion: Motion) -> Self {
        (self.elapsed_ms, self.motion) = (elapsed_ms, motion);
        self
    }

    /// Which panel this is, so two panels do not sweep in lockstep.
    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    /// How strongly a cell is tinted, 0.0 for a cell the texture misses.
    fn weight(&self, dx: u16, dy: u16) -> f32 {
        match self.theme.a.fx.texture {
            Texture::None => 0.0,
            Texture::Scanline { every } if every > 0 => {
                dy.is_multiple_of(every) as u8 as f32 * 0.045
            }
            Texture::Grid { cols, rows } => {
                let col = cols > 0 && dx.is_multiple_of(cols);
                let row = rows > 0 && dy.is_multiple_of(rows);
                if col || row { 0.045 } else { 0.0 }
            }
            Texture::Dots { every } if every > 0 => {
                (dy.is_multiple_of(2) && dx.is_multiple_of(every)) as u8 as f32 * 0.04
            }
            Texture::Diagonal { every } if every > 0 => {
                (dx + dy).is_multiple_of(every) as u8 as f32 * 0.04
            }
            _ => 0.0,
        }
    }

    pub fn paint(self, area: Rect, buf: &mut Buffer) {
        if self.theme.depth == ColorDepth::Ansi16 || area.is_empty() {
            return;
        }
        let p = self.theme.id.palette();
        let lit = self
            .theme
            .a
            .fx
            .sweep
            .and_then(|s| s.row(area.height, self.elapsed_ms, self.id, self.motion));
        for dy in 0..area.height {
            for dx in 0..area.width {
                let w = self.weight(dx, dy);
                // The sweep is brighter than the texture and takes the
                // theme's own primary, which is what `kp-alarm-sweep`
                // paints in `css/cyberpunk-register.css`.
                let (towards, w) = if lit == Some(dy) {
                    (p.primary, 0.22)
                } else if w == 0.0 {
                    continue;
                } else {
                    (p.foreground, w)
                };
                let bg = self
                    .theme
                    .depth
                    .resolve(Role::Surface, mix(self.ground, towards, w));
                buf[(area.x + dx, area.y + dy)].set_bg(bg);
            }
        }
    }
}

/// The theme's attention treatment: a panel that strikes as it arrives,
/// a headline that comes apart, a frame that breathes — whichever of the
/// three that register declares, and nothing for the ones that declare
/// none.
///
/// Homelab paints its own alarm state red and leaves it there
/// (`client/src/tui/view/mod.rs`); here the theme decides, because the
/// package already decided once, per theme, in `research/alarm-per-theme/`.
pub struct AlarmPanel<'a> {
    theme: &'a Theme,
    title: &'a str,
    body: &'a str,
    elapsed_ms: u32,
    motion: Motion,
}

impl<'a> AlarmPanel<'a> {
    pub fn new(theme: &'a Theme, title: &'a str, body: &'a str) -> Self {
        AlarmPanel {
            theme,
            title,
            body,
            elapsed_ms: 0,
            motion: Motion::Full,
        }
    }

    pub fn at(mut self, elapsed_ms: u32, motion: Motion) -> Self {
        (self.elapsed_ms, self.motion) = (elapsed_ms, motion);
        self
    }

    /// The headline as it is drawn this frame, glitched if the theme
    /// glitches. Public so a test can read it without a buffer.
    pub fn headline(&self) -> String {
        let label = self.theme.label(self.title);
        match self.theme.a.fx.alarm.glitch {
            Some(g) => g.text(&label, self.elapsed_ms, 0xA1, self.motion),
            None => label,
        }
    }
}

impl Widget for AlarmPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let th = self.theme;
        let (p, alarm) = (th.id.palette(), th.a.fx.alarm);
        let strike = alarm
            .strike
            .and_then(|s| s.at(self.elapsed_ms, self.motion));
        // The strike's opacity, in a terminal, is how far the ink has come
        // from the plate towards its own colour.
        let t = strike.map(|f| f.ink).unwrap_or(1.0);
        let area = match strike.map(|f| f.kick).unwrap_or(0) {
            k if k < 0 && area.x > 0 => Rect {
                x: area.x - 1,
                ..area
            },
            k if k > 0 => Rect {
                x: area.x + 1,
                width: area.width.saturating_sub(1),
                ..area
            },
            _ => area,
        };
        // The glow: the frame breathing between the destructive colour and
        // the plate, at the period the register declares.
        let frame = match alarm.glow_ms {
            Some(ms) => effects::pulse(p.destructive, p.card, self.elapsed_ms, ms, self.motion),
            None => p.destructive,
        };
        // The headline takes the destructive colour itself, not the ink
        // that goes ON it: the plate here is the card, the way
        // `.kp-alarm__title` sits on the notice's own ground.
        let ink = mix(p.card, p.destructive, t);
        let block = Block::new()
            .borders(Borders::ALL)
            .border_set(th.a.border)
            .border_style(Style::new().fg(th.depth.resolve(Role::Line, mix(p.card, frame, t))))
            .style(Style::new().bg(th.c.card));
        let inner = block.inner(area);
        block.render(area, buf);
        let head = Style::new()
            .fg(th.depth.resolve(Role::Ink, ink))
            .add_modifier(th.a.title_modifier);
        Paragraph::new(vec![
            Line::from(Span::styled(self.headline(), head)),
            Line::from(Span::styled(
                self.body.to_string(),
                Style::new().fg(th
                    .depth
                    .resolve(Role::Ink, mix(p.card, p.card_foreground, t))),
            )),
        ])
        .render(inner, buf);
    }
}

/// The bottom line of telemetry, sliding. Homelab's `ticker_text` joins
/// its segments with a hard-coded `  ::  `; this one uses the divider the
/// theme already declares between its tabs, so the same line reads as
/// `│` in formal and as `▐` in the registers that plate their tabs.
pub struct Ticker<'a> {
    theme: &'a Theme,
    segments: &'a [String],
    elapsed_ms: u32,
    motion: Motion,
    cps: f32,
}

impl<'a> Ticker<'a> {
    pub fn new(theme: &'a Theme, segments: &'a [String]) -> Self {
        Ticker {
            theme,
            segments,
            elapsed_ms: 0,
            motion: Motion::Full,
            // Ten characters a second: homelab's ticker moves one character
            // every three ticks at 30 fps, which is the same speed.
            cps: 10.0,
        }
    }

    pub fn at(mut self, elapsed_ms: u32, motion: Motion) -> Self {
        (self.elapsed_ms, self.motion) = (elapsed_ms, motion);
        self
    }

    pub fn cps(mut self, cps: f32) -> Self {
        self.cps = cps;
        self
    }

    pub fn text(&self, width: u16) -> String {
        effects::marquee(
            self.segments,
            width,
            self.elapsed_ms,
            self.cps,
            self.theme.a.tab_divider,
            self.motion,
        )
    }
}

impl Widget for Ticker<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.text(area.width);
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::new()
                .bg(self.theme.c.muted)
                .fg(self.theme.c.muted_foreground),
        )))
        .render(area, buf);
    }
}

/// The frame of the theme's spinner for this moment. `--kp-spinner-duration`
/// is 900 ms in `css/components.css` and no register overrides it.
pub fn spinner(theme: &Theme, elapsed_ms: u32, motion: Motion) -> &'static str {
    theme.a.fx.spinner.frame(elapsed_ms, 900, motion)
}

// ── The wizard's breadcrumb ─────────────────────────────────────────────

/// Where a multi-step flow stands: the steps behind, the step in hand, the
/// steps ahead.
///
/// homelab draws this once, in the create-container wizard
/// (`client/src/tui/view/mod.rs:242`): five fixed crumbs, cyan plate on the
/// active one, green on what is done, muted on what is not, with `▶`
/// between. Here the plate, the ink and the divider are the theme's, and
/// the steps come from the caller.
pub struct Stepper<'a> {
    theme: &'a Theme,
    steps: &'a [&'a str],
    current: usize,
}

impl<'a> Stepper<'a> {
    pub fn new(theme: &'a Theme, steps: &'a [&'a str], current: usize) -> Self {
        Stepper {
            theme,
            steps,
            current,
        }
    }

    /// The line, as spans, so a caller can put it in a block of its own.
    /// GUESS: the divider is the theme's `tab_divider` — the mark it
    /// already declares between items standing side by side in one row.
    /// Nothing in a register speaks about a wizard's crumbs. The five
    /// registers whose tabs are plates rather than words divide them with
    /// space alone, and a row of steps with nothing between them does not
    /// read as a sequence, so those get an arrow instead.
    pub fn line(&self) -> Line<'static> {
        let (t, a) = (&self.theme.c, self.theme.a);
        let mut spans = Vec::new();
        for (i, step) in self.steps.iter().enumerate() {
            if i > 0 {
                let divider = if a.tab_divider.trim().is_empty() {
                    " ▸ "
                } else {
                    a.tab_divider
                };
                spans.push(Span::styled(
                    divider.to_string(),
                    Style::new().fg(t.muted_foreground),
                ));
            }
            let label = if a.uppercase_labels {
                step.to_uppercase()
            } else {
                (*step).to_string()
            };
            let style = match i.cmp(&self.current) {
                std::cmp::Ordering::Less => Style::new().fg(self
                    .theme
                    .ink(Tone::Success, self.theme.id.palette().background)),
                std::cmp::Ordering::Equal => Style::new()
                    .bg(t.primary)
                    .fg(t.primary_foreground)
                    .add_modifier(a.title_modifier),
                std::cmp::Ordering::Greater => Style::new().fg(t.muted_foreground),
            };
            // The step in hand keeps a cell of its plate on either side, the
            // way every button in this crate is sized to its label plus air.
            let text = if i == self.current {
                format!(" {label} ")
            } else {
                label
            };
            spans.push(Span::styled(text, style));
        }
        Line::from(spans)
    }
}

impl Widget for Stepper<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.line();
        Paragraph::new(line)
            .style(Style::new().bg(self.theme.c.background))
            .render(area, buf);
    }
}

// ── Fuzzy matching ──────────────────────────────────────────────────────

/// Which characters of `item` the query hit, if it hits at all.
///
/// A subsequence match, not a substring one: `dpl` finds "Deploy stack"
/// where homelab's palette (`client/src/tui/model.rs:1135`,
/// `label.to_lowercase().contains(&q)`) would find nothing. Case is
/// ignored, and an empty query matches everything with no marks.
pub fn fuzzy(query: &str, item: &str) -> Option<Vec<usize>> {
    let hay: Vec<char> = item.chars().collect();
    let mut hits = Vec::new();
    let mut at = 0usize;
    for q in query.chars().flat_map(|c| c.to_lowercase()) {
        let found = hay[at..]
            .iter()
            .position(|c| c.to_lowercase().next() == Some(q))?;
        hits.push(at + found);
        at += found + 1;
    }
    Some(hits)
}

/// How good a hit is: earlier and more contiguous wins. Lower is better,
/// so a caller sorts ascending and keeps the original order on a tie.
pub fn fuzzy_score(hits: &[usize]) -> usize {
    if hits.is_empty() {
        return usize::MAX;
    }
    let gaps: usize = hits.windows(2).map(|w| w[1] - w[0] - 1).sum();
    // The first hit's column counts once; every gap counts double, because
    // a run of letters reads as the word and a scatter does not.
    hits[0] + gaps * 2
}

/// One item's label with the characters the query hit lifted out of it.
///
/// The hits take the theme's primary ink, its title modifier and an
/// underline. The underline is not decoration: on the row in hand the
/// plate is often the primary colour itself, so the ink mark disappears
/// there and the underline is what is left to read.
pub fn fuzzy_spans(theme: &Theme, item: &str, hits: &[usize], base: Style) -> Vec<Span<'static>> {
    let hit = base
        .fg(theme.c.primary)
        .add_modifier(theme.a.title_modifier | Modifier::UNDERLINED);
    let mut spans = Vec::new();
    let mut run = String::new();
    let mut run_hit = false;
    for (i, c) in item.chars().enumerate() {
        let is_hit = hits.contains(&i);
        if is_hit != run_hit && !run.is_empty() {
            spans.push(Span::styled(
                std::mem::take(&mut run),
                if run_hit { hit } else { base },
            ));
        }
        run_hit = is_hit;
        run.push(c);
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, if run_hit { hit } else { base }));
    }
    spans
}

// ── The row a person is standing on ─────────────────────────────────────

impl Theme {
    /// A tone as a plate colour, resolved for this terminal. `Tone::None`
    /// has no colour: a caller leaves that side of the style alone.
    ///
    /// For TEXT in a state colour, reach for `Theme::ink` instead — the
    /// state tokens are plates, and 43 of the 66 state/card pairs in the
    /// set read under 4.5:1 when one is used as an ink.
    pub fn tone(&self, tone: Tone) -> Option<Color> {
        let role = match tone {
            Tone::Ink | Tone::MutedInk | Tone::PrimaryInk | Tone::SecondaryInk => Role::Ink,
            _ => Role::Surface,
        };
        self.tone_rgb(tone).map(|c| self.depth.resolve(role, c))
    }

    /// The style of a selected row in this theme.
    pub fn selected_style(&self) -> Style {
        let s = self.a.selection;
        let mut style = Style::new().add_modifier(s.modifier);
        if let Some(bg) = self.tone(s.plate) {
            style = style.bg(bg);
        }
        if let Some(fg) = self.tone(s.ink) {
            style = style.fg(fg);
        }
        style
    }
}

/// A list where one row is the one in hand.
///
/// homelab writes this four times — the stack list, the wizard's presets,
/// the palette, the fleet table (`client/src/tui/view/stacks.rs:65` is one)
/// — each time as a cyan-on-breathing-plate row with a `▶`. Here the plate,
/// the ink, the marker and the weight are the register's, and the rows that
/// are not selected keep the ground they are drawn on.
pub struct SelectList<'a> {
    theme: &'a Theme,
    items: &'a [Line<'static>],
    selected: usize,
    /// The first row drawn, for a list longer than its box.
    offset: usize,
}

impl<'a> SelectList<'a> {
    pub fn new(theme: &'a Theme, items: &'a [Line<'static>], selected: usize) -> Self {
        SelectList {
            theme,
            items,
            selected,
            offset: 0,
        }
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// The marker column: the register's own glyph, padded to the same
    /// width on every row so the labels line up whether selected or not.
    fn marker(&self, selected: bool) -> Option<Span<'static>> {
        let s = self.theme.a.selection;
        if s.marker.is_empty() {
            return None;
        }
        let width = s.marker.chars().count();
        Some(if selected {
            Span::styled(
                s.marker.to_string(),
                Style::new()
                    .fg(self
                        .theme
                        .tone(s.marker_tone)
                        .unwrap_or(self.theme.c.primary))
                    .bg(self.theme.tone(s.plate).unwrap_or(self.theme.c.card)),
            )
        } else {
            Span::raw(" ".repeat(width))
        })
    }

    /// One row, as it is drawn. Public so a test can read a row without a
    /// buffer, and so a caller can put a list inside something else.
    pub fn row(&self, i: usize) -> Line<'static> {
        let selected = i == self.selected;
        let style = if selected {
            self.theme.selected_style()
        } else {
            Style::new().fg(self.theme.c.card_foreground)
        };
        let mut spans = Vec::new();
        if let Some(m) = self.marker(selected) {
            spans.push(m);
        }
        for span in &self.items[i].spans {
            // The row's own marks win over the item's, except the colour an
            // item set for itself when the row is not the one in hand.
            let mut s = style;
            if let Some(fg) = span.style.fg.filter(|_| !selected) {
                s = s.fg(fg);
            }
            spans.push(Span::styled(
                span.content.to_string(),
                s.add_modifier(span.style.add_modifier),
            ));
        }
        if self.theme.a.selection.spaced && selected {
            // deco spaces the letters of the row it is standing on.
            spans = spans
                .into_iter()
                .map(|s| {
                    let spaced: String =
                        s.content.chars().flat_map(|c| [c, ' ']).collect::<String>();
                    Span::styled(spaced, s.style)
                })
                .collect();
        }
        Line::from(spans)
    }
}

impl Widget for SelectList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Line<'static>> = (self.offset..self.items.len())
            .take(area.height as usize)
            .map(|i| self.row(i))
            .collect();
        let plate = self.theme.c.card;
        for (n, row) in rows.into_iter().enumerate() {
            let y = area.y + n as u16;
            // The selected row's plate runs the width of the list, not the
            // width of its label.
            let bg = if self.offset + n == self.selected {
                self.theme
                    .tone(self.theme.a.selection.plate)
                    .unwrap_or(plate)
            } else {
                plate
            };
            Paragraph::new(row).style(Style::new().bg(bg)).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }
}

// ── The command palette ─────────────────────────────────────────────────

/// Everything a person can do, findable by typing three letters of it.
///
/// homelab has one (`client/src/tui/view/mod.rs:830`) and it is the piece
/// that makes the rest discoverable — nothing has to be known by heart.
/// Two things are different here: the match is a subsequence rather than a
/// substring, and the letters that matched are lifted out in the theme's
/// own primary ink.
pub struct CommandPalette<'a> {
    theme: &'a Theme,
    query: &'a str,
    items: &'a [&'a str],
    selected: usize,
    /// Width and height in cells.
    size: (u16, u16),
    blink: Option<u32>,
}

impl<'a> CommandPalette<'a> {
    pub fn new(theme: &'a Theme, query: &'a str, items: &'a [&'a str], selected: usize) -> Self {
        CommandPalette {
            theme,
            query,
            items,
            selected,
            size: (46, 12),
            blink: None,
        }
    }

    pub fn size(mut self, size: (u16, u16)) -> Self {
        self.size = size;
        self
    }

    pub fn blink(mut self, elapsed_ms: u32) -> Self {
        self.blink = Some(elapsed_ms);
        self
    }

    /// The items that match, best first, each with the columns its letters
    /// hit. An empty query keeps the caller's own order.
    pub fn matches(&self) -> Vec<(usize, Vec<usize>)> {
        let mut found: Vec<(usize, Vec<usize>)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| fuzzy(self.query, item).map(|hits| (i, hits)))
            .collect();
        if !self.query.is_empty() {
            found.sort_by_key(|(i, hits)| (fuzzy_score(hits), *i));
        }
        found
    }

    /// Draws over whatever is on the screen. Returns the rect it took, so a
    /// caller can tell where it landed.
    pub fn render_over(self, screen: Rect, buf: &mut Buffer) -> Rect {
        let th = self.theme;
        let found = self.matches();
        let popup = Popup::new(th, "Commands", self.size);
        let area = popup.area(screen);
        let inner = popup.render_over(screen, buf);
        if inner.height == 0 {
            return area;
        }
        // The query line, with the theme's own caret riding it.
        let field = Rect { height: 1, ..inner };
        let mut f = Field::new(th, "find", self.query).focused(true);
        if let Some(ms) = self.blink {
            f = f.blink(ms / 33);
        }
        f.render(field, buf);

        let list = Rect {
            y: inner.y + 1,
            height: inner.height - 1,
            ..inner
        };
        let base = Style::new().fg(th.c.popover_foreground);
        let rows: Vec<Line<'static>> = found
            .iter()
            .map(|(i, hits)| Line::from(fuzzy_spans(th, self.items[*i], hits, base)))
            .collect();
        if rows.is_empty() {
            Paragraph::new(Line::from(Span::styled(
                "nothing matches",
                Style::new().fg(th.c.muted_foreground),
            )))
            .render(list, buf);
            return area;
        }
        let selected = self.selected.min(rows.len() - 1);
        // Keep the row in hand inside the box.
        let offset = selected.saturating_sub(list.height.saturating_sub(1) as usize);
        SelectList::new(th, &rows, selected)
            .offset(offset)
            .render(list, buf);
        area
    }
}

/// The page behind an overlay, stepped back.
///
/// Every colour on it moves a third of the way to the theme's own
/// background, ink included, so the layer underneath reads as further away
/// rather than as switched off. A sixteen-colour terminal has no third of
/// the way, so it keeps its colours.
pub fn scrim(theme: &Theme, screen: Rect, buf: &mut Buffer) {
    if theme.depth == ColorDepth::Ansi16 {
        return;
    }
    let p = theme.id.palette();
    let step = |c: Option<Color>, role: Role, towards: Rgb| match c {
        Some(Color::Rgb(r, g, b)) => {
            Some(theme.depth.resolve(role, mix(Rgb(r, g, b), towards, 0.35)))
        }
        other => other,
    };
    for y in screen.y..screen.bottom() {
        for x in screen.x..screen.right() {
            let cell = &mut buf[(x, y)];
            let (fg, bg) = (cell.fg, cell.bg);
            if let Some(c) = step(Some(fg), Role::Ink, p.background) {
                cell.set_fg(c);
            }
            if let Some(c) = step(Some(bg), Role::Surface, p.background) {
                cell.set_bg(c);
            }
        }
    }
}

/// The shadow an overlay throws: one row under it and one column beside
/// it, darkened rather than blacked out, and clipped to the screen.
pub fn shadow(theme: &Theme, area: Rect, screen: Rect, buf: &mut Buffer) {
    if theme.depth == ColorDepth::Ansi16 {
        return;
    }
    let p = theme.id.palette();
    // Towards the darker of the two grounds, so the shadow reads on a
    // light theme as well as on a dark one.
    let dark = if theme.id.dark() {
        Rgb(0, 0, 0)
    } else {
        p.foreground
    };
    let dim = |x: u16, y: u16, buf: &mut Buffer| {
        if x < screen.right() && y < screen.bottom() {
            let cell = &mut buf[(x, y)];
            if let Color::Rgb(r, g, b) = cell.bg {
                cell.set_bg(
                    theme
                        .depth
                        .resolve(Role::Surface, mix(Rgb(r, g, b), dark, 0.45)),
                );
            }
            if let Color::Rgb(r, g, b) = cell.fg {
                cell.set_fg(
                    theme
                        .depth
                        .resolve(Role::Ink, mix(Rgb(r, g, b), dark, 0.45)),
                );
            }
        }
    };
    for x in area.x + 1..area.right() + 1 {
        dim(x, area.bottom(), buf);
    }
    for y in area.y + 1..area.bottom() + 1 {
        dim(area.right(), y, buf);
    }
}

// ── Badges: a state, in one word ────────────────────────────────────────

/// A chip carrying a state — `running`, `UPD`, `sealed` — on a plate of
/// its own, ending the way this register ends a plate.
///
/// homelab writes these by hand in three screens (`●`/`○`, `[UPD]`,
/// `[OFF]`, `RUN ✓`, `DOWN ✗`); the web package has `.kp-badge`. This is
/// the first of the three gaps `docs/HOMELAB_PROOF.md` measured.
pub struct Badge<'a> {
    theme: &'a Theme,
    label: &'a str,
    tone: Tone,
    /// A bare badge is ink on the ground; a plated one carries the tone as
    /// its plate, the way a filled button does.
    plated: bool,
}

impl<'a> Badge<'a> {
    pub fn new(theme: &'a Theme, label: &'a str, tone: Tone) -> Self {
        Badge {
            theme,
            label,
            tone,
            plated: false,
        }
    }

    pub fn plated(mut self, plated: bool) -> Self {
        self.plated = plated;
        self
    }

    /// The state dot every status list needs: filled when it is up, hollow
    /// when it is not. A glyph rather than a plate, because a dot beside a
    /// name is read as part of the name.
    pub fn dot(theme: &Theme, up: bool) -> Span<'static> {
        if up {
            Span::styled(
                "●",
                Style::new().fg(theme.ink(Tone::Success, theme.id.palette().card)),
            )
        } else {
            Span::styled("○", Style::new().fg(theme.c.muted_foreground))
        }
    }

    /// The chip as spans, so it sits inside a line beside other text.
    pub fn spans(&self) -> Vec<Span<'static>> {
        let th = self.theme;
        let ground = th.id.palette().card;
        // A bare chip is text on the card, so it takes an ink that can be
        // read there; a plated one keeps the token as its plate.
        let colour = if self.plated {
            th.tone(self.tone).unwrap_or(th.c.muted)
        } else {
            th.ink(self.tone, ground)
        };
        let label = if th.a.uppercase_labels {
            self.label.to_uppercase()
        } else {
            self.label.to_string()
        };
        if !self.plated {
            let (open, close) = th.a.button_brackets;
            return vec![Span::styled(
                format!("{open}{label}{close}"),
                Style::new().fg(colour),
            )];
        }
        // A plated chip ends the way this register ends a button: half
        // blocks for a rounded theme, square cells for a sharp one, the
        // register's own brackets where it has them.
        let plate_rgb = th.tone_rgb(self.tone).unwrap_or(th.id.palette().muted);
        let plate = Style::new().bg(colour).fg(th.on_plate(plate_rgb));
        let ground = Style::new().fg(colour).bg(th.c.card);
        match th.a.button_face {
            ButtonFace::Soft => vec![
                Span::styled("▐", ground),
                Span::styled(label, plate),
                Span::styled("▌", ground),
            ],
            ButtonFace::Bracket => {
                let (open, close) = th.a.button_brackets;
                vec![Span::styled(format!("{open}{label}{close}"), plate)]
            }
            ButtonFace::Square => vec![Span::styled(format!(" {label} "), plate)],
        }
    }
}

// ── Facts: a label, a value, a state ────────────────────────────────────

/// Rows of `label  value`, in columns, with each value free to carry its
/// own state colour.
///
/// The second gap: homelab aligns these by hand on five screens
/// (`vmid 112   env ● sealed`), and the web package has `.kp-definition`.
pub struct Facts<'a> {
    theme: &'a Theme,
    /// Label, value, and the tone the value reads in.
    rows: &'a [(&'a str, String, Tone)],
    /// How many label/value pairs stand side by side on one line.
    columns: usize,
}

impl<'a> Facts<'a> {
    pub fn new(theme: &'a Theme, rows: &'a [(&'a str, String, Tone)]) -> Self {
        Facts {
            theme,
            rows,
            columns: 2,
        }
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// The lines, with the labels of a column padded to one width so the
    /// values stand in a column of their own.
    pub fn lines(&self) -> Vec<Line<'static>> {
        let th = self.theme;
        let key = Style::new().fg(th.c.muted_foreground);
        let widest: Vec<usize> = (0..self.columns)
            .map(|col| {
                self.rows
                    .iter()
                    .skip(col)
                    .step_by(self.columns)
                    .map(|(label, _, _)| label.chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        self.rows
            .chunks(self.columns)
            .map(|chunk| {
                let mut spans = Vec::new();
                for (i, (label, value, tone)) in chunk.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled("   ", key));
                    }
                    let pad = widest[i].saturating_sub(label.chars().count());
                    spans.push(Span::styled(format!("{}{} ", label, " ".repeat(pad)), key));
                    spans.push(Span::styled(
                        value.clone(),
                        Style::new().fg(th.ink(*tone, th.id.palette().card)),
                    ));
                }
                Line::from(spans)
            })
            .collect()
    }
}

impl Widget for Facts<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = self.theme.c.card;
        Paragraph::new(self.lines())
            .style(Style::new().bg(bg))
            .render(area, buf);
    }
}

// ── Braille: four times the resolution a block has ──────────────────────

/// A chart drawn in braille dots: two samples per column, four levels per
/// row, so a strip three cells high carries twelve levels where a block
/// sparkline carries eight in one.
///
/// homelab has the idea and never used it (`braille_spark` in
/// `client/src/tui/fx.rs` is dead code, and its dashboard draws block
/// bars). This is the same idea finished: an area under the line, a value
/// that crosses a threshold painted in the theme's warning colour, and no
/// colour of its own.
pub struct Spark<'a> {
    theme: &'a Theme,
    data: &'a [f64],
    max: f64,
    tone: Tone,
    /// Above this share of `max`, a column takes the warning tone.
    warn_above: Option<f64>,
    /// Fill under the line, or draw the line alone.
    fill: bool,
}

/// The dot bits of a braille cell: two columns of four.
/// `DOTS[col][row]`, row 0 at the top.
const DOTS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

impl<'a> Spark<'a> {
    pub fn new(theme: &'a Theme, data: &'a [f64]) -> Self {
        let max = data
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            .max(f64::EPSILON);
        Spark {
            theme,
            data,
            max,
            tone: Tone::Primary,
            warn_above: None,
            fill: true,
        }
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max.max(f64::EPSILON);
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn warn_above(mut self, share: f64) -> Self {
        self.warn_above = Some(share);
        self
    }

    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// The glyphs, row by row, and the share the tallest sample in each
    /// cell column reached — a caller can colour by it, and a test can
    /// read the shape without a buffer.
    pub fn glyphs(&self, width: u16, height: u16) -> (Vec<String>, Vec<f64>) {
        let (w, h) = (width as usize, height as usize);
        if w == 0 || h == 0 {
            return (Vec::new(), Vec::new());
        }
        let levels = h * 4;
        // Two samples to a column, the last `2 * w` of them, right-aligned
        // so the newest reading sits at the right edge.
        let take = (w * 2).min(self.data.len());
        let tail = &self.data[self.data.len() - take..];
        let mut cells = vec![vec![0u8; w]; h];
        let mut peak = vec![0.0_f64; w];
        for (i, v) in tail.iter().enumerate() {
            let col = w - take.div_ceil(2) + i / 2;
            let half = i % 2;
            let share = (v / self.max).clamp(0.0, 1.0);
            peak[col] = peak[col].max(share);
            let lit = ((share * levels as f64).round() as usize).clamp(1, levels);
            // Row 0 is the top cell, so a value of `lit` levels fills the
            // bottom `lit` of `levels`.
            for level in 0..levels {
                let from_bottom = levels - 1 - level;
                let on = if self.fill {
                    from_bottom < lit
                } else {
                    from_bottom + 1 == lit
                };
                if on {
                    cells[level / 4][col] |= DOTS[half][level % 4];
                }
            }
        }
        let rows = cells
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|bits| char::from_u32(0x2800 + bits as u32).unwrap_or(' '))
                    .collect::<String>()
            })
            .collect();
        (rows, peak)
    }
}

impl Widget for Spark<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.data.is_empty() {
            return;
        }
        let th = self.theme;
        let (rows, peak) = self.glyphs(area.width, area.height);
        let base = th.tone(self.tone).unwrap_or(th.c.primary);
        for (y, row) in rows.iter().enumerate() {
            for (x, glyph) in row.chars().enumerate() {
                let colour = match self.warn_above {
                    Some(share) if peak[x] > share => th.c.warning,
                    _ => base,
                };
                let cell = &mut buf[(area.x + x as u16, area.y + y as u16)];
                cell.set_symbol(&glyph.to_string());
                cell.set_style(Style::new().fg(colour).bg(th.c.card));
            }
        }
    }
}

// ── The table ───────────────────────────────────────────────────────────

/// A column: its heading, its width, and which way its cells sit.
#[derive(Clone, Copy, Debug)]
pub struct Column<'a> {
    pub head: &'a str,
    pub width: u16,
    /// Numbers read right-aligned; the package sets `tabular-nums` on
    /// every table cell for the same reason.
    pub right: bool,
}

impl<'a> Column<'a> {
    pub fn new(head: &'a str, width: u16) -> Self {
        Column {
            head,
            width,
            right: false,
        }
    }

    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

/// A table in the register's own dress: the header plate, its case and
/// tracking, the rule under it — including the one register that draws
/// that rule as a gradient — and the row in hand on the theme's own
/// selection plate.
///
/// The third gap `docs/HOMELAB_PROOF.md` measured, and the largest: the app
/// grid cost 45 lines of hand-chosen style without it.
pub struct DataTable<'a> {
    theme: &'a Theme,
    columns: &'a [Column<'a>],
    rows: &'a [Vec<Line<'static>>],
    selected: Option<usize>,
    offset: usize,
}

impl<'a> DataTable<'a> {
    pub fn new(
        theme: &'a Theme,
        columns: &'a [Column<'a>],
        rows: &'a [Vec<Line<'static>>],
    ) -> Self {
        DataTable {
            theme,
            columns,
            rows,
            selected: None,
            offset: 0,
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// The header line, in the register's case and tracking.
    pub fn header(&self) -> Line<'static> {
        let th = self.theme;
        let h = th.a.table;
        let mut style = Style::new().add_modifier(h.modifier);
        if let Some(bg) = th.tone(h.plate) {
            style = style.bg(bg);
        }
        style = style.fg(th.tone(h.ink).unwrap_or(th.c.muted_foreground));
        let spans = self
            .columns
            .iter()
            .map(|c| {
                let head = if h.uppercase {
                    c.head.to_uppercase()
                } else {
                    c.head.to_string()
                };
                // The tracking goes on only where the column has room for
                // it — the same rule a button's label follows: a heading
                // that does not fit loses its spacing before its tail.
                let spaced: String = head.chars().flat_map(|ch| [ch, ' ']).collect();
                let head = if h.spaced && spaced.trim_end().chars().count() <= c.width as usize {
                    spaced.trim_end().to_string()
                } else {
                    head
                };
                Span::styled(pad(&head, c.width, c.right), style)
            })
            .collect::<Vec<_>>();
        Line::from(spans)
    }

    /// The rule under the header, cell by cell. A gradient register gets a
    /// ramp from `--primary` to `--accent` across the whole width, which is
    /// what `--kp-stripe` paints on the page.
    fn rule_cells(&self, width: u16) -> Vec<(String, Color)> {
        let th = self.theme;
        let h = th.a.table;
        let p = th.id.palette();
        let glyph = match h.rule {
            Rule::Thin => "─",
            Rule::Heavy => "━",
            Rule::Double => "═",
            Rule::Gradient => "━",
        };
        (0..width)
            .map(|x| {
                let colour = match h.rule {
                    Rule::Gradient => {
                        let t = x as f32 / (width.max(2) - 1) as f32;
                        th.depth.resolve(Role::Line, mix(p.primary, p.accent, t))
                    }
                    _ => th.tone(h.rule_tone).unwrap_or(th.c.border_strong),
                };
                (glyph.to_string(), colour)
            })
            .collect()
    }
}

/// Pad a cell to its column, on the side its alignment asks for.
/// A cell padded to its column with every one of its spans intact. A cell
/// is built out of parts that each carry a colour — the hue bar before a
/// node's name, the dot before its status, two badges side by side — and
/// flattening it to the first span's style paints the whole cell in one of
/// them; the second proof found the fleet table doing exactly that
/// [docs/HOMELAB_PROOF.md]. A cell wider than its column is cut at the
/// column's edge, never wrapped, because a row is one line.
fn pad_spans(cell: &Line<'static>, width: u16, right: bool) -> Vec<Span<'static>> {
    let width = width as usize;
    let mut out: Vec<Span<'static>> = Vec::with_capacity(cell.spans.len() + 1);
    let mut used = 0usize;
    for span in &cell.spans {
        if used >= width {
            break;
        }
        let n = span.content.chars().count();
        if used + n <= width {
            out.push(span.clone());
            used += n;
        } else {
            let cut: String = span.content.chars().take(width - used).collect();
            out.push(Span::styled(cut, span.style));
            used = width;
        }
    }
    if used < width {
        let gap = Span::raw(" ".repeat(width - used));
        if right {
            out.insert(0, gap);
        } else {
            out.push(gap);
        }
    }
    out
}

fn pad(text: &str, width: u16, right: bool) -> String {
    let width = width as usize;
    let n = text.chars().count();
    if n >= width {
        return text.chars().take(width).collect();
    }
    let gap = " ".repeat(width - n);
    if right {
        format!("{gap}{text}")
    } else {
        format!("{text}{gap}")
    }
}

impl Widget for DataTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let th = self.theme;
        let plate = th.c.card;
        let head_row = Rect { height: 1, ..area };
        Paragraph::new(self.header())
            .style(Style::new().bg(plate))
            .render(head_row, buf);

        if area.height >= 2 {
            for (x, (glyph, colour)) in self.rule_cells(area.width).into_iter().enumerate() {
                let cell = &mut buf[(area.x + x as u16, area.y + 1)];
                cell.set_symbol(&glyph);
                cell.set_style(Style::new().fg(colour).bg(plate));
            }
        }

        let body = Rect {
            y: area.y + 2,
            height: area.height.saturating_sub(2),
            ..area
        };
        let lines: Vec<Line<'static>> = self
            .rows
            .iter()
            .map(|cells| {
                let spans: Vec<Span<'static>> = cells
                    .iter()
                    .zip(self.columns)
                    .flat_map(|(cell, col)| pad_spans(cell, col.width, col.right))
                    .collect();
                Line::from(spans)
            })
            .collect();
        match self.selected {
            Some(sel) => SelectList::new(th, &lines, sel)
                .offset(self.offset)
                .render(body, buf),
            None => {
                for (n, line) in lines.iter().skip(self.offset).enumerate() {
                    if n as u16 >= body.height {
                        break;
                    }
                    Paragraph::new(line.clone())
                        .style(Style::new().bg(plate))
                        .render(
                            Rect {
                                y: body.y + n as u16,
                                height: 1,
                                ..body
                            },
                            buf,
                        );
                }
            }
        }
    }
}
