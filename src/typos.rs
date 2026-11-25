//! Spotting fat-fingered commands: `gti`, `sl`, `claer`, `got push`.
//!
//! A program counts as a typo of another when it is one edit away (insert,
//! delete, substitute, or swap two neighbouring letters) and the other one is
//! used far more often. The frequency check is what stops `cp`/`cd` or
//! `vi`/`vim` from flagging each other.

/// How much more often the intended command has to appear than the typo.
const MIN_RATIO: usize = 5;

#[derive(Debug, PartialEq, Eq)]
pub struct Typo {
    pub typed: String,
    pub meant: String,
    pub count: usize,
}

/// `programs` must be sorted by count, most used first (as `stats` returns it).
pub fn find(programs: &[(String, usize)]) -> Vec<Typo> {
    let mut typos = Vec::new();
    for (typed, count) in programs {
        if typed.chars().count() < 2 {
            continue;
        }
        let meant = programs
            .iter()
            .take_while(|(_, n)| *n >= count * MIN_RATIO)
            .find(|(candidate, _)| is_one_edit(typed, candidate));
        if let Some((meant, _)) = meant {
            typos.push(Typo {
                typed: typed.clone(),
                meant: meant.clone(),
                count: *count,
            });
        }
    }
    typos.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.typed.cmp(&b.typed)));
    typos
}

/// Optimal string alignment distance == 1, without building the full matrix.
fn is_one_edit(a: &str, b: &str) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let (ra, rb) = (&a[prefix..], &b[prefix..]);
    match (ra.len(), rb.len()) {
        (0, 0) => false,
        (n, m) if n == m => {
            ra[1..] == rb[1..] || (n >= 2 && ra[0] == rb[1] && ra[1] == rb[0] && ra[2..] == rb[2..])
        }
        (n, m) if n == m + 1 => ra[1..] == *rb,
        (n, m) if n + 1 == m => *ra == rb[1..],
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
        pairs.iter().map(|(s, n)| (s.to_string(), *n)).collect()
    }

    #[test]
    fn one_edit_covers_all_four_kinds() {
        assert!(is_one_edit("gti", "git")); // swap
        assert!(is_one_edit("sl", "ls")); // swap
        assert!(is_one_edit("got", "git")); // substitute
        assert!(is_one_edit("gitt", "git")); // insert
        assert!(is_one_edit("clar", "clear")); // delete
        assert!(!is_one_edit("git", "git"));
        assert!(!is_one_edit("claer", "cargo"));
        assert!(!is_one_edit("tig", "git")); // two edits
    }

    #[test]
    fn rare_neighbours_of_frequent_commands_are_typos() {
        let programs = counts(&[
            ("git", 500),
            ("ls", 300),
            ("cd", 250),
            ("gti", 7),
            ("sl", 3),
            ("claer", 1),
        ]);
        let typos = find(&programs);
        assert_eq!(
            typos,
            [
                Typo {
                    typed: "gti".into(),
                    meant: "git".into(),
                    count: 7
                },
                Typo {
                    typed: "sl".into(),
                    meant: "ls".into(),
                    count: 3
                },
            ]
        );
    }

    #[test]
    fn similar_commands_used_often_are_not_typos() {
        // cd/cp and vi/vim are one edit apart but both genuinely used.
        let programs = counts(&[("cd", 100), ("cp", 40), ("vim", 60), ("vi", 30)]);
        assert!(find(&programs).is_empty());
    }
}
