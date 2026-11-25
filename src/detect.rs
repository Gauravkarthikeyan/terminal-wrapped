//! Finding history files and working out which shell wrote them.

use std::path::{Path, PathBuf};

use crate::parse::{self, Entry, Shell};

/// Guess the shell from the file name, then from the content.
///
/// The name wins because plain bash history and zsh history without
/// `EXTENDED_HISTORY` are byte-for-byte the same format.
pub fn shell_for(path: &Path, raw: &[u8]) -> Shell {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.contains("zsh") || name == ".histfile" {
        return Shell::Zsh;
    }
    if name.contains("fish") {
        return Shell::Fish;
    }
    if name.contains("bash") {
        return Shell::Bash;
    }
    sniff(raw)
}

fn sniff(raw: &[u8]) -> Shell {
    let text = String::from_utf8_lossy(&raw[..raw.len().min(4096)]);
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    if first.starts_with("- cmd: ") {
        Shell::Fish
    } else if is_zsh_extended(first) {
        Shell::Zsh
    } else {
        Shell::Bash
    }
}

fn is_zsh_extended(line: &str) -> bool {
    let Some(rest) = line.strip_prefix(": ") else {
        return false;
    };
    let Some((meta, _cmd)) = rest.split_once(';') else {
        return false;
    };
    let Some((start, elapsed)) = meta.split_once(':') else {
        return false;
    };
    [start, elapsed]
        .iter()
        .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

pub fn parse_file(path: &Path, raw: &[u8]) -> Vec<Entry> {
    match shell_for(path, raw) {
        Shell::Zsh => parse::zsh::parse(raw),
        Shell::Fish => parse::fish::parse(&String::from_utf8_lossy(raw)),
        Shell::Bash => parse::bash::parse(&String::from_utf8_lossy(raw)),
    }
}

/// Every place a history file usually lives, most specific first.
///
/// `env` looks up environment variables; it is a parameter so tests don't have
/// to mutate the real process environment. Duplicates are removed, but the
/// files are not checked for existence here.
pub fn candidates(env: impl Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let home = env("HOME").map(PathBuf::from);
    let mut out: Vec<PathBuf> = Vec::new();

    if let Some(histfile) = env("HISTFILE") {
        out.push(PathBuf::from(histfile));
    }
    if let Some(home) = &home {
        out.push(home.join(".bash_history"));
    }
    if let Some(zdotdir) = env("ZDOTDIR") {
        out.push(PathBuf::from(zdotdir).join(".zsh_history"));
    }
    if let Some(home) = &home {
        out.push(home.join(".zsh_history"));
        out.push(home.join(".histfile"));
    }
    let data_home = env("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".local/share")));
    if let Some(data) = data_home {
        out.push(data.join("fish/fish_history"));
    }

    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.clone()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            vars.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        }
    }

    #[test]
    fn file_name_decides_first() {
        assert_eq!(shell_for(Path::new("/h/.zsh_history"), b"ls\n"), Shell::Zsh);
        assert_eq!(
            shell_for(Path::new("/h/fish_history"), b"ls\n"),
            Shell::Fish
        );
        assert_eq!(
            shell_for(Path::new("/h/.bash_history"), b"ls\n"),
            Shell::Bash
        );
    }

    #[test]
    fn unknown_names_are_sniffed() {
        let p = Path::new("/tmp/backup.txt");
        assert_eq!(shell_for(p, b": 1764096000:0;ls\n"), Shell::Zsh);
        assert_eq!(shell_for(p, b"\n- cmd: ls\n  when: 1\n"), Shell::Fish);
        assert_eq!(shell_for(p, b"#1764096000\nls\n"), Shell::Bash);
        // Looks a bit like zsh, but the timestamp isn't numeric.
        assert_eq!(shell_for(p, b": a:b;ls\n"), Shell::Bash);
    }

    #[test]
    fn candidates_respect_overrides_and_dedupe() {
        let vars = [
            ("HOME", "/home/g"),
            ("HISTFILE", "/home/g/.zsh_history"),
            ("XDG_DATA_HOME", "/data"),
        ];
        let got = candidates(env(&vars));
        assert_eq!(
            got,
            [
                "/home/g/.zsh_history",
                "/home/g/.bash_history",
                "/home/g/.histfile",
                "/data/fish/fish_history",
            ]
            .map(PathBuf::from)
        );
    }

    #[test]
    fn candidates_without_xdg_fall_back_to_local_share() {
        let got = candidates(env(&[("HOME", "/h")]));
        assert!(got.contains(&PathBuf::from("/h/.local/share/fish/fish_history")));
    }
}
