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

use crate::anatomy::Texture;
use crate::color::{ColorDepth, Rgb, Role};
use crate::effects::{self, mix};
use crate::fx::Motion;
use crate::logs::{LogBuffer, Severity};
use crate::theme::Theme;
use crate::widgets::Panel;

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
        Clear.render(area, buf);
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
            let danger = Style::new().fg(t.destructive).bg(t.popover);
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
        let filled = ((bar_w as f32) * self.value).round() as u16;
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
        spans.push(Span::styled(
            " ".repeat(bar_w.saturating_sub(filled) as usize),
            Style::new().bg(t.muted),
        ));
        spans.push(Span::styled(
            format!(" {reading}"),
            Style::new().fg(if self.value >= self.danger_at {
                t.destructive
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
                        .fg(c.warning_foreground)
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
