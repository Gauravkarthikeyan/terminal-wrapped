//! Parsers that turn raw shell history files into a common [`Entry`] stream.

pub mod bash;
pub mod zsh;

/// Which shell a history entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
}

/// One command the user ran, normalised across shells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The full command line, exactly as typed (multi-line commands joined with `\n`).
    pub command: String,
    /// Unix timestamp in seconds, when the shell recorded one.
    pub timestamp: Option<i64>,
    pub shell: Shell,
}
