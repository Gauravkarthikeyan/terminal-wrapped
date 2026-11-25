//! Turning a pile of history entries into numbers worth bragging about.

use std::collections::{BTreeSet, HashMap};

use crate::parse::Entry;

const DAY: i64 = 86_400;

#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub total: usize,
    /// Commands that had a timestamp; the time-based stats only use these.
    pub timestamped: usize,
    /// First word of each command, `sudo` stripped, most used first.
    pub programs: Vec<(String, usize)>,
    /// `git <subcommand>` counts, most used first.
    pub git: Vec<(String, usize)>,
    /// Commands per local hour of day.
    pub hours: [usize; 24],
    /// Share of timestamped commands run between 00:00 and 04:59 local time.
    pub night_owl: f64,
    /// Most consecutive local days with at least one command.
    pub longest_streak: usize,
    pub sudo: usize,
    pub force_pushes: usize,
    /// Mean number of `|` pipes per command.
    pub pipes_per_command: f64,
}

/// `utc_offset` is the local time zone's offset from UTC, in seconds.
pub fn compute(entries: &[Entry], utc_offset: i64) -> Stats {
    let mut programs: HashMap<&str, usize> = HashMap::new();
    let mut git: HashMap<&str, usize> = HashMap::new();
    let mut hours = [0usize; 24];
    let mut days = BTreeSet::new();
    let (mut sudo, mut force_pushes, mut pipes, mut timestamped) = (0, 0, 0, 0);

    for entry in entries {
        let mut words = entry.command.split_whitespace().peekable();
        if words.peek() == Some(&"sudo") {
            sudo += 1;
            words.next();
        }
        let Some(program) = words.next() else {
            continue;
        };
        *programs.entry(program).or_default() += 1;

        if program == "git" {
            let rest: Vec<&str> = words.collect();
            if let Some(sub) = git_subcommand(&rest) {
                *git.entry(sub).or_default() += 1;
                if sub == "push" && rest.iter().any(|w| is_force_flag(w)) {
                    force_pushes += 1;
                }
            }
        }

        pipes += entry.command.matches('|').count() - 2 * entry.command.matches("||").count();

        if let Some(ts) = entry.timestamp {
            timestamped += 1;
            let local = ts + utc_offset;
            hours[local.rem_euclid(DAY) as usize / 3600] += 1;
            days.insert(local.div_euclid(DAY));
        }
    }

    let night: usize = hours[..5].iter().sum();
    Stats {
        total: entries.len(),
        timestamped,
        programs: ranked(programs),
        git: ranked(git),
        hours,
        night_owl: ratio(night, timestamped),
        longest_streak: longest_run(&days),
        sudo,
        force_pushes,
        pipes_per_command: ratio(pipes, entries.len()),
    }
}

/// Skip git's global options (`-C <dir>`, `-c k=v`, `--no-pager`, ...).
fn git_subcommand<'a>(args: &[&'a str]) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(&arg) = iter.next() {
        match arg {
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace" => {
                iter.next();
            }
            a if a.starts_with('-') => {}
            a => return Some(a),
        }
    }
    None
}

fn is_force_flag(word: &str) -> bool {
    word == "-f" || word == "--force" || word.starts_with("--force-with-lease")
}

fn ranked(counts: HashMap<&str, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<_> = counts
        .into_iter()
        .map(|(k, n)| (k.to_string(), n))
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

fn longest_run(days: &BTreeSet<i64>) -> usize {
    let (mut best, mut run, mut prev) = (0, 0, None);
    for &day in days {
        run = if prev == Some(day - 1) { run + 1 } else { 1 };
        best = best.max(run);
        prev = Some(day);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Shell;

    fn at(command: &str, timestamp: Option<i64>) -> Entry {
        Entry {
            command: command.to_string(),
            timestamp,
            shell: Shell::Bash,
        }
    }

    // 2026-11-24 00:00:00 UTC
    const MIDNIGHT: i64 = 1_763_942_400;

    #[test]
    fn programs_skip_sudo_and_rank_by_count() {
        let s = compute(
            &[
                at("ls", None),
                at("sudo ls -la", None),
                at("cd /", None),
                at("   ", None),
            ],
            0,
        );
        assert_eq!(s.programs, [("ls".to_string(), 2), ("cd".to_string(), 1)]);
        assert_eq!(s.sudo, 1);
        assert_eq!(s.total, 4);
    }

    #[test]
    fn git_subcommands_and_force_pushes() {
        let s = compute(
            &[
                at("git -C repo status", None),
                at("git push --force", None),
                at("git push -f origin main", None),
                at("git push --force-with-lease", None),
                at("git push", None),
            ],
            0,
        );
        assert_eq!(s.git, [("push".to_string(), 4), ("status".to_string(), 1)]);
        assert_eq!(s.force_pushes, 3);
    }

    #[test]
    fn hours_and_night_owl_use_the_local_offset() {
        // 20:30 UTC is 02:00 in IST (+05:30).
        let ist = 5 * 3600 + 1800;
        let entries = [
            at("vim", Some(MIDNIGHT + 20 * 3600 + 1800)),
            at("ls", Some(MIDNIGHT + 6 * 3600)),
            at("pwd", None),
        ];
        let s = compute(&entries, ist);
        assert_eq!(s.hours[2], 1);
        assert_eq!(s.hours[11], 1);
        assert_eq!(s.timestamped, 2);
        assert!((s.night_owl - 0.5).abs() < 1e-9);
    }

    #[test]
    fn streak_counts_consecutive_days_only() {
        let days = [0, 1, 2, 5, 6];
        let entries: Vec<_> = days
            .iter()
            .map(|d| at("make", Some(MIDNIGHT + d * DAY + 60)))
            .collect();
        assert_eq!(compute(&entries, 0).longest_streak, 3);
    }

    #[test]
    fn pipes_ignore_logical_or() {
        let s = compute(
            &[
                at("ps aux | grep x | wc -l", None),
                at("make || true", None),
            ],
            0,
        );
        assert!((s.pipes_per_command - 1.0).abs() < 1e-9);
    }

    #[test]
    fn empty_history_is_all_zero() {
        assert_eq!(compute(&[], 0), Stats::default());
    }
}
