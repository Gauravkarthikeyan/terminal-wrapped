//! `--json`: the stats for scripts and dashboards.
//!
//! Hand-written instead of pulling in serde: the shape is small and fixed. Only
//! aggregates are written, never whole command lines, so the output is safe to
//! share in a way your raw history isn't.

use std::fmt::Write as _;
use std::io::{self, Write};

use crate::report::Report;

pub fn write(out: &mut impl Write, r: &Report) -> io::Result<()> {
    writeln!(out, "{}", to_json(r))
}

fn to_json(r: &Report) -> String {
    let s = &r.stats;
    let counts = |v: &[(String, usize)], key: &str| -> String {
        let items: Vec<String> = v
            .iter()
            .take(20)
            .map(|(name, n)| format!("{{{}:{},\"count\":{n}}}", string(key), string(name)))
            .collect();
        format!("[{}]", items.join(","))
    };
    let sources: Vec<String> = r
        .sources
        .iter()
        .map(|(p, n)| {
            format!(
                "{{\"path\":{},\"commands\":{n}}}",
                string(&p.to_string_lossy())
            )
        })
        .collect();
    let typos: Vec<String> = r
        .typos
        .iter()
        .map(|t| {
            format!(
                "{{\"typed\":{},\"meant\":{},\"count\":{}}}",
                string(&t.typed),
                string(&t.meant),
                t.count
            )
        })
        .collect();
    let traits: Vec<String> = r.verdict.traits.iter().map(|p| string(p.name)).collect();
    let hours: Vec<String> = s.hours.iter().map(usize::to_string).collect();

    let mut j = String::from("{");
    let _ = write!(
        j,
        "\"sources\":[{}],\"total\":{},\"timestamped\":{},\"programs\":{},\"git\":{},\
         \"hours\":[{}],\"peak_hour\":{},\"night_owl\":{},\"longest_streak\":{},\"sudo\":{},\
         \"force_pushes\":{},\"pipes_per_command\":{},\"typos\":[{}],\
         \"persona\":{{\"name\":{},\"tagline\":{},\"traits\":[{}]}}",
        sources.join(","),
        s.total,
        s.timestamped,
        counts(&s.programs, "program"),
        counts(&s.git, "subcommand"),
        hours.join(","),
        r.peak_hour().map_or("null".into(), |h| h.to_string()),
        number(s.night_owl),
        s.longest_streak,
        s.sudo,
        s.force_pushes,
        number(s.pipes_per_command),
        typos.join(","),
        string(r.verdict.persona.name),
        string(r.verdict.persona.tagline),
        traits.join(","),
    );
    j.push('}');
    j
}

/// A JSON string literal, escaping quotes, backslashes and control characters.
fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// JSON has no NaN or infinity; ratios here are always finite, but be safe.
fn number(x: f64) -> String {
    if x.is_finite() {
        format!("{:.4}", x)
    } else {
        "null".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_escaped() {
        assert_eq!(string(r#"say "hi"\n"#), r#""say \"hi\"\\n""#);
        assert_eq!(string("a\nb\tc"), r#""a\nb\tc""#);
        assert_eq!(string("\u{1b}[31m"), r#""\u001b[31m""#);
        assert_eq!(string("café 🚀"), "\"café 🚀\"");
    }

    #[test]
    fn numbers_never_produce_invalid_json() {
        assert_eq!(number(0.5), "0.5000");
        assert_eq!(number(f64::NAN), "null");
    }

    #[test]
    fn empty_report_is_well_formed() {
        let r = Report::build(vec![], &[], 0);
        let j = to_json(&r);
        assert!(j.starts_with("{\"sources\":[]"));
        assert!(j.contains("\"peak_hour\":null"));
        assert!(j.contains("\"persona\":{\"name\":\"The Generalist\""));
        assert_eq!(j.matches('{').count(), j.matches('}').count());
    }
}
