//! Demo state and the draw functions of both screens. The terminal loop
//! lives in `main.rs`; tests drive this with `TestBackend`.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::config::Config;
use kp_tui::KP_THEMES_VERSION as PACKAGE_VERSION;
use kp_tui::color::ColorDepth;
use kp_tui::dashboard::{self, Dashboard};
use kp_tui::fx::{self, Motion};
use kp_tui::widgets::{Button, ButtonKind, ButtonState, Panel, RevealText, ThemedTabs};
use kp_tui::{
    AlarmPanel, CommandPalette, Field, KeyHints, Meter, Popup, PopupKind, Rail, SelectList, Stage,
    Stepper, Surface, Texture, Theme, Ticker, roll, source_colour, spinner,
};

pub const TABS: [&str; 3] = ["Overview", "Deployments", "Settings"];

const CONTENT: [(&str, &str, &str); 3] = [
    (
        "Status",
        "All services are running",
        "Twelve containers on two hosts. The last backup finished at 03:10 and was verified at 03:24. \
         No certificate expires in the next thirty days.",
    ),
    (
        "Release",
        "Version 2.4 is ready to deploy",
        "The build passed on both hosts. Deploying restarts the web and worker containers one at a time; \
         the database is not touched. Rolling back restores 2.3 from the image cache.",
    ),
    (
        "Preferences",
        "Theme and motion are saved",
        "Press t to change the theme and m to turn motion off. Both choices are written to the \
         config file and read on the next start.",
    ),
];

/// cyberpunk-register.css `animation: kp-charge 520ms`, the sweep across a
/// button's face.
pub const CHARGE_MS: u32 = 520;

/// A button takes the width it asks for, at the start of the box it was
/// given, and never more than the box holds.
fn fit(slot: Rect, want: u16) -> Rect {
    Rect {
        width: want.min(slot.width),
        ..slot
    }
}

pub const BUTTONS: [(&str, ButtonKind); 2] = [
    ("Deploy", ButtonKind::Primary),
    ("Roll back", ButtonKind::Destructive),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Live machine stats, charts and the journal. The binary opens here.
    Dashboard,
    /// The first round's panels, tabs and buttons. `App::new` starts here,
    /// so the first round's tests read the same frame they always did.
    Components,
    /// What homelab hand-rolls today: a popup over a screen, fields with
    /// the theme's caret, meters with their thresholds, and one keymap
    /// drawn as a footer and as an overlay [docs/HOMELAB_INVENTORY.md].
    Console,
    /// The theme's own motion: the texture on the ground, the sweep where
    /// a register declares one, the alarm, the spinner and the ticker —
    /// homelab's six hand-rolled effects, answered by the theme.
    Effects,
    /// homelab's own stacks screen, rebuilt on this crate [docs/HOMELAB_PROOF.md].
    Fleet,
    /// homelab's own dashboard, rebuilt on this crate — the second proof
    /// [docs/HOMELAB_PROOF.md].
    Ops,
    /// homelab's own settings tab, rebuilt on this crate — the third proof
    /// [docs/HOMELAB_PROOF.md].
    Settings,
}

pub struct App {
    pub screen: Screen,
    pub dash: Dashboard,
    pub config: Config,
    pub depth: ColorDepth,
    pub theme: Theme,
    pub tab: usize,
    pub focus: usize,
    /// Which button is pressed, and for how many more milliseconds. A
    /// terminal reports key presses, not releases (unless the kitty keyboard
    /// protocol is on), so "pressed" is shown for a fixed time.
    pub pressed: Option<(usize, u32)>,
    pub reveal_ms: u32,
    /// Milliseconds since the focused button took focus, while its charge
    /// sweep runs. cyberpunk sweeps on hover; a keyboard TUI has no
    /// pointer, so focus is where it lands (GUESS, as with the focus
    /// modifier).
    pub charge_ms: Option<u32>,
    /// Milliseconds since the alarm was raised: the strike and the glitch
    /// read it, and `a` sets it back to zero.
    pub alarm_ms: u32,
    /// The ticker's segments, rebuilt from the live sample each tick, so
    /// the line that slides past is this machine and not a fixture.
    pub ticker: Vec<String>,
    /// The command palette: open, what is typed in it, and which row is in
    /// hand. `p` opens it, Esc closes it.
    pub palette_open: bool,
    pub palette_query: String,
    pub palette_sel: usize,
    /// Which stack is in hand in the console's list.
    pub stack_sel: usize,
    pub field_sel: usize,
    /// Which step of the wizard the breadcrumb shows.
    pub step: usize,
    pub config_path: Option<PathBuf>,
    pub message: String,
    pub quit: bool,
}

pub struct Areas {
    pub header: Rect,
    pub tabs: Rect,
    pub story: Rect,
    pub actions: Rect,
    pub states: Rect,
    pub footer: Rect,
}

pub fn areas(area: Rect) -> Areas {
    let [header, tabs, body, states, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(9),
        Constraint::Length(5),
        Constraint::Length(1),
    ])
    .areas(area);
    let [story, actions] =
        Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
    Areas {
        header,
        tabs,
        story,
        actions,
        states,
        footer,
    }
}

impl App {
    pub fn new(config: Config, depth: ColorDepth, config_path: Option<PathBuf>) -> Self {
        App {
            screen: Screen::Components,
            dash: Dashboard::default(),
            config,
            depth,
            theme: Theme::new(config.theme, depth),
            tab: 1,
            focus: 0,
            pressed: None,
            reveal_ms: 0,
            charge_ms: None,
            alarm_ms: 0,
            ticker: Vec::new(),
            palette_open: false,
            palette_query: String::from("st"),
            palette_sel: 0,
            stack_sel: 1,
            field_sel: 0,
            step: 2,
            config_path,
            message: String::new(),
            quit: false,
        }
    }

    fn persist(&mut self) {
        if let Some(path) = &self.config_path {
            self.message = match self.config.save(path) {
                Ok(()) => format!("saved to {}", path.display()),
                Err(e) => format!("not saved: {e}"),
            };
        }
    }

    /// The next frame renders in the next theme; nothing else is rebuilt.
    pub fn cycle_theme(&mut self) {
        self.config.theme = self.config.theme.next();
        self.theme = Theme::new(self.config.theme, self.depth);
        self.reveal_ms = 0;
        self.persist();
    }

    pub fn toggle_motion(&mut self) {
        self.config.motion = match self.config.motion {
            Motion::Full => Motion::Reduced,
            Motion::Reduced => Motion::Full,
        };
        self.persist();
    }

    pub fn tick(&mut self, ms: u32) {
        self.reveal_ms = self.reveal_ms.saturating_add(ms);
        self.dash.tick(ms);
        self.charge_ms = self.charge_ms.map(|c| c + ms).filter(|c| *c < CHARGE_MS);
        self.alarm_ms = self.alarm_ms.saturating_add(ms);
        self.ticker = match self.dash.history.back() {
            Some((_, s)) => vec![
                format!("cpu {:.0}%", s.cpu_total),
                format!("mem {:.0}%", s.mem_used_pct),
                format!("load {:.2}", s.load[0]),
                format!("net {:.0} kB/s in", s.rx_bps / 1000.0),
                format!("{} log lines", self.dash.logs.len()),
                format!("draw {:.2} ms", self.dash.draw_ms),
            ],
            None => vec![
                "no sample yet".into(),
                format!("theme {}", self.theme.id.name()),
                format!("{} log lines", self.dash.logs.len()),
            ],
        };
        if let Some((i, left)) = self.pressed {
            self.pressed = left.checked_sub(ms).filter(|l| *l > 0).map(|l| (i, l));
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.palette_open {
            match key.code {
                KeyCode::Esc => self.palette_open = false,
                KeyCode::Char(c) => {
                    self.palette_query.push(c);
                    self.palette_sel = 0;
                }
                KeyCode::Backspace => {
                    self.palette_query.pop();
                    self.palette_sel = 0;
                }
                KeyCode::Down => self.palette_sel += 1,
                KeyCode::Up => self.palette_sel = self.palette_sel.saturating_sub(1),
                KeyCode::Enter => self.palette_open = false,
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Char('t') => self.cycle_theme(),
            KeyCode::Char('m') => self.toggle_motion(),
            KeyCode::Char('r') => self.reveal_ms = 0,
            // The alarm strikes once, so it needs a key to strike again.
            KeyCode::Char('a') => self.alarm_ms = 0,
            KeyCode::Char('p') if self.screen == Screen::Console => {
                self.palette_open = true;
                self.palette_sel = 0;
            }
            KeyCode::Down | KeyCode::Char('j') if self.screen == Screen::Settings => {
                self.field_sel = (self.field_sel + 1) % crate::settings::ROWS;
            }
            KeyCode::Up | KeyCode::Char('k') if self.screen == Screen::Settings => {
                self.field_sel =
                    (self.field_sel + crate::settings::ROWS - 1) % crate::settings::ROWS;
            }
            KeyCode::Down | KeyCode::Char('j')
                if matches!(self.screen, Screen::Fleet | Screen::Ops) =>
            {
                self.stack_sel = (self.stack_sel + 1) % crate::fleet::FLEET.len();
            }
            KeyCode::Up | KeyCode::Char('k')
                if matches!(self.screen, Screen::Fleet | Screen::Ops) =>
            {
                self.stack_sel =
                    (self.stack_sel + crate::fleet::FLEET.len() - 1) % crate::fleet::FLEET.len();
            }
            KeyCode::Char('n') if self.screen == Screen::Console => {
                self.step = (self.step + 1) % WIZARD.len();
            }
            KeyCode::Char('s') => {
                self.screen = match self.screen {
                    Screen::Dashboard => Screen::Components,
                    Screen::Components => Screen::Console,
                    Screen::Console => Screen::Effects,
                    Screen::Effects => Screen::Fleet,
                    Screen::Fleet => Screen::Ops,
                    Screen::Ops => Screen::Settings,
                    Screen::Settings => Screen::Dashboard,
                };
                self.reveal_ms = 0;
            }
            _ if self.screen == Screen::Dashboard => self.dashboard_key(key.code),
            KeyCode::Left => {
                self.tab = (self.tab + TABS.len() - 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Right => {
                self.tab = (self.tab + 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = (self.focus + 1) % BUTTONS.len();
                self.charge_ms = Some(0);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.pressed = Some((self.focus, 160));
                self.message = format!("{} pressed", BUTTONS[self.focus].0);
            }
            _ => {}
        }
    }

    fn dashboard_key(&mut self, code: KeyCode) {
        let page = 10;
        let logs = &mut self.dash.logs;
        match code {
            KeyCode::Char('p') => logs.toggle_pause(),
            KeyCode::Char('f') => logs.cycle_filter(),
            KeyCode::Up | KeyCode::Char('k') => logs.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => logs.scroll_down(1),
            KeyCode::PageUp => logs.scroll_up(page),
            KeyCode::PageDown => logs.scroll_down(page),
            KeyCode::Home => logs.scroll_up(usize::MAX),
            KeyCode::End => logs.follow(),
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        if self.screen == Screen::Dashboard {
            let header = format!(
                "{PACKAGE_VERSION} · theme {} · colours {} · motion {} · {} fps",
                self.theme.id.name(),
                self.depth.label(),
                if self.config.motion == Motion::Reduced {
                    "reduced"
                } else {
                    "full"
                },
                self.dash.fps
            );
            let view = dashboard::View {
                theme: &self.theme,
                motion: self.config.motion,
                reveal_ms: self.reveal_ms,
                header,
            };
            dashboard::draw(frame, &self.dash, &view);
            return;
        }
        if self.screen == Screen::Console {
            self.draw_console(frame);
            return;
        }
        if self.screen == Screen::Effects {
            self.draw_effects(frame);
            return;
        }
        if self.screen == Screen::Fleet {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::fleet::draw(
                frame,
                &self.theme,
                self.stack_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Ops {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::ops::draw(
                frame,
                &self.theme,
                self.stack_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Settings {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::settings::draw(
                frame,
                &self.theme,
                self.field_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        let th = &self.theme;
        let buf_area = frame.area();
        frame.render_widget(Block::new().style(th.base()), buf_area);
        let a = areas(buf_area);

        let header = Line::from(vec![
            Span::styled(
                th.label("kp-themes"),
                Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  {PACKAGE_VERSION} · theme {} · colours {} · motion {}",
                    th.id.name(),
                    self.depth.label(),
                    if self.config.motion == Motion::Reduced {
                        "reduced"
                    } else {
                        "full"
                    }
                ),
                Style::new().fg(th.c.muted_foreground),
            ),
        ]);
        frame.render_widget(Paragraph::new(header), a.header);
        frame.render_widget(ThemedTabs::new(th, &TABS, self.tab), a.tabs);

        // Story panel: the reveal, the body, and a filter field for the cursor.
        let (title, headline, body) = CONTENT[self.tab];
        let panel = Panel::new(th, title).focused(false);
        let inner = panel.block().inner(a.story);
        frame.render_widget(panel, a.story);
        let [head, _, text, field] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(2),
            Constraint::Length(1),
        ])
        .areas(inner.inner(ratatui::layout::Margin::new(1, 0)));
        // With `--features tachyonfx` the headline is drawn whole here and
        // main.rs runs the tachyonfx effect over `App::headline`.
        let motion = self.config.motion;
        frame.render_widget(RevealText::new(th, headline, self.reveal_ms, motion), head);
        frame.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: true })
                .style(Style::new().fg(th.c.card_foreground)),
            text,
        );
        let prompt = th.label("filter");
        let value = "web";
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{prompt}: "),
                    Style::new().fg(th.c.muted_foreground),
                ),
                Span::styled(value, Style::new().fg(th.c.foreground)),
            ])),
            field,
        );
        let col = field.x + (prompt.chars().count() + 2 + value.len()) as u16;
        frame.set_cursor_position((col.min(field.right().saturating_sub(1)), field.y));

        // Actions panel: focus follows Tab, Enter presses.
        let panel = Panel::new(th, "Actions").focused(true);
        let inner = panel.block().inner(a.actions);
        frame.render_widget(panel, a.actions);
        let rows = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas::<4>(inner.inner(ratatui::layout::Margin::new(2, 1)));
        for (i, (label, kind)) in BUTTONS.iter().enumerate() {
            let state = match self.pressed {
                Some((p, _)) if p == i => ButtonState::Pressed,
                _ if self.focus == i => ButtonState::Focus,
                _ => ButtonState::Rest,
            };
            let charge = match (self.focus == i, self.config.motion) {
                (true, Motion::Full) => self.charge_ms.map(|c| c as f32 / CHARGE_MS as f32),
                _ => None,
            };
            let button = Button::new(th, label)
                .kind(*kind)
                .state(state)
                .charge(charge);
            let slot = fit(rows[i * 2], button.width());
            frame.render_widget(button, slot);
        }

        // Every button state at once, so no key has to be pressed to judge them.
        let panel = Panel::new(th, "Button states");
        let inner = panel.block().inner(a.states);
        frame.render_widget(panel, a.states);
        let cells = Layout::horizontal([Constraint::Ratio(1, 4); 4])
            .spacing(2)
            .areas::<4>(inner.inner(ratatui::layout::Margin::new(1, 0)));
        for (cell, (label, state)) in cells.iter().zip([
            ("Rest", ButtonState::Rest),
            ("Focus", ButtonState::Focus),
            ("Pressed", ButtonState::Pressed),
            ("Disabled", ButtonState::Disabled),
        ]) {
            let button = Button::new(th, label).state(state);
            let slot = fit(*cell, button.width());
            frame.render_widget(button, slot);
        }

        let keys = format!(
            " s dashboard · t theme · ←/→ tab · Tab focus · Enter press · r replay · m motion · q quit   {}",
            self.message
        );
        frame.render_widget(
            Paragraph::new(keys).style(
                Style::new()
                    .bg(th.c.secondary)
                    .fg(th.c.secondary_foreground),
            ),
            a.footer,
        );
        let _ = fx::GLYPHS; // the effect module is part of the public surface
    }
}

/// The console screen: the four components homelab writes out by hand, in
/// whichever theme is on. The numbers are fixed so the screen can be
/// compared between themes rather than between moments.
impl App {
    fn draw_console(&self, frame: &mut Frame) {
        let th = &self.theme;
        let screen = frame.area();
        let stage = Stage::new(self.reveal_ms, self.config.motion);
        frame.render_widget(Block::new().style(th.base()), screen);
        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas::<7>(screen);
        let rail_row = rows[1];
        let wizard_row = rows[2];
        let rows = [rows[0], rows[3], rows[4], rows[5], rows[6]];

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    th.label("console"),
                    Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  {} · what homelab hand-rolls, from the crate",
                        th.id.name()
                    ),
                    Style::new().fg(th.c.muted_foreground),
                ),
            ]))
            .style(Style::new().bg(th.c.background)),
            rows[0],
        );

        // The rail runs out of the left over the ground beat.
        frame.render_widget(Rail::new(th).grown(stage.ground()), rail_row);
        frame.render_widget(Stepper::new(th, &WIZARD, self.step), wizard_row);

        // Three meters, one under each threshold and one over. The values
        // roll to their reading rather than snapping to it.
        let panel = Panel::new(th, "Capacity").stage(stage);
        let inner = panel.block().inner(rows[1]);
        frame.render_widget(panel, rows[1]);
        let meters = Layout::vertical([Constraint::Length(1); 3]).areas::<3>(inner);
        for (area, (label, value)) in
            meters
                .iter()
                .zip([("ram", 0.42_f32), ("ssd", 0.78), ("load", 0.94)])
        {
            let shown = roll(
                0.0,
                value as f64,
                self.reveal_ms,
                th.id.fx_duration_ms(),
                self.config.motion,
            ) as f32;
            frame.render_widget(Meter::new(th, label, shown), *area);
        }

        // Two fields; the second has the caret, blinking on the theme's own
        // clock where the theme blinks.
        let panel = Panel::new(th, "Filter").focused(true).stage(stage);
        let inner = panel.block().inner(rows[2]);
        frame.render_widget(panel, rows[2]);
        let fields = Layout::vertical([Constraint::Length(1); 2]).areas::<2>(inner);
        frame.render_widget(Field::new(th, "stack", "media"), fields[0]);
        frame.render_widget(
            Field::new(th, "filter", "web")
                .focused(true)
                .blink(self.reveal_ms / 33),
            fields[1],
        );

        // The stacks, one of them in hand, beside the help overlay that is
        // drawn from the same keymap as the footer.
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)])
                .areas(rows[3]);
        let panel = Panel::new(th, "Stacks").focused(true).stage(stage);
        let inner = panel.block().inner(left);
        frame.render_widget(panel, left);
        let items: Vec<Line<'static>> = STACKS
            .iter()
            .map(|(name, note)| {
                Line::from(vec![
                    Span::styled(
                        (*name).to_string(),
                        Style::new().fg(source_colour(th, name)),
                    ),
                    Span::styled(format!("  {note}"), Style::new().fg(th.c.muted_foreground)),
                ])
            })
            .collect();
        frame.render_widget(SelectList::new(th, &items, self.stack_sel), inner);

        let hints = KeyHints::new(th, CONSOLE_KEYS);
        let panel = Panel::new(th, "Keys").stage(stage);
        let inner = panel.block().inner(right);
        frame.render_widget(panel, right);
        frame.render_widget(
            Paragraph::new(hints.overlay()).style(Style::new().bg(th.c.card)),
            inner,
        );

        frame.render_widget(KeyHints::new(th, CONSOLE_KEYS), rows[4]);

        if self.palette_open {
            CommandPalette::new(th, &self.palette_query, &COMMANDS, self.palette_sel)
                .size((46, 10))
                .blink(self.reveal_ms)
                .render_over(screen, frame.buffer_mut());
            return;
        }

        // And a dialog over all of it, as a restore would be.
        let popup = Popup::new(th, "Restore backup", (52, 7)).kind(PopupKind::Danger);
        let inner = popup.render_over(screen, frame.buffer_mut());
        let lines = vec![
            Line::from(Span::styled(
                "This replaces the running stack with 2026-09-16.",
                Style::new().fg(th.c.popover_foreground),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type the stack's name to confirm:",
                Style::new().fg(th.c.muted_foreground),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::new().bg(th.c.popover)),
            inner,
        );
        let field = Rect {
            y: inner.bottom() - 1,
            height: 1,
            ..inner
        };
        frame.render_widget(
            Field::new(th, "name", "medi")
                .focused(true)
                .blink(self.reveal_ms / 33),
            field,
        );
    }
}

impl App {
    /// The effects screen: everything on it is the theme's, not the
    /// screen's. homelab's `client/src/tui/fx.rs` writes the same six
    /// effects against eighteen colour literals and one fixed look.
    fn draw_effects(&self, frame: &mut Frame) {
        let th = &self.theme;
        let motion = self.config.motion;
        let screen = frame.area();
        let p = th.id.palette();
        frame.render_widget(Block::new().style(th.base()), screen);
        // The ground first: the register's static texture, and the sweep
        // for the one register that declares one.
        Surface::new(th, p.background)
            .at(self.reveal_ms, motion)
            .paint(screen, frame.buffer_mut());

        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas::<5>(screen);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    th.label("effects"),
                    Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} · {}", th.id.name(), describe(th)),
                    Style::new().fg(th.c.muted_foreground),
                ),
            ])),
            rows[0],
        );

        // The alarm, as this register raises it.
        frame.render_widget(
            AlarmPanel::new(
                th,
                "Power lost",
                "ups on battery · 14 minutes of runtime left",
            )
            .at(self.alarm_ms, motion),
            rows[1],
        );

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(rows[2]);

        // A card of its own, so the texture is visible on a plate as well
        // as on the page, and the sweep crosses it on its own clock.
        let panel = Panel::new(th, "Texture");
        let inner = panel.block().inner(left);
        frame.render_widget(panel, left);
        Surface::new(th, p.card)
            .at(self.reveal_ms, motion)
            .id(7)
            .paint(inner, frame.buffer_mut());
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    texture_note(th),
                    Style::new().fg(th.c.card_foreground),
                )),
                Line::from(Span::styled(
                    match th.a.fx.sweep {
                        Some(s) => format!(
                            "a lit row crosses every {:.0} s",
                            s.period_ms as f32 / 1000.0
                        ),
                        None => "nothing crosses it: the register's texture is static".into(),
                    },
                    Style::new().fg(th.c.muted_foreground),
                )),
            ])
            .wrap(Wrap { trim: true }),
            inner,
        );

        // The spinner turning, beside the reveal the theme already had.
        let panel = Panel::new(th, "Waiting");
        let inner = panel.block().inner(right);
        frame.render_widget(panel, right);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        spinner(th, self.reveal_ms, motion),
                        Style::new().fg(th.c.primary),
                    ),
                    Span::styled(
                        format!("  {}", "deploying 2.4 to both hosts"),
                        Style::new().fg(th.c.card_foreground),
                    ),
                ]),
                Line::from(Span::styled(
                    format!(
                        "spinner {:?} · glow {} · glitch {}",
                        th.a.fx.spinner,
                        match th.a.fx.alarm.glow_ms {
                            Some(ms) => format!("{ms} ms"),
                            None => "none".into(),
                        },
                        if th.a.fx.alarm.glitch.is_some() {
                            "yes"
                        } else {
                            "no"
                        }
                    ),
                    Style::new().fg(th.c.muted_foreground),
                )),
            ])
            .wrap(Wrap { trim: true }),
            inner,
        );

        frame.render_widget(
            Ticker::new(th, &self.ticker).at(self.reveal_ms, motion),
            rows[3],
        );
        frame.render_widget(KeyHints::new(th, EFFECT_KEYS), rows[4]);
    }
}

/// What this register declares, in one clause, so the screen says what it
/// is showing rather than only showing it.
fn describe(th: &Theme) -> String {
    let alarm = match (th.a.fx.alarm.strike, th.a.fx.alarm.glitch) {
        (Some(_), Some(_)) => "strikes and comes apart",
        (Some(_), None) => "strikes",
        (None, Some(_)) => "comes apart",
        (None, None) => match th.a.fx.alarm.glow_ms {
            Some(_) => "settles, then glows",
            None => "settles, and holds",
        },
    };
    format!("the alarm {alarm}")
}

fn texture_note(th: &Theme) -> String {
    match th.a.fx.texture {
        Texture::None => "no texture layer: this register paints its ground flat".into(),
        Texture::Scanline { every } => format!("scanlines: one row in {every}"),
        Texture::Grid { cols, rows } => format!("a drafting grid: {cols} by {rows} cells"),
        Texture::Dots { every } => format!("halftone dots: one in {every}, every other row"),
        Texture::Diagonal { every } => format!("a twill: one diagonal in {every}"),
    }
}

const EFFECT_KEYS: &[(&str, &str)] = &[
    ("a", "alarm"),
    ("s", "screen"),
    ("t", "theme"),
    ("m", "motion"),
    ("q", "quit"),
];

/// The five steps homelab's create-container wizard walks
/// (`client/src/tui/view/mod.rs:242`).
const WIZARD: [&str; 5] = ["Preset", "Name", "Resources", "Storage", "Review"];

/// What the console's list holds, and what the palette can find.
const STACKS: [(&str, &str); 5] = [
    ("media", "8 containers"),
    ("web", "3 containers"),
    ("backup", "idle"),
    ("monitoring", "2 containers"),
    ("dns", "1 container"),
];

const COMMANDS: [&str; 8] = [
    "Deploy stack",
    "Restart stack",
    "Roll back to 2.3",
    "Open logs",
    "Stop stack",
    "Prune images",
    "Restore backup",
    "Switch theme",
];

/// One keymap: the footer and the overlay both read this.
const CONSOLE_KEYS: &[(&str, &str)] = &[
    ("p", "palette"),
    ("n", "step"),
    ("s", "screen"),
    ("t", "theme"),
    ("m", "motion"),
    ("Esc", "close"),
    ("q", "quit"),
];
