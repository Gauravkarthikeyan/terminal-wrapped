//! Everything at once, no colour. Used with `--plain` and when stdout is a pipe.

use std::io::{self, Write};

use crate::report::Report;
use crate::spark;

pub fn write(out: &mut impl Write, r: &Report) -> io::Result<()> {
    let s = &r.stats;
    for (path, n) in &r.sources {
        writeln!(out, "{n:>7} commands  {}", path.display())?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "{} commands, {} with timestamps",
        s.total, s.timestamped
    )?;
    writeln!(out, "top programs:")?;
    for (program, n) in s.programs.iter().take(5) {
        writeln!(out, "  {n:>6}  {program}")?;
    }
    if !s.git.is_empty() {
        let git: Vec<String> = s
            .git
            .iter()
            .take(3)
            .map(|(c, n)| format!("{c} ({n})"))
            .collect();
        writeln!(out, "git: {}", git.join(", "))?;
    }
    if let Some(peak) = r.peak_hour() {
        writeln!(out, "busiest hour: {peak:02}:00")?;
        writeln!(out, "  00 {} 23", spark::sparkline(&s.hours))?;
        writeln!(
            out,
            "night owl: {:.0}% of commands between 00:00 and 05:00",
            s.night_owl * 100.0
        )?;
        writeln!(out, "longest streak: {} days", s.longest_streak)?;
    }
    if !r.typos.is_empty() {
        let list: Vec<String> = r
            .typos
            .iter()
            .take(3)
            .map(|t| format!("{} → {} ({}x)", t.typed, t.meant, t.count))
            .collect();
        writeln!(out, "typos: {}", list.join(", "))?;
    }
    let v = &r.verdict;
    writeln!(out)?;
    writeln!(out, "you are {}", v.persona.name)?;
    writeln!(out, "  {}", v.persona.tagline)?;
    if !v.traits.is_empty() {
        let names: Vec<&str> = v.traits.iter().map(|p| p.name).collect();
        writeln!(out, "  with traces of: {}", names.join(", "))?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "sudo: {}  force pushes: {}  pipes per command: {:.2}",
        s.sudo, s.force_pushes, s.pipes_per_command
    )
}
