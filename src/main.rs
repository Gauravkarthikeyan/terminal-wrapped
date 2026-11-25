mod parse;

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".bash_history")));

    let Some(path) = path else {
        eprintln!("usage: terminal-wrapped [HISTORY_FILE]");
        std::process::exit(2);
    };

    let raw = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("could not read {}: {err}", path.display());
            std::process::exit(1);
        }
    };

    let is_zsh = path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().contains("zsh"));
    let entries = if is_zsh {
        parse::zsh::parse(&raw)
    } else {
        parse::bash::parse(&String::from_utf8_lossy(&raw))
    };
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for e in &entries {
        if let Some(program) = e.command.split_whitespace().next() {
            *counts.entry(program).or_default() += 1;
        }
    }
    let mut top: Vec<_> = counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    println!("{} commands in {}", entries.len(), path.display());
    for (program, n) in top.iter().take(5) {
        println!("  {n:>6}  {program}");
    }
}
