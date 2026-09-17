//! The reveal, hand-written: a pure function of (text, routine, elapsed
//! time, reduced motion). No state is kept between frames, so replaying,
//! testing and switching theme mid-reveal are all free.

use crate::anatomy::Reveal;

/// The glyphs a headline deciphers through: `GLYPHS` in `js/effects.js`
/// (AR40, "no block glyphs").
pub const GLYPHS: &[char] = &[
    '0', '1', '<', '>', '/', '\\', '|', '=', '+', '*', '#', '%', '@', '&', '$', '?', '!', 'Z', 'X',
    'K', 'Q',
];

/// Motion preference. No terminal standard exists for
/// `prefers-reduced-motion`; the demo reads `--reduced-motion`,
/// `KP_REDUCED_MOTION=1`, the config file, and the `m` key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Full,
    Reduced,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RevealFrame {
    pub text: String,
    /// Column of a block caret, if one rides the text.
    pub caret: Option<usize>,
    /// 0.0 = the text is the ground's colour, 1.0 = its own.
    pub opacity: f32,
    pub done: bool,
    /// For `Words`: the opacity of each word, in order, with the spaces
    /// between them carried on the word before. Empty for every other
    /// routine, which fades or types the whole line at once.
    pub words: Vec<(String, f32)>,
}

pub fn duration_ms(reveal: Reveal, text: &str) -> u32 {
    let n = text.chars().count() as f32;
    match reveal {
        Reveal::Arrive { ms } => ms,
        Reveal::Decipher { cps, lead_ms, .. } => lead_ms + (n * 1000.0 / cps) as u32,
        Reveal::Type { cps } => (n * 1000.0 / cps) as u32,
        // The last word starts after every earlier one has begun.
        Reveal::Words { ms, stagger_ms } => {
            let words = text.split_whitespace().count().max(1) as u32;
            ms + stagger_ms * (words - 1)
        }
    }
}

fn hash(a: u64, b: u64) -> u64 {
    // SplitMix64 on the pair: deterministic, dependency-free.
    let mut z = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_add(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn frame(text: &str, reveal: Reveal, elapsed_ms: u32, motion: Motion) -> RevealFrame {
    let whole = RevealFrame {
        text: text.to_string(),
        caret: None,
        opacity: 1.0,
        done: true,
        words: Vec::new(),
    };
    if motion == Motion::Reduced || elapsed_ms >= duration_ms(reveal, text) {
        return whole;
    }
    let chars: Vec<char> = text.chars().collect();
    match reveal {
        Reveal::Arrive { ms } => {
            let t = elapsed_ms as f32 / ms as f32;
            // `--fx-ease: cubic-bezier(0.2, 0, 0, 1)` approximated by a
            // cubic ease-out; close enough for a 450 ms fade.
            let eased = 1.0 - (1.0 - t).powi(3);
            RevealFrame {
                text: text.to_string(),
                caret: None,
                opacity: eased,
                done: false,
                words: Vec::new(),
            }
        }
        Reveal::Decipher { cps, lead_ms, swap } => {
            let per_char = 1000.0 / cps;
            // The web swaps on every animation frame; a TUI draws at ~30 fps.
            let bucket = (elapsed_ms / 33) as u64;
            let out = chars
                .iter()
                .enumerate()
                .map(|(i, &ch)| {
                    if ch.is_whitespace()
                        || elapsed_ms as f32 > lead_ms as f32 + i as f32 * per_char
                    {
                        return ch;
                    }
                    // A glyph holds for a bucket with probability 1 - swap.
                    let mut b = bucket;
                    while b > 0 && (hash(i as u64, b) % 1000) as f32 >= swap * 1000.0 {
                        b -= 1;
                    }
                    GLYPHS[(hash(i as u64 ^ 0xD1, b) % GLYPHS.len() as u64) as usize]
                })
                .collect();
            RevealFrame {
                text: out,
                caret: None,
                opacity: 1.0,
                done: false,
                words: Vec::new(),
            }
        }
        Reveal::Type { cps } => {
            let typed = ((elapsed_ms as f32 * cps / 1000.0) as usize).min(chars.len());
            RevealFrame {
                text: chars[..typed].iter().collect(),
                caret: Some(typed),
                opacity: 1.0,
                done: false,
                words: Vec::new(),
            }
        }
        // `--kp-word-stagger`: word `i` starts `i * stagger_ms` in and
        // takes `ms`, on the same ease as Arrive. Nothing is typed and no
        // character changes; only the moment a word reaches its colour.
        Reveal::Words { ms, stagger_ms } => {
            let pieces: Vec<&str> = text.split_inclusive(' ').collect();
            let words = pieces
                .iter()
                .enumerate()
                .map(|(i, piece)| {
                    let start = (i as u32) * stagger_ms;
                    let t = elapsed_ms.saturating_sub(start) as f32 / ms as f32;
                    let eased = if t <= 0.0 {
                        0.0
                    } else {
                        1.0 - (1.0 - t.min(1.0)).powi(3)
                    };
                    ((*piece).to_string(), eased)
                })
                .collect();
            RevealFrame {
                text: text.to_string(),
                caret: None,
                opacity: 1.0,
                done: false,
                words,
            }
        }
    }
}
