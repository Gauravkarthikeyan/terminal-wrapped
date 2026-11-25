//! fish history parser.
//!
//! fish stores history in a YAML-like file (`~/.local/share/fish/fish_history`):
//!
//! ```text
//! - cmd: git push --force
//!   when: 1764096000
//!   paths:
//!     - src/main.rs
//! ```
//!
//! It is not real YAML: fish writes it with its own serialiser, which escapes
//! only backslashes (`\\`) and newlines (`\n`) inside `cmd`. So a tiny line
//! scanner is both simpler and more faithful than pulling in a YAML crate.

use super::{Entry, Shell};

pub fn parse(input: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    for line in input.lines() {
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            entries.push(Entry {
                command: unescape(cmd),
                timestamp: None,
                shell: Shell::Fish,
            });
        } else if let Some(when) = line.strip_prefix("  when: ") {
            // `when` belongs to the most recent `- cmd:` record.
            if let Some(last) = entries.last_mut() {
                last.timestamp = when.trim().parse().ok();
            }
        }
        // `paths:` and its items are ignored.
    }

    entries.retain(|e| !e.command.trim().is_empty());
    entries
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            // Unknown escape: keep it verbatim rather than guessing.
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_with_timestamps_and_paths() {
        let input = "- cmd: cd ~/code\n  when: 1764096000\n  paths:\n    - ~/code\n\
                     - cmd: cargo test\n  when: 1764096042\n";
        let entries = parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "cd ~/code");
        assert_eq!(entries[0].timestamp, Some(1764096000));
        assert_eq!(entries[1].command, "cargo test");
        assert_eq!(entries[1].timestamp, Some(1764096042));
        assert!(entries.iter().all(|e| e.shell == Shell::Fish));
    }

    #[test]
    fn escaped_newlines_and_backslashes() {
        let input = "- cmd: for f in *\\n  echo $f\\nend\n  when: 1\n\
                     - cmd: printf 'a\\\\nb'\n  when: 2\n";
        let entries = parse(input);
        assert_eq!(entries[0].command, "for f in *\n  echo $f\nend");
        assert_eq!(entries[1].command, "printf 'a\\nb'");
    }

    #[test]
    fn missing_or_bad_timestamp() {
        let entries = parse("- cmd: ls\n- cmd: pwd\n  when: soon\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].timestamp, None);
        assert_eq!(entries[1].timestamp, None);
    }
}
