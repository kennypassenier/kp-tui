//! What a terminal can carry of a theme beyond colour: border glyphs, text
//! modifiers, case, prefixes, the cursor and the reveal routine.
//!
//! Hand-written, not generated: `tokens.json` has no field for any of this.
//! Each choice cites where it comes from; `GUESS` marks a choice no source
//! states, so a reviewer knows which ones to judge.

use crossterm::cursor::SetCursorStyle;

use crate::effects::{Glitch, Spinner, Strike, Sweep};
use ratatui::{style::Modifier, symbols::border};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reveal {
    /// Fade in whole, no character touched. formal `arrive`.
    Arrive { ms: u32 },
    /// Characters churn through glyphs, then settle left to right.
    /// cyberpunk `decipher`.
    Decipher { cps: f32, lead_ms: u32, swap: f32 },
    /// Typed one glyph at a time with a block caret on the last.
    /// terminal `type`.
    Type { cps: f32 },
    /// Whole words arriving one after another, `stagger_ms` apart: the
    /// register's `--kp-word-stagger`, which six themes declare and which
    /// `Arrive` threw away [scope-127]. No character is touched, only the
    /// moment a word becomes its own colour.
    Words { ms: u32, stagger_ms: u32 },
}

/// How a button's plate ends. Every theme's button is a filled plate with
/// its label centred on it — the shape a modern terminal interface uses,
/// and what the registers draw on the web. What differs is the edge and
/// what sits around the label (Kenny, 2026-09-17: a button must look
/// "strak", not like drawn geometry).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonFace {
    /// Half-block caps, so the plate ends mid-cell: the nearest a cell grid
    /// has to a corner radius.
    Soft,
    /// The plate runs to the full cell on both sides. A radius of 0.
    Square,
    /// Square, with the register's brackets around the label.
    Bracket,
}

/// The line under a table's header, as a cell grid can draw it: the
/// registers write 1px, 2px and 3px, and one writes a gradient.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// 1px — the package base.
    Thin,
    /// 2px.
    Heavy,
    /// 3px.
    Double,
    /// synthwave's `--kp-stripe`: one ramp from `--primary` to `--accent`
    /// across the whole row, and the only gradient rule in the set.
    Gradient,
}

/// How a register dresses a table's header.
///
/// Measured across the twenty-two on 2026-09-18. Six give the header a
/// plate of its own and sixteen leave it transparent; the rule under it is
/// 1px in seventeen, 2px in three, 3px in two, and a gradient in one.
///
/// What is deliberately NOT carried over: the 1px `--border` rule the base
/// draws between body rows. On a page that is one pixel; in a cell grid it
/// is a whole row, which halves how many records fit. The grid already
/// separates the rows, so the header rule — the one every register
/// re-states — is the one that is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableHead {
    pub plate: Tone,
    pub ink: Tone,
    pub uppercase: bool,
    pub modifier: Modifier,
    pub rule: Rule,
    pub rule_tone: Tone,
}

/// The package base: no plate, muted ink, weight 600, a 1px
/// `--border-strong` rule (`css/components.css:1373`).
pub const BASE_HEAD: TableHead = TableHead {
    plate: Tone::None,
    ink: Tone::MutedInk,
    uppercase: false,
    modifier: Modifier::BOLD,
    rule: Rule::Thin,
    rule_tone: Tone::Line,
};

/// A colour by the role it plays, so a row of the table below can name
/// one without reaching into the palette itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// No colour at all: the ground the row already sits on.
    None,
    Background,
    Card,
    Muted,
    Line,
    Ink,
    Primary,
    PrimaryInk,
    Secondary,
    SecondaryInk,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
    MutedInk,
}

/// How a register paints the row a person is standing on.
///
/// Measured from the twenty-two registers on 2026-09-17: four plate it in
/// `--primary`, one in `--muted`, and the rest either tint it with another
/// token or leave the ground alone and speak with a bar, a bracket or a
/// weight. The package's own base rule (`css/components.css:2359`,
/// `background: var(--muted); font-weight: 600`) is what a silent register
/// inherits.
///
/// A terminal has no pseudo-element, so a register's `::before` bar or
/// bracket becomes a leading glyph in the same colour — the nearest a cell
/// grid has to a rule drawn down the leading edge.
///
/// A register that writes `background: none` still inherits the base
/// rule's `font-weight: 600`, so every row here with `plate: Tone::None`
/// carries BOLD. Without it, a theme whose selected ink equals its body
/// ink — light is one — would show no selection at all in a terminal,
/// which the web never does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub plate: Tone,
    pub ink: Tone,
    /// The glyph before the label, "" for a register that adds none.
    pub marker: &'static str,
    pub marker_tone: Tone,
    /// `font-weight`, `text-decoration` on the selected state.
    pub modifier: Modifier,
    /// A register that sets `letter-spacing` on it.
    pub spaced: bool,
}

/// How a register closes the ends of a progress bar.
///
/// Measured across all 22 registers' own `.kp-progress` rule, 2026-09-19,
/// after Kenny asked for elements that differ per theme rather than only
/// per palette: "ook elementen zoals een progressbar moet uniek zijn per
/// thema". A page has a border and a radius; a cell grid has the two cells
/// either side of the bar, so that is where the same decision lands.
///
/// Eight registers draw no border at all and get no ends. Of the fourteen
/// that do, the radius says which ends: `0` is square, a pill radius is
/// round, anything between is a thin rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    /// No border on `.kp-progress`: the bar is a plate and nothing else —
    /// cyberpunk, formal, light, pastel, sepia, shade-dark, shade-light,
    /// solstice.
    Plain,
    /// A border at `border-radius: 0` — brutalism, deco, grotesk, phantom,
    /// retro, synthwave, terminal.
    Square,
    /// A border at `border-radius: 999px` — nostromo alone.
    Round,
    /// A border at a radius between the two — blueprint, dark, forest,
    /// high-contrast, lapis, titanium.
    Rail,
}

impl Track {
    /// The two cells either side of the bar, and how many cells they cost.
    pub const fn ends(self) -> Option<(&'static str, &'static str)> {
        match self {
            Track::Plain => None,
            Track::Square => Some(("[", "]")),
            Track::Round => Some(("(", ")")),
            Track::Rail => Some(("▏", "▕")),
        }
    }
}

/// No marker, no weight: the row is told apart by its plate alone.
pub const fn plated(plate: Tone, ink: Tone) -> Selection {
    Selection {
        plate,
        ink,
        marker: "",
        marker_tone: Tone::None,
        modifier: Modifier::empty(),
        spaced: false,
    }
}

/// The static texture a register paints on its ground (DI9), as near as a
/// cell grid comes to it. A wash of one dim glyph, never a colour change:
/// the CSS layers sit between 2 % and 6 % alpha, and anything louder in a
/// terminal reads as content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Texture {
    /// No texture layer, or one too fine for a cell (grain, contour).
    None,
    /// Dim every `every`-th row: the scanline registers.
    Scanline { every: u16 },
    /// A dim rule every `cols` columns and every `rows` rows; `0` leaves
    /// that axis alone.
    Grid { cols: u16, rows: u16 },
    /// A dim dot every `every` cells, on alternating rows: halftone, dot
    /// grid, checkerboard.
    Dots { every: u16 },
    /// A dim diagonal every `every` cells: twill, chevrons.
    Diagonal { every: u16 },
}

impl Texture {
    /// The two glyphs a progress bar is woven from in this theme: what a
    /// filled cell carries, and what an empty one does.
    ///
    /// Kenny, 2026-09-20: *"vooruitgangsbalken mogen wat textuur hebben,
    /// zoals de oude vooruitgangsbalken. nu is het te 'plat'"*. A bar
    /// painted as a plain plate reads as a plate; homelab's own bars were
    /// block characters. Rather than invent a twenty-third decision, the
    /// bar wears the weave its own register already declares for the
    /// ground, so no two themes with different grounds share a bar
    /// [fix-66].
    pub const fn weave(self) -> (&'static str, &'static str) {
        match self {
            // A register with no texture of its own keeps the solid block
            // homelab used, over a track that is visible but quiet.
            Texture::None => ("█", "░"),
            Texture::Scanline { .. } => ("▓", "░"),
            Texture::Grid { .. } => ("▒", "░"),
            Texture::Dots { .. } => ("⣿", "⣀"),
            Texture::Diagonal { .. } => ("▨", "░"),
        }
    }
}

/// The theme's attention treatment, which every register answers for
/// itself. The package baseline (`css/components.css`, `kp-alarm-*`) is a
/// settle of 240/520/480 ms once and a glow at 1400 ms
/// `infinite alternate`; a register that sets the glow to alpha 0 has no
/// loop left, and `glow_ms: None` says so.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alarm {
    /// The panel striking as it arrives.
    pub strike: Option<Strike>,
    /// Wrong characters through the headline.
    pub glitch: Option<Glitch>,
    /// The breathing glow's period; `None` for the registers where it is
    /// invisible or refused.
    pub glow_ms: Option<u32>,
}

/// What moves, and what turns, in a theme. One field on the anatomy so a
/// widget asks the theme rather than naming an effect of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fx {
    pub texture: Texture,
    /// A lit row crossing a panel. Only for a register that declares a
    /// sweep; every other texture here is static, and DI5 refused a loop
    /// in most of them.
    pub sweep: Option<Sweep>,
    pub alarm: Alarm,
    pub spinner: Spinner,
}

/// The baseline every register inherits until it says otherwise.
pub const BASELINE_ALARM: Alarm = Alarm {
    strike: None,
    glitch: None,
    glow_ms: Some(1400),
};

/// The baseline with the glow off, for a register that sets its alpha to 0.
pub const STILL_ALARM: Alarm = Alarm {
    strike: None,
    glitch: None,
    glow_ms: None,
};

#[derive(Clone, Copy, Debug)]
pub struct Anatomy {
    pub border: border::Set<'static>,
    pub border_focus: border::Set<'static>,
    /// The button's own frame. Kept for a theme that frames something else
    /// the same way; the button itself is a plate.
    pub button_border: border::Set<'static>,
    /// How the plate ends.
    pub button_face: ButtonFace,
    /// A space between the label's characters, for a register that sets
    /// `letter-spacing` on its buttons.
    pub button_spaced: bool,
    pub title_modifier: Modifier,
    pub uppercase_labels: bool,
    /// Before a microlabel (a panel title).
    pub label_prefix: &'static str,
    /// Around a button label.
    pub button_brackets: (&'static str, &'static str),
    /// Added to a focused button's label.
    pub focus_modifier: Modifier,
    pub tab_divider: &'static str,
    pub selected_tab_modifier: Modifier,
    pub cursor: SetCursorStyle,
    pub reveal: Reveal,
    /// The row a person is standing on.
    pub selection: Selection,
    /// How this register dresses a table's header.
    pub table: TableHead,
    /// The rule across the top of a screen. Gradient only where the
    /// register paints a gradient onto a rule (`border-image`): synthwave
    /// eight times, terminal and retro three each, and nowhere else.
    pub rail: Rule,
    /// Corner marks on the panel with the focus. Only for the registers
    /// that cut their corners in earnest — `clip-path` five times or more
    /// — so a theme that never cuts a corner does not grow one here.
    pub hud: bool,

    /// How this register closes the ends of a progress bar, from its own
    /// `.kp-progress` rule [Track].
    pub meter: Track,
    /// What moves: the texture, the alarm, the spinner.
    pub fx: Fx,
}

/// cyberpunk's notch: the bottom-right corner cut on the diagonal
/// (`clip-path` on `.kp-button`, `css/cyberpunk-register.css`).
pub const NOTCHED: border::Set<'static> = border::Set {
    bottom_right: "◢",
    ..border::PLAIN
};

/// dark's chamfer: the top-right and bottom-left corners cut on the
/// diagonal (`clip-path` in `css/dark-register.css`, on the controls and
/// on the panels).
pub const CHAMFER_FALL: border::Set<'static> = border::Set {
    top_right: "◥",
    bottom_left: "◣",
    ..border::PLAIN
};

/// titanium's chamfer: the same cut, mirrored (`css/titanium-register.css`).
pub const CHAMFER_RISE: border::Set<'static> = border::Set {
    top_left: "◤",
    bottom_right: "◢",
    ..border::PLAIN
};

/// phantom's panel: no frame, one heavy bar down the left edge
/// (`.kp-card { border: 0; border-left: 5px solid var(--primary) }`).
pub const PHANTOM_BAR: border::Set<'static> = border::Set {
    vertical_left: "┃",
    ..border::PLAIN
};

/// formal. Paper and ink; restraint; one rule that doubles.
pub const FORMAL: Anatomy = Anatomy {
    // GUESS: `--radius: 0.375rem` is the only radius > 0 of the three; a
    // rounded corner glyph is the nearest a cell grid has. PLAIN is the
    // alternative if rounded reads as too soft for print.
    border: border::ROUNDED,
    // anatomy.md "The rule doubles under the pointer (gap-4)";
    // formal-register.css `.kp-button:focus-visible::after`.
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    // formal-register.css `.kp-button--primary`: a filled plate, and
    // `--radius: 0.375rem` rounds it. Half-block caps are that radius in a
    // cell grid.
    button_face: ButtonFace::Soft,
    button_spaced: false,
    // GUESS: Fraunces headings cannot exist in a terminal; bold is the only
    // weight channel left.
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    // formal-register.css `.kp-tab[aria-selected]`: primary colour and a
    // primary border; the underline is the border a cell row can carry.
    selected_tab_modifier: Modifier::UNDERLINED,
    // GUESS: nothing states a caret; a steady bar is the quietest.
    cursor: SetCursorStyle::SteadyBar,
    // anatomy.md "The headline arrives whole"; register: opacity 450ms.
    reveal: Reveal::Arrive { ms: 450 },
    // formal-register.css:29 "nothing here glitches, glows, loops or
    // flickers"; the laid-paper grain is 3.5 % and finer than a cell.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Braille,
    },
    // formal-register.css: the current sidenav link takes `background:
    // var(--muted)` with `color: var(--primary)`.
    selection: plated(Tone::Muted, Tone::Primary),
    // formal-register.css restates the base: muted ink, weight 600, a 1px
    // --border-strong rule.
    table: BASE_HEAD,
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 0 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Plain,
};

/// cyberpunk. Signal yellow on a void; square or notched; a machine voice.
pub const CYBERPUNK: Anatomy = Anatomy {
    // anatomy.md "The radius is 0".
    border: border::PLAIN,
    // GUESS: the web focus is a ring in `--ring`; THICK is the heavier line
    // a terminal has.
    border_focus: border::THICK,
    button_border: NOTCHED,
    // anatomy.md "The radius is 0": the plate keeps its full cells. The
    // register's slit and its 14px corner cut were tried as glyphs on
    // 2026-09-17 and thrown out — at one cell they read as drawn decoration
    // rather than as a cut edge.
    button_face: ButtonFace::Square,
    // `letter-spacing: 0.12em` on an uppercase label.
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    // anatomy.md "uppercase, spaced, prefixed"; register `.microlabel`.
    uppercase_labels: true,
    // cyberpunk-register.css `.microlabel::before { content: '/// ' }`.
    label_prefix: "/// ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::BOLD,
    // cyberpunk-register.css `.kp-breadcrumb li + li::before { content: '//' }`.
    tab_divider: " // ",
    // `.kp-tab[aria-selected] { border-bottom: 2px solid var(--primary) }`.
    selected_tab_modifier: Modifier::BOLD,
    // GUESS: no caret is stated; a steady block reads as a HUD field.
    cursor: SetCursorStyle::SteadyBlock,
    // cyberpunk-register.css `--kp-decipher-cps: 26; --kp-decipher-lead:
    // 260ms; --kp-decipher-swap: 0.5`.
    reveal: Reveal::Decipher {
        cps: 26.0,
        lead_ms: 260,
        swap: 0.5,
    },
    // cyberpunk-register.css:62 scanlines 1px/3px;
    // kp-alarm-cyberpunk-flicker 600ms [scope-100]; kp-alarm-jitter/slice
    // at 5000ms and kp-alarm-sweep at 6000ms. The glow is the sweep here,
    // so no separate one.
    fx: Fx {
        texture: Texture::Scanline { every: 3 },
        sweep: Some(Sweep {
            period_ms: 6000,
            sweep_ms: 1200,
        }),
        alarm: Alarm {
            strike: Some(Strike { ms: 600 }),
            glitch: Some(Glitch {
                every_ms: 5000,
                hold_ms: 300,
                share: 0.35,
            }),
            glow_ms: None,
        },
        spinner: Spinner::Half,
    },
    // cyberpunk-register.css:1736: `--primary` plate,
    // `--primary-foreground` ink, and a notched corner a cell cannot cut.
    selection: plated(Tone::Primary, Tone::PrimaryInk),
    // cyberpunk-register.css: mono .72rem in --accent, uppercase,
    // letter-spacing .14em, and the rule takes --primary instead of the
    // border colour.
    table: TableHead {
        ink: Tone::Accent,
        uppercase: true,
        modifier: Modifier::empty(),
        rule_tone: Tone::Primary,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 42 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Plain,
};

/// terminal. A phosphor CRT that accepts a terminal's constraints.
pub const TERMINAL: Anatomy = Anatomy {
    // anatomy.md "the panels are TUI panels"; `--radius: 0`.
    border: border::PLAIN,
    // GUESS: the register moves the border to `--foreground` on focus and
    // keeps the line; the colour does the work, the glyph stays plain.
    border_focus: border::PLAIN,
    button_border: border::PLAIN,
    // anatomy.md "the buttons are brackets and plates": the plate, with the
    // brackets below around the label. `--radius: 0`, so square.
    button_face: ButtonFace::Bracket,
    button_spaced: false,
    // GUESS: the bloom on titles cannot exist; bold is the brightening a
    // terminal offers.
    title_modifier: Modifier::BOLD,
    // terminal-register.css `.kp-tab { text-transform: uppercase }`.
    uppercase_labels: true,
    // terminal-register.css `content: '$ '` before a microlabel.
    label_prefix: "$ ",
    // anatomy.md "the buttons are brackets and plates"; register `'[ '`, `' ]'`.
    button_brackets: ("[ ", " ]"),
    // anatomy.md "Every hover is inverse video" (hover, applied to focus
    // here because a keyboard TUI has no pointer: GUESS).
    focus_modifier: Modifier::REVERSED,
    tab_divider: "  ",
    // `.kp-tab[aria-selected] { background: var(--primary) }`: a plate.
    selected_tab_modifier: Modifier::empty(),
    // anatomy.md "a block of one character cell ... blinking once a second".
    cursor: SetCursorStyle::BlinkingBlock,
    // terminal-register.css `--kp-decipher-cps: 29.41`, the `type` routine.
    reveal: Reveal::Type { cps: 29.41 },
    // terminal-register.css:31 static raster scanlines 1px/3px at 6 % — and
    // "flicker at .15s infinite was measured and refused (DI5)", so the
    // raster does not move.
    fx: Fx {
        texture: Texture::Scanline { every: 3 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Ascii,
    },
    // terminal-register.css:1528: the `--primary` plate, and :1534 a
    // `::before` carrying a literal `>` in the plate's own ink — the only
    // register whose marker is a glyph rather than a rule.
    selection: Selection {
        marker: "> ",
        marker_tone: Tone::PrimaryInk,
        ..plated(Tone::Primary, Tone::PrimaryInk)
    },
    // terminal-register.css: uppercase, letter-spacing .12em, and weight
    // 400 — the only register that takes the base weight back off.
    table: TableHead {
        uppercase: true,
        modifier: Modifier::empty(),
        ..BASE_HEAD
    },
    // rail: a gradient on a rule, as this register paints one. hud: clip-path 0 times.
    rail: Rule::Gradient,
    hud: false,
    meter: Track::Square,
};

/// light. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const LIGHT: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::THICK,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 620 },
    // light-register.css: millimetre grid at 24px and 120px, 5 %.
    fx: Fx {
        texture: Texture::Grid { cols: 12, rows: 6 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // light-register.css: the current link keeps the ground and takes
    // `--foreground`.
    selection: Selection {
        modifier: Modifier::BOLD,
        ..plated(Tone::None, Tone::Ink)
    },
    // light-register.css: a --muted plate under the header, body ink on it.
    table: TableHead {
        plate: Tone::Muted,
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 5 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Plain,
};

/// dark. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const DARK: Anatomy = Anatomy {
    border: CHAMFER_FALL,
    border_focus: border::THICK,
    button_border: CHAMFER_FALL,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "· ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::BOLD,
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Words {
        ms: 640,
        stagger_ms: 28,
    },
    // dark-register.css: a 1px instrument grid every 5.5rem at 4.5 %.
    fx: Fx {
        texture: Texture::Grid { cols: 22, rows: 11 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // dark-register.css: an `--accent` plate under `--foreground`.
    selection: plated(Tone::Accent, Tone::Ink),
    // dark-register.css keeps the base header and only adds a --card hover
    // plate, which is the selected row here.
    table: BASE_HEAD,
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 12 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Rail,
};

/// synthwave. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const SYNTHWAVE: Anatomy = Anatomy {
    border: border::PLAIN,
    border_focus: border::DOUBLE,
    button_border: border::PLAIN,
    button_face: ButtonFace::Square,
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    uppercase_labels: true,
    label_prefix: "▶ ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Arrive { ms: 700 },
    // synthwave-register.css:23 "Nothing here flickers"; the 2px row raster
    // and the 3px RGB stripe are static.
    fx: Fx {
        texture: Texture::Scanline { every: 2 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Bar,
    },
    // synthwave-register.css: no plate; an inset `--primary` bar with a
    // glow, which in a cell grid is the bar without the glow.
    selection: Selection {
        modifier: Modifier::BOLD,
        marker: "▌",
        marker_tone: Tone::Primary,
        ..plated(Tone::None, Tone::Ink)
    },
    // synthwave-register.css: the OSD face at 1rem, uppercase,
    // letter-spacing .12em, and a 2px --kp-stripe gradient on the header
    // row with the cell borders set transparent.
    table: TableHead {
        ink: Tone::Primary,
        uppercase: true,
        modifier: Modifier::empty(),
        rule: Rule::Gradient,
        ..BASE_HEAD
    },
    // rail: a gradient on a rule, as this register paints one. hud: clip-path 4 times.
    rail: Rule::Gradient,
    hud: false,
    meter: Track::Square,
};

/// pastel. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const PASTEL: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::THICK,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 650 },
    // pastel-register.css:1309 the sticker lands, the plate fills and the
    // overprint registers — 420/420/650 ms, "Nothing loops."
    fx: Fx {
        texture: Texture::Dots { every: 3 },
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Braille,
    },
    // pastel-register.css: a `--primary` plate with its own ink.
    selection: plated(Tone::Primary, Tone::PrimaryInk),
    // pastel-register.css restates the base with body ink.
    table: TableHead {
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 2 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Plain,
};

/// forest. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const FOREST: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::THICK,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "◦ ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 500 },
    // forest-register.css:25 "Nothing here flickers"; the contour field is
    // an SVG at 6 % with no cell equivalent.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // forest-register.css: the ground is left alone and the label takes
    // `--primary`.
    selection: Selection {
        modifier: Modifier::BOLD,
        ..plated(Tone::None, Tone::Primary)
    },
    // forest-register.css: the display face at weight 700, and a 2px
    // --foreground rule.
    table: TableHead {
        ink: Tone::Ink,
        rule: Rule::Heavy,
        rule_tone: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 2 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Rail,
};

/// high-contrast. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const HIGH_CONTRAST: Anatomy = Anatomy {
    border: border::THICK,
    border_focus: border::DOUBLE,
    button_border: border::THICK,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Arrive { ms: 550 },
    // high-contrast-register.css:14 "nothing loops, nothing blinks", and it
    // declares no texture at all.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Half,
    },
    // high-contrast-register.css: no plate, inset `--foreground` bars and
    // `font-weight: 700` — this theme says everything with the ink it has.
    selection: Selection {
        marker: "▌",
        marker_tone: Tone::Ink,
        modifier: Modifier::BOLD,
        ..plated(Tone::None, Tone::Ink)
    },
    // high-contrast-register.css: weight 700 and a 2px --border-strong
    // rule, inside a 2px frame.
    table: TableHead {
        ink: Tone::Ink,
        rule: Rule::Heavy,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 4 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Rail,
};

/// sepia. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const SEPIA: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 1050 },
    // sepia-register.css:70 "Nothing loops, nothing oscillates"; it
    // declares no --fx-texture.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // sepia-register.css: a `--card` plate, `--primary` ink, and a
    // `::before` bracket in `--primary`.
    selection: Selection {
        marker: "▏",
        marker_tone: Tone::Primary,
        ..plated(Tone::Card, Tone::Primary)
    },
    // sepia-register.css: a --card plate on the header over a --popover
    // table ground.
    table: TableHead {
        plate: Tone::Card,
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 0 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Plain,
};

/// blueprint. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const BLUEPRINT: Anatomy = Anatomy {
    border: border::PLAIN,
    border_focus: border::THICK,
    button_border: border::PLAIN,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 300 },
    // blueprint-register.css: a drafting grid at 32px with crosshairs every
    // 160px, 5.5 %.
    fx: Fx {
        texture: Texture::Grid { cols: 16, rows: 8 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // blueprint-register.css: `background: none`, and a `::before` bar in
    // `--border-strong` — a construction line rather than a plate.
    selection: Selection {
        modifier: Modifier::BOLD,
        marker: "▌",
        marker_tone: Tone::Line,
        ..plated(Tone::None, Tone::Ink)
    },
    // blueprint-register.css: a --card plate, separate borders, and a
    // framed datatable.
    table: TableHead {
        plate: Tone::Card,
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 1 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Rail,
};

/// solstice. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const SOLSTICE: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::THICK,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 740 },
    // solstice-register.css:14 "Nothing here is a bevel, a scanline or a
    // grid"; :1012 "Once, never in a loop".
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // solstice-register.css: a `--primary` gradient fading to transparent,
    // plus an inset `--primary` bar. A cell grid cannot fade, so the bar is
    // what survives.
    selection: Selection {
        modifier: Modifier::BOLD,
        marker: "▌",
        marker_tone: Tone::Primary,
        ..plated(Tone::None, Tone::Ink)
    },
    // solstice-register.css leaves the base alone, hover included.
    table: BASE_HEAD,
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 5 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Plain,
};

/// brutalism. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const BRUTALISM: Anatomy = Anatomy {
    border: border::THICK,
    border_focus: border::DOUBLE,
    button_border: border::THICK,
    button_face: ButtonFace::Square,
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    uppercase_labels: true,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: "  ",
    selected_tab_modifier: Modifier::empty(),
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Words {
        ms: 260,
        stagger_ms: 60,
    },
    // brutalism-register.css: a dot grid, 1.6px dots every 20px; the alarm
    // glow is at alpha 0.
    fx: Fx {
        texture: Texture::Dots { every: 10 },
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Half,
    },
    // brutalism-register.css: a `--secondary` slab with its own ink — the
    // only register that selects in the secondary colour.
    selection: plated(Tone::Secondary, Tone::SecondaryInk),
    // brutalism-register.css: a --secondary slab, weight 700, uppercase,
    // letter-spacing .08em, and a 3px rule inside a 3px frame.
    table: TableHead {
        plate: Tone::Secondary,
        ink: Tone::SecondaryInk,
        uppercase: true,
        rule: Rule::Double,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 0 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Square,
};

/// deco. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const DECO: Anatomy = Anatomy {
    border: border::PLAIN,
    border_focus: border::PLAIN,
    button_border: border::PLAIN,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: true,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 520 },
    // deco-register.css: a chevron pair every 24px at 5 %.
    fx: Fx {
        texture: Texture::Diagonal { every: 12 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // deco-register.css: no plate, `--primary` ink and `letter-spacing:
    // 0.1em` — the same spacing its buttons carry.
    selection: Selection {
        modifier: Modifier::BOLD,
        spaced: true,
        ..plated(Tone::None, Tone::Primary)
    },
    // deco-register.css: the display face in --primary, uppercase,
    // letter-spacing .08em, weight 400.
    table: TableHead {
        ink: Tone::Primary,
        uppercase: true,
        modifier: Modifier::empty(),
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 1 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Square,
};

/// phantom. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const PHANTOM: Anatomy = Anatomy {
    border: PHANTOM_BAR,
    border_focus: border::DOUBLE,
    button_border: PHANTOM_BAR,
    button_face: ButtonFace::Square,
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    uppercase_labels: true,
    label_prefix: "▮ ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: "  ",
    selected_tab_modifier: Modifier::empty(),
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Words {
        ms: 620,
        stagger_ms: 28,
    },
    // phantom-register.css:22 "Nothing here flickers: the halftone and the
    // scan are static" — 7px halftone over a 1px/3px scan at 14 %.
    fx: Fx {
        texture: Texture::Scanline { every: 3 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // phantom-register.css: the `::before` bar grows to `inline-size:
    // 100%`, which is a `--primary` plate by another road.
    selection: plated(Tone::Primary, Tone::PrimaryInk),
    // phantom-register.css: the display face at weight 700, uppercase,
    // letter-spacing .1em, and a 3px --foreground rule.
    table: TableHead {
        ink: Tone::Ink,
        uppercase: true,
        rule: Rule::Double,
        rule_tone: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 13 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Square,
};

/// shade-light. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const SHADE_LIGHT: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Words {
        ms: 520,
        stagger_ms: 70,
    },
    // shade-light-register.css:29 "nothing loops, nothing blinks"; only the
    // divider seam is textured.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // shade-light-register.css: the row drops to `--background` with a
    // shadow, and the label takes `--primary`.
    selection: plated(Tone::Background, Tone::Primary),
    // shade-light-register.css: a --card plate on a --card table ground.
    table: TableHead {
        plate: Tone::Card,
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 3 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Plain,
};

/// shade-dark. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const SHADE_DARK: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Words {
        ms: 600,
        stagger_ms: 90,
    },
    // shade-dark-register.css:19 "Nothing loops" — "no starfield, no
    // grain".
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // shade-dark-register.css: the same drop to `--background`, inset
    // shadows, `--foreground` ink.
    selection: plated(Tone::Background, Tone::Ink),
    // shade-dark-register.css keeps the base and adds a square inset focus
    // ring on a cell.
    table: BASE_HEAD,
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 0 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Plain,
};

/// retro. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const RETRO: Anatomy = Anatomy {
    border: border::DOUBLE,
    border_focus: border::DOUBLE,
    button_border: border::DOUBLE,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "> ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::REVERSED,
    tab_divider: "  ",
    selected_tab_modifier: Modifier::BOLD,
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Arrive { ms: 640 },
    // retro-register.css:33 "nothing loops and nothing blinks"; the ground
    // is a 4px conic checkerboard at 4 %.
    fx: Fx {
        texture: Texture::Dots { every: 2 },
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Bar,
    },
    // retro-register.css: a `--primary` plate with its own ink, the way a
    // selected item in that era always looked.
    selection: plated(Tone::Primary, Tone::PrimaryInk),
    // retro-register.css: a --card plate with a bevel and a rule on all
    // four sides of the header cells; a cell grid keeps the plate and the
    // rule under it.
    table: TableHead {
        plate: Tone::Card,
        ink: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a gradient on a rule, as this register paints one. hud: clip-path 8 times.
    rail: Rule::Gradient,
    hud: true,
    meter: Track::Square,
};

/// grotesk. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const GROTESK: Anatomy = Anatomy {
    border: border::THICK,
    border_focus: border::THICK,
    button_border: border::THICK,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::REVERSED,
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 640 },
    // grotesk-register.css:25 "Nothing here flickers or loops"; the texture
    // is twelve column rules at 5 %, vertical only.
    fx: Fx {
        texture: Texture::Grid { cols: 8, rows: 0 },
        sweep: None,
        alarm: STILL_ALARM,
        spinner: Spinner::Quadrant,
    },
    // grotesk-register.css: no plate at all; `font-weight: 800` and a
    // tightened tracking do the work, and BOLD is the weight a terminal
    // has.
    selection: Selection {
        modifier: Modifier::BOLD,
        ..plated(Tone::None, Tone::Ink)
    },
    // grotesk-register.css: the display face at weight 800 and a 3px
    // --foreground rule, the heaviest plain rule in the set.
    table: TableHead {
        ink: Tone::Ink,
        rule: Rule::Double,
        rule_tone: Tone::Ink,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 0 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Square,
};

/// lapis. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const LAPIS: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::THICK,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: "  ",
    selected_tab_modifier: Modifier::empty(),
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Arrive { ms: 900 },
    // lapis-register.css:35 "Nothing here flickers or loops"; the page
    // texture layer is at opacity 0.
    fx: Fx {
        texture: Texture::None,
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Braille,
    },
    // lapis-register.css: the ground stays, the label takes `--primary`.
    selection: Selection {
        modifier: Modifier::BOLD,
        ..plated(Tone::None, Tone::Primary)
    },
    // lapis-register.css: the mono face, uppercase, letter-spacing .06em.
    table: TableHead {
        uppercase: true,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 6 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Rail,
};

/// nostromo. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const NOSTROMO: Anatomy = Anatomy {
    border: border::ROUNDED,
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    button_face: ButtonFace::Soft,
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    uppercase_labels: true,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: "  ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBlock,
    reveal: Reveal::Arrive { ms: 340 },
    // nostromo-register.css:29 "The only loops on this page: none"; the
    // ground is 1px vent ribs every 6px at 5 %.
    fx: Fx {
        texture: Texture::Scanline { every: 6 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Ascii,
    },
    // nostromo-register.css declares no plate of its own, so it inherits
    // the package base (`--muted`); its `::before` dot in `--primary` is
    // its own.
    selection: Selection {
        marker: "•",
        marker_tone: Tone::Primary,
        ..plated(Tone::Muted, Tone::Ink)
    },
    // nostromo-register.css: mono .72rem at weight 600, uppercase,
    // letter-spacing .08em.
    table: TableHead {
        uppercase: true,
        ..BASE_HEAD
    },
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 4 times.
    rail: Rule::Thin,
    hud: false,
    meter: Track::Round,
};

/// titanium. Proposed in research/ratatui/ANATOMY_PROPOSAL.md, which cites the
/// register line behind every field and marks the guesses [scope-127].
pub const TITANIUM: Anatomy = Anatomy {
    border: CHAMFER_RISE,
    border_focus: border::DOUBLE,
    button_border: CHAMFER_RISE,
    button_face: ButtonFace::Square,
    button_spaced: false,
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "· ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    selected_tab_modifier: Modifier::UNDERLINED,
    cursor: SetCursorStyle::SteadyBar,
    reveal: Reveal::Words {
        ms: 640,
        stagger_ms: 28,
    },
    // titanium-register.css: a carbon twill at +/-45 degrees, 2px lines
    // every 9px.
    fx: Fx {
        texture: Texture::Diagonal { every: 5 },
        sweep: None,
        alarm: BASELINE_ALARM,
        spinner: Spinner::Quadrant,
    },
    // titanium-register.css: an `--accent` plate under `--foreground`.
    selection: plated(Tone::Accent, Tone::Ink),
    // titanium-register.css keeps the base header; its --card hover plate
    // is the selected row here.
    table: BASE_HEAD,
    // rail: a plain rule; this register paints no gradient onto one. hud: clip-path 6 times.
    rail: Rule::Thin,
    hud: true,
    meter: Track::Rail,
};

/// Every theme's anatomy, in `themes/order.json`'s order, so `ThemeId`
/// indexes straight into it.
pub const ANATOMIES: [&Anatomy; 22] = [
    &FORMAL,
    &LIGHT,
    &DARK,
    &CYBERPUNK,
    &SYNTHWAVE,
    &PASTEL,
    &TERMINAL,
    &FOREST,
    &HIGH_CONTRAST,
    &SEPIA,
    &BLUEPRINT,
    &SOLSTICE,
    &BRUTALISM,
    &DECO,
    &PHANTOM,
    &SHADE_LIGHT,
    &SHADE_DARK,
    &RETRO,
    &GROTESK,
    &LAPIS,
    &NOSTROMO,
    &TITANIUM,
];
