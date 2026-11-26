//! What each slide of the reveal says. Pure, so it's tested without a terminal.

use crate::report::Report;
use crate::spark;

#[derive(Debug, PartialEq, Eq)]
pub struct Slide {
    /// Small line above the headline ("Your #1 program").
    pub kicker: String,
    /// The big reveal ("git").
    pub headline: String,
    /// Supporting lines, revealed one at a time.
    pub lines: Vec<String>,
}

fn slide(kicker: &str, headline: impl Into<String>, lines: Vec<String>) -> Slide {
    Slide {
        kicker: kicker.into(),
        headline: headline.into(),
        lines,
    }
}

pub fn slides(r: &Report) -> Vec<Slide> {
    let s = &r.stats;
    let mut out = Vec::new();

    let files = match r.sources.len() {
        1 => "from 1 history file".to_string(),
        n => format!("from {n} history files"),
    };
    out.push(slide(
        "Your shell remembers",
        format!("{} commands", group(s.total)),
        vec![files],
    ));

    if let Some((top, n)) = s.programs.first() {
        let max = *n;
        let runners: Vec<String> = s
            .programs
            .iter()
            .take(5)
            .map(|(p, n)| format!("{:<10} {} {}", truncate(p, 10), bar(*n, max, 24), group(*n)))
            .collect();
        out.push(slide("Your #1 program was", top.clone(), runners));
    }

    if let Some((sub, n)) = s.git.first() {
        let pushes = s
            .git
            .iter()
            .find(|(c, _)| c == "push")
            .map_or(0, |(_, n)| *n);
        let mut lines = vec![format!("{} times. It's always `git {sub}`.", group(*n))];
        if pushes > 0 {
            let force = match s.force_pushes {
                0 => "none of them forced. Respect.".to_string(),
                1 => "1 of them with --force.".to_string(),
                f => format!("{f} of them with --force."),
            };
            lines.push(format!("You pushed {} times, {force}", group(pushes)));
        }
        out.push(slide(
            "Your favourite git command",
            format!("git {sub}"),
            lines,
        ));
    }

    if let Some(peak) = r.peak_hour() {
        let mut lines = vec![
            format!("00 {} 23", spark::sparkline(&s.hours)),
            format!(
                "{:.0}% of your commands ran between midnight and 5 AM.",
                s.night_owl * 100.0
            ),
        ];
        if s.longest_streak > 1 {
            lines.push(format!(
                "Longest streak: {} days in a row.",
                s.longest_streak
            ));
        }
        out.push(slide(
            "Your terminal was busiest at",
            format!("{peak:02}:00"),
            lines,
        ));
    }

    if let Some(t) = r.typos.first() {
        let times = if t.count == 1 {
            "once".to_string()
        } else {
            format!("{} times", t.count)
        };
        let mut lines = vec![format!("{times}. You meant `{}`.", t.meant)];
        let others: Vec<String> = r
            .typos
            .iter()
            .skip(1)
            .take(3)
            .map(|t| t.typed.clone())
            .collect();
        if !others.is_empty() {
            lines.push(format!("Also: {}.", others.join(", ")));
        }
        out.push(slide("Your signature typo", t.typed.clone(), lines));
    }

    let v = &r.verdict;
    let mut lines = vec![v.persona.tagline.to_string()];
    if !v.traits.is_empty() {
        let names: Vec<&str> = v.traits.iter().take(3).map(|p| p.name).collect();
        lines.push(format!("With traces of {}.", names.join(", ")));
    }
    out.push(slide("Your terminal personality", v.persona.name, lines));
    out
}

/// 12345 -> "12,345".
fn group(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn bar(n: usize, max: usize, width: usize) -> String {
    let filled = if max == 0 {
        0
    } else {
        (n * width).div_ceil(max)
    };
    "█".repeat(filled)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max - 1).collect();
        t.push('…');
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Entry, Shell};
    use std::path::PathBuf;

    fn entry(command: &str, timestamp: Option<i64>) -> Entry {
        Entry {
            command: command.into(),
            timestamp,
            shell: Shell::Zsh,
        }
    }

    #[test]
    fn number_grouping() {
        assert_eq!(group(0), "0");
        assert_eq!(group(999), "999");
        assert_eq!(group(1000), "1,000");
        assert_eq!(group(1234567), "1,234,567");
    }

    #[test]
    fn empty_history_still_gets_a_beginning_and_an_end() {
        let r = Report::build(vec![], &[], 0);
        let s = slides(&r);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].headline, "0 commands");
        assert_eq!(s.last().unwrap().headline, "The Generalist");
    }

    #[test]
    fn a_full_history_gets_every_slide_in_order() {
        let mut entries = vec![];
        for i in 0..40 {
            entries.push(entry("git status", Some(1_764_000_000 + i * 60)));
            entries.push(entry("ls", Some(1_764_000_030 + i * 60)));
        }
        entries.push(entry("git push --force", Some(1_764_100_000)));
        entries.push(entry("sl", None));
        let r = Report::build(vec![(PathBuf::from("h"), entries.len())], &entries, 0);
        let all = slides(&r);
        let kickers: Vec<&str> = all.iter().map(|s| s.kicker.as_str()).collect();
        assert_eq!(
            kickers,
            [
                "Your shell remembers",
                "Your #1 program was",
                "Your favourite git command",
                "Your terminal was busiest at",
                "Your signature typo",
                "Your terminal personality",
            ]
        );
        let git = &all[2];
        assert_eq!(git.headline, "git status");
        assert!(git.lines[1].contains("1 of them with --force"));
    }

    #[test]
    fn long_program_names_are_cut_to_fit_the_bar_chart() {
        assert_eq!(truncate("docker-compose", 10), "docker-co…");
        assert_eq!(truncate("ls", 10), "ls");
    }
}
