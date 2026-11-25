//! Bash history parser.
//!
//! Plain `~/.bash_history` is just one command per line. When `HISTTIMEFORMAT`
//! is set, bash also writes a comment line `#<unix-seconds>` before each command,
//! which is the only way to get timestamps out of bash. Both forms (and files
//! that mix them, which happens when the setting changes halfway through the
//! year) are handled here.

use super::{Entry, Shell};

/// Parse the contents of a bash history file.
pub fn parse(input: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut pending_ts: Option<i64> = None;

    for line in input.lines() {
        if let Some(ts) = timestamp_line(line) {
            pending_ts = Some(ts);
            continue;
        }
        let command = line.trim_end();
        if command.trim().is_empty() {
            continue;
        }
        entries.push(Entry {
            command: command.to_string(),
            timestamp: pending_ts.take(),
            shell: Shell::Bash,
        });
    }
    entries
}

/// `#1700000000` -> Some(1700000000). A real comment such as `# todo` is not a timestamp.
fn timestamp_line(line: &str) -> Option<i64> {
    let digits = line.strip_prefix('#')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_history_has_no_timestamps() {
        let entries = parse("ls -la\ngit status\n\ncd ..\n");
        let cmds: Vec<_> = entries.iter().map(|e| e.command.as_str()).collect();
        assert_eq!(cmds, ["ls -la", "git status", "cd .."]);
        assert!(entries.iter().all(|e| e.timestamp.is_none()));
    }

    #[test]
    fn timestamp_lines_attach_to_the_next_command() {
        let entries = parse("#1700000000\ngit push --force\n#1700000060\nsl\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "git push --force");
        assert_eq!(entries[0].timestamp, Some(1_700_000_000));
        assert_eq!(entries[1].timestamp, Some(1_700_000_060));
    }

    #[test]
    fn mixed_files_and_real_comments() {
        let entries = parse("echo old\n#1700000000\necho new\n# just a comment\n");
        assert_eq!(entries[0].timestamp, None);
        assert_eq!(entries[1].timestamp, Some(1_700_000_000));
        // a comment typed at the prompt is still a "command" bash saved
        assert_eq!(entries[2].command, "# just a comment");
        assert_eq!(entries[2].timestamp, None);
    }
}
