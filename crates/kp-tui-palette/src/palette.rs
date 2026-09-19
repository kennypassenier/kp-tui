//! The vendored palette, included verbatim.
//!
//! `vendor/kp-tui-palette.rs` is the release asset byte for byte — not a
//! copy that was formatted, re-indented or touched on the way in. It is
//! pulled in with `include!` rather than declared as a module, because
//! `cargo fmt` walks modules and would realign the generator's columns;
//! an included file it leaves alone, and the bytes stay checkable against
//! the release that produced them.
//!
//! `vendor/PIN` names that release and carries the checksum the gates
//! verify. Upgrading is two commands, both in `../../../README.md`.

include!("../vendor/kp-tui-palette.rs");
