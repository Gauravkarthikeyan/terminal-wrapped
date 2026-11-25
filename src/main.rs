mod detect;
mod parse;
mod stats;

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

    let s = stats::compute(&entries, 0);
    println!();
    println!("{} commands, {} with timestamps", s.total, s.timestamped);
    println!("top programs:");
    for (program, n) in s.programs.iter().take(5) {
        println!("  {n:>6}  {program}");
    }
    if !s.git.is_empty() {
        let git: Vec<String> = s
            .git
            .iter()
            .take(3)
            .map(|(c, n)| format!("{c} ({n})"))
            .collect();
        println!("git: {}", git.join(", "));
    }
    if s.timestamped > 0 {
        let peak = (0..24)
            .max_by_key(|&h| (s.hours[h], std::cmp::Reverse(h)))
            .unwrap_or(0);
        println!("busiest hour: {peak:02}:00 (UTC)");
        println!(
            "night owl: {:.0}% of commands between 00:00 and 05:00",
            s.night_owl * 100.0
        );
        println!("longest streak: {} days", s.longest_streak);
    }
    println!(
        "sudo: {}  force pushes: {}  pipes per command: {:.2}",
        s.sudo, s.force_pushes, s.pipes_per_command
    );
}
