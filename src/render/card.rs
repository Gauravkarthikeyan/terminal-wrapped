//! `--card FILE`: a shareable SVG summary, the "post this" part of Wrapped.
//!
//! Only aggregates go on the card (program names, counts, the persona), never a
//! full command line, so it's safe to share without reading it first.

use std::fmt::Write as _;

use crate::report::Report;

const W: usize = 600;
const H: usize = 800;
const BG: &str = "#0f1117";
const FG: &str = "#e6e6e6";
const DIM: &str = "#8b93a3";
const ACCENT: &str = "#ffd43b";
const BAR: &str = "#e599f7";
const FONT: &str = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'DejaVu Sans Mono', monospace";

pub fn svg(r: &Report) -> String {
    let s = &r.stats;
    let mut b = String::new();
    let _ = write!(
        b,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" font-family="{FONT}">"#
    );
    let _ = write!(b, r#"<rect width="{W}" height="{H}" rx="24" fill="{BG}"/>"#);
    text(&mut b, 48, 72, 16, DIM, false, "terminal-wrapped");
    text(&mut b, 48, 128, 15, DIM, false, "you are");
    text(&mut b, 48, 170, 34, ACCENT, true, r.verdict.persona.name);
    let mut y = 206;
    for line in wrap(r.verdict.persona.tagline, 52) {
        text(&mut b, 48, y, 15, FG, false, &line);
        y += 22;
    }

    // Top programs as bars.
    y += 34;
    text(&mut b, 48, y, 15, DIM, false, "top programs");
    y += 16;
    let max = s.programs.first().map_or(1, |(_, n)| *n).max(1);
    for (program, n) in s.programs.iter().take(5) {
        y += 34;
        // 14 characters fit before the bars start.
        let name: String = if program.chars().count() > 14 {
            program.chars().take(13).chain(['…']).collect()
        } else {
            program.clone()
        };
        text(&mut b, 48, y, 17, FG, false, &name);
        let width = 280 * n / max;
        let _ = write!(
            b,
            r#"<rect x="200" y="{}" width="{}" height="16" rx="3" fill="{BAR}"/>"#,
            y - 13,
            width.max(4)
        );
        text(&mut b, 210 + width, y, 15, DIM, false, &n.to_string());
    }

    // Three big numbers.
    y += 70;
    let mut stat = |x: usize, value: String, label: &str| {
        text(&mut b, x, y, 30, FG, true, &value);
        text(&mut b, x, y + 24, 13, DIM, false, label);
    };
    stat(48, s.total.to_string(), "commands");
    if r.peak_hour().is_some() {
        stat(
            230,
            format!("{:.0}%", s.night_owl * 100.0),
            "after midnight",
        );
        stat(410, s.longest_streak.to_string(), "day streak");
    } else {
        stat(230, s.force_pushes.to_string(), "force pushes");
        stat(410, s.sudo.to_string(), "sudos");
    }

    if let Some(t) = r.typos.first() {
        let line = format!("signature typo: {} → {} ({}x)", t.typed, t.meant, t.count);
        text(&mut b, 48, y + 82, 15, DIM, false, &line);
    }
    text(
        &mut b,
        48,
        H - 40,
        13,
        DIM,
        false,
        "github.com/Gauravkarthikeyan/terminal-wrapped",
    );
    b.push_str("</svg>\n");
    b
}

fn text(b: &mut String, x: usize, y: usize, size: u32, fill: &str, bold: bool, content: &str) {
    let weight = if bold { r#" font-weight="700""# } else { "" };
    let _ = write!(
        b,
        r#"<text x="{x}" y="{y}" font-size="{size}" fill="{fill}"{weight}>{}</text>"#,
        escape(content)
    );
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![String::new()];
    for word in text.split(' ') {
        let last = lines.last_mut().unwrap();
        if !last.is_empty() && last.chars().count() + 1 + word.chars().count() > width {
            lines.push(word.to_string());
        } else {
            if !last.is_empty() {
                last.push(' ');
            }
            last.push_str(word);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Entry, Shell};

    #[test]
    fn escapes_markup_in_program_names() {
        let entries = [Entry {
            command: "<script>&x".into(),
            timestamp: None,
            shell: Shell::Bash,
        }];
        let card = svg(&Report::build(vec![], &entries, 0));
        assert!(card.contains("&lt;script&gt;&amp;x"));
        assert!(!card.contains("<script>"));
    }

    #[test]
    fn empty_history_still_makes_a_card() {
        let card = svg(&Report::build(vec![], &[], 0));
        assert!(card.starts_with("<svg") && card.ends_with("</svg>\n"));
        assert!(card.contains("The Generalist"));
        assert!(card.contains("force pushes"));
    }
}
