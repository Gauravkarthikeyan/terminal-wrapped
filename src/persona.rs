//! Your terminal personality, decided by a handful of rules.
//!
//! Each persona has a test that returns how strongly the history matches it:
//! the measured value divided by the threshold, so 1.0 means "just made it" and
//! 3.0 means "this is your whole personality". The strongest match wins and the
//! runners-up are reported as traits.

use crate::stats::Stats;
use crate::typos::Typo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Persona {
    pub name: &'static str,
    pub tagline: &'static str,
}

pub const GENERALIST: Persona = Persona {
    name: "The Generalist",
    tagline: "A bit of everything, nothing to confess. Suspicious.",
};

struct Rule {
    persona: Persona,
    strength: fn(&Signals) -> f64,
}

/// The numbers the rules look at, derived once from the stats.
struct Signals {
    night_owl: f64,
    git_share: f64,
    container_share: f64,
    editor_share: f64,
    cargo_share: f64,
    force_share_of_pushes: f64,
    force_pushes: usize,
    pipes_per_command: f64,
    typo_share: f64,
}

const RULES: &[Rule] = &[
    Rule {
        persona: Persona {
            name: "The 3 AM Git Pusher",
            tagline: "Your commits have timestamps your manager shouldn't see.",
        },
        strength: |s| (s.night_owl / 0.3).min(s.git_share / 0.25),
    },
    Rule {
        persona: Persona {
            name: "The Force Pusher",
            tagline: "History is written by whoever ran `git push --force` last.",
        },
        strength: |s| {
            if s.force_pushes < 3 {
                0.0
            } else {
                s.force_share_of_pushes / 0.2
            }
        },
    },
    Rule {
        persona: Persona {
            name: "The Docker Whisperer",
            tagline: "It works on your machine, and you shipped your machine.",
        },
        strength: |s| s.container_share / 0.15,
    },
    Rule {
        persona: Persona {
            name: "The Rustacean",
            tagline: "The borrow checker and you are on a first-name basis.",
        },
        strength: |s| s.cargo_share / 0.15,
    },
    Rule {
        persona: Persona {
            name: "The Editor Loyalist",
            tagline: "You open a terminal mainly to open your editor.",
        },
        strength: |s| s.editor_share / 0.15,
    },
    Rule {
        persona: Persona {
            name: "The Pipe Plumber",
            tagline: "Why write a script when you can write one very long line?",
        },
        strength: |s| s.pipes_per_command / 0.5,
    },
    Rule {
        persona: Persona {
            name: "The Fat Finger",
            tagline: "gti, sl, claer. Your keyboard has filed a complaint.",
        },
        strength: |s| s.typo_share / 0.02,
    },
    Rule {
        persona: Persona {
            name: "The Night Owl",
            tagline: "Your shell has seen more sunrises than you have.",
        },
        // The generic version of the first rule, so it needs a higher bar to
        // beat a more specific persona.
        strength: |s| s.night_owl / 0.5,
    },
];

#[derive(Debug, PartialEq)]
pub struct Verdict {
    pub persona: Persona,
    /// Other personas that also matched, strongest first.
    pub traits: Vec<Persona>,
}

pub fn decide(stats: &Stats, typos: &[Typo]) -> Verdict {
    let signals = signals(stats, typos);
    let mut matched: Vec<(f64, Persona)> = RULES
        .iter()
        .map(|r| ((r.strength)(&signals), r.persona))
        .filter(|(strength, _)| *strength >= 1.0)
        .collect();
    // Stable sort: on a tie, the rule listed first wins.
    matched.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut personas = matched.into_iter().map(|(_, p)| p);
    Verdict {
        persona: personas.next().unwrap_or(GENERALIST),
        traits: personas.collect(),
    }
}

fn signals(stats: &Stats, typos: &[Typo]) -> Signals {
    let share = |names: &[&str]| -> f64 {
        let n: usize = stats
            .programs
            .iter()
            .filter(|(p, _)| names.contains(&p.as_str()))
            .map(|(_, n)| n)
            .sum();
        ratio(n, stats.total)
    };
    let pushes = stats
        .git
        .iter()
        .find(|(sub, _)| sub == "push")
        .map_or(0, |(_, n)| *n);

    Signals {
        night_owl: stats.night_owl,
        git_share: share(&["git"]),
        container_share: share(&["docker", "podman", "kubectl", "k9s", "docker-compose"]),
        editor_share: share(&["vim", "nvim", "vi", "emacs", "nano", "hx", "code"]),
        cargo_share: share(&["cargo", "rustc", "rustup"]),
        force_share_of_pushes: ratio(stats.force_pushes, pushes),
        force_pushes: stats.force_pushes,
        pipes_per_command: stats.pipes_per_command,
        typo_share: ratio(typos.iter().map(|t| t.count).sum(), stats.total),
    }
}

fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(programs: &[(&str, usize)]) -> Stats {
        let programs: Vec<(String, usize)> =
            programs.iter().map(|(p, n)| (p.to_string(), *n)).collect();
        Stats {
            total: programs.iter().map(|(_, n)| n).sum(),
            programs,
            ..Stats::default()
        }
    }

    #[test]
    fn empty_history_is_a_generalist() {
        let v = decide(&Stats::default(), &[]);
        assert_eq!(v.persona, GENERALIST);
        assert!(v.traits.is_empty());
    }

    #[test]
    fn heavy_docker_use_wins() {
        let v = decide(&stats(&[("docker", 40), ("ls", 60)]), &[]);
        assert_eq!(v.persona.name, "The Docker Whisperer");
    }

    #[test]
    fn night_git_beats_plain_night_owl() {
        let mut s = stats(&[("git", 50), ("ls", 50)]);
        s.night_owl = 0.6;
        let v = decide(&s, &[]);
        assert_eq!(v.persona.name, "The 3 AM Git Pusher");
        assert!(v.traits.iter().any(|p| p.name == "The Night Owl"));
    }

    #[test]
    fn a_couple_of_force_pushes_are_forgiven() {
        let mut s = stats(&[("git", 10)]);
        s.git = vec![("push".into(), 4)];
        s.force_pushes = 2;
        assert!(decide(&s, &[]).persona.name != "The Force Pusher");
        s.force_pushes = 3;
        assert_eq!(decide(&s, &[]).persona.name, "The Force Pusher");
    }

    #[test]
    fn typos_make_a_fat_finger() {
        let s = stats(&[("git", 90), ("gti", 10)]);
        let typos = [Typo {
            typed: "gti".into(),
            meant: "git".into(),
            count: 10,
        }];
        assert_eq!(decide(&s, &typos).persona.name, "The Fat Finger");
    }
}
