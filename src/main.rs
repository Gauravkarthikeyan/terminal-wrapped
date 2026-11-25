mod detect;
mod parse;

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let explicit = !args.is_empty();
    let paths = if explicit {
        args
    } else {
        detect::candidates(|k| std::env::var(k).ok())
            .into_iter()
            .filter(|p| p.is_file())
            .collect()
    };

    if paths.is_empty() {
        eprintln!("no shell history found; pass a file: terminal-wrapped [HISTORY_FILE...]");
        std::process::exit(2);
    }

    let mut entries = Vec::new();
    for path in &paths {
        match std::fs::read(path) {
            Ok(raw) => {
                let found = detect::parse_file(path, &raw);
                println!("{:>7} commands  {}", found.len(), path.display());
                entries.extend(found);
            }
            Err(err) if explicit => {
                eprintln!("could not read {}: {err}", path.display());
                std::process::exit(1);
            }
            Err(_) => {}
        }
    }

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for e in &entries {
        if let Some(program) = e.command.split_whitespace().next() {
            *counts.entry(program).or_default() += 1;
        }
    }
    let mut top: Vec<_> = counts.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    println!();
    for (program, n) in top.iter().take(5) {
        println!("  {n:>6}  {program}");
    }
}
