//! Everything the renderers show, computed once.

use std::path::PathBuf;

use crate::parse::Entry;
use crate::persona::{self, Verdict};
use crate::stats::{self, Stats};
use crate::typos::{self, Typo};

pub struct Report {
    /// Each history file read, with how many commands it held.
    pub sources: Vec<(PathBuf, usize)>,
    pub stats: Stats,
    pub typos: Vec<Typo>,
    pub verdict: Verdict,
}

impl Report {
    pub fn build(sources: Vec<(PathBuf, usize)>, entries: &[Entry], utc_offset: i64) -> Self {
        let stats = stats::compute(entries, utc_offset);
        let typos = typos::find(&stats.programs);
        let verdict = persona::decide(&stats, &typos);
        Report {
            sources,
            stats,
            typos,
            verdict,
        }
    }

    /// The local hour with the most commands, if any command had a timestamp.
    pub fn peak_hour(&self) -> Option<usize> {
        (self.stats.timestamped > 0).then(|| {
            (0..24)
                .max_by_key(|&h| (self.stats.hours[h], std::cmp::Reverse(h)))
                .unwrap_or(0)
        })
    }
}
