//! Command-line arguments. Small enough that a parser crate isn't worth it.

use std::ffi::OsString;
use std::path::PathBuf;

pub const HELP: &str = "\
terminal-wrapped: Spotify Wrapped, but for your shell history

usage: terminal-wrapped [OPTIONS] [HISTORY_FILE...]

With no files, it looks for bash, zsh and fish history in the usual places.

options:
  --plain        print everything at once, no animation or colour
  --json         print the stats as JSON
  --fast         animate, but without the dramatic pauses
  -h, --help     show this help
  -V, --version  show the version

Nothing leaves your machine. NO_COLOR is respected.
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    /// Slide-by-slide reveal; falls back to `Plain` when stdout isn't a terminal.
    Animated {
        fast: bool,
    },
    Plain,
    Json,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run { files: Vec<PathBuf>, output: Output },
    Help,
    Version,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut files = Vec::new();
    let (mut plain, mut json, mut fast) = (false, false, false);
    let mut only_files = false;

    for arg in args {
        if only_files {
            files.push(PathBuf::from(arg));
            continue;
        }
        match arg.to_str() {
            Some("-h" | "--help") => return Ok(Command::Help),
            Some("-V" | "--version") => return Ok(Command::Version),
            Some("--plain") => plain = true,
            Some("--json") => json = true,
            Some("--fast") => fast = true,
            Some("--") => only_files = true,
            Some(flag) if flag.starts_with('-') && flag.len() > 1 => {
                return Err(format!("unknown option {flag} (try --help)"));
            }
            _ => files.push(PathBuf::from(arg)),
        }
    }

    let output = match (json, plain) {
        (true, true) => return Err("--json and --plain don't mix".into()),
        (true, false) => Output::Json,
        (false, true) => Output::Plain,
        (false, false) => Output::Animated { fast },
    };
    Ok(Command::Run { files, output })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn defaults_to_animated_autodetect() {
        assert_eq!(
            run(&[]),
            Ok(Command::Run {
                files: vec![],
                output: Output::Animated { fast: false }
            })
        );
    }

    #[test]
    fn flags_and_files_mix_in_any_order() {
        assert_eq!(
            run(&["a.txt", "--json", "b.txt"]),
            Ok(Command::Run {
                files: vec!["a.txt".into(), "b.txt".into()],
                output: Output::Json
            })
        );
        assert_eq!(
            run(&["--fast"]),
            Ok(Command::Run {
                files: vec![],
                output: Output::Animated { fast: true }
            })
        );
    }

    #[test]
    fn double_dash_allows_files_that_look_like_flags() {
        assert_eq!(
            run(&["--plain", "--", "--weird-name"]),
            Ok(Command::Run {
                files: vec!["--weird-name".into()],
                output: Output::Plain
            })
        );
    }

    #[test]
    fn help_version_and_errors() {
        assert_eq!(run(&["x", "--help"]), Ok(Command::Help));
        assert_eq!(run(&["-V"]), Ok(Command::Version));
        assert!(run(&["--nope"]).unwrap_err().contains("--nope"));
        assert!(run(&["--json", "--plain"]).is_err());
        // A lone "-" is a file name, not a flag.
        assert!(matches!(run(&["-"]), Ok(Command::Run { .. })));
    }
}
