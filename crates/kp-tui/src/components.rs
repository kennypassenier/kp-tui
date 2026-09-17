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
    widgets::{Clear, Paragraph, Widget},
};

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
