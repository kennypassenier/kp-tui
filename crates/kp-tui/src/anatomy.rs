//! What a terminal can carry of a theme beyond colour: border glyphs, text
//! modifiers, case, prefixes, the cursor and the reveal routine.
//!
//! Hand-written, not generated: `tokens.json` has no field for any of this.
//! Each choice cites where it comes from; `GUESS` marks a choice no source
//! states, so a reviewer knows which ones to judge.

use crossterm::cursor::SetCursorStyle;
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
