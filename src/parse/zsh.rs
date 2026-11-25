//! zsh history parser.
//!
//! With `setopt EXTENDED_HISTORY` every entry looks like
//! `: <start-epoch>:<elapsed-seconds>;<command>`; without it the file is one
//! command per line. Multi-line commands are stored with each inner newline
//! escaped as a trailing backslash.
//!
//! zsh also "metafies" the file: any byte in 0x83..=0x9f (and NUL) is written as
//! the marker byte 0x83 followed by the original byte XOR 0x20. Reading the file
//! as UTF-8 without undoing that turns every accented character, emoji or CJK
//! string into mojibake, so the bytes are unmetafied before decoding.

use super::{Entry, Shell};

const META: u8 = 0x83;

pub fn parse(raw: &[u8]) -> Vec<Entry> {
    let text = String::from_utf8_lossy(&unmetafy(raw)).into_owned();
    let mut entries = Vec::new();
    let mut lines = text.lines();

    while let Some(first) = lines.next() {
        let mut line = first.to_string();
        // A trailing backslash means the command continues on the next line.
        while line.ends_with('\\') {
            match lines.next() {
                Some(next) => {
                    line.pop();
                    line.push('\n');
                    line.push_str(next);
                }
                None => break,
            }
        }

        let (timestamp, command) = match split_extended(&line) {
            Some((ts, cmd)) => (Some(ts), cmd),
            None => (None, line.as_str()),
        };
        if command.trim().is_empty() {
            continue;
        }
        entries.push(Entry {
            command: command.to_string(),
            timestamp,
            shell: Shell::Zsh,
        });
    }
    entries
}

/// `: 1700000000:12;git push` -> (1700000000, "git push")
fn split_extended(line: &str) -> Option<(i64, &str)> {
    let rest = line.strip_prefix(": ")?;
    let (meta, command) = rest.split_once(';')?;
    let (start, elapsed) = meta.split_once(':')?;
    if !elapsed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((start.parse().ok()?, command))
}

fn unmetafy(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut bytes = raw.iter();
    while let Some(&b) = bytes.next() {
        if b == META {
            if let Some(&next) = bytes.next() {
                out.push(next ^ 0x20);
            }
        } else {
            out.push(b);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_history() {
        let e = parse(b": 1700000000:0;git status\n: 1700000042:3;cargo test\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].command, "git status");
        assert_eq!(e[0].timestamp, Some(1_700_000_000));
        assert_eq!(e[1].timestamp, Some(1_700_000_042));
        assert_eq!(e[1].shell, Shell::Zsh);
    }

    #[test]
    fn plain_history_and_commands_containing_semicolons() {
        let e = parse(b"ls\necho a; echo b\n");
        assert_eq!(e[1].command, "echo a; echo b");
        assert_eq!(e[1].timestamp, None);
    }

    #[test]
    fn multi_line_commands_are_joined() {
        let e = parse(b": 1700000000:0;for f in *; do\\\n  echo $f\\\ndone\n: 1700000001:0;ls\n");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].command, "for f in *; do\n  echo $f\ndone");
        assert_eq!(e[1].command, "ls");
    }

    #[test]
    fn metafied_utf8_is_decoded() {
        // "é" is C3 A9; "🚀" is F0 9F 9A 80. 0x9F and 0x9A fall in the meta range.
        let rocket = [0xF0, META, 0x9F ^ 0x20, META, 0x9A ^ 0x20, 0x80];
        let mut raw = b": 1700000000:0;echo caf\xC3\xA9 ".to_vec();
        raw.extend_from_slice(&rocket);
        raw.push(b'\n');
        assert_eq!(parse(&raw)[0].command, "echo café 🚀");
    }
}
