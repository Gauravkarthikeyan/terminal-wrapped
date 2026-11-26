//! The reveal: one slide at a time, in a box, line by line.
//!
//! Plain ANSI escapes, no terminal library. It uses the 16 standard colours so
//! it follows the user's terminal theme instead of fighting it, and drops colour
//! entirely under `NO_COLOR`. The cursor stays visible: without a signal handler
//! a Ctrl-C mid-reveal would otherwise leave it hidden.

use std::io::{self, Write};
use std::time::Duration;

use super::slides::Slide;

/// Inner width of the card, in columns.
const INNER: usize = 58;
const ACCENTS: [&str; 6] = ["35", "36", "33", "32", "34", "31"];

pub struct Style {
    pub color: bool,
    pub fast: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn pause(&self, ms: u64) -> Duration {
        Duration::from_millis(if self.fast { ms / 4 } else { ms })
    }
}

pub fn play(
    out: &mut impl Write,
    slides: &[Slide],
    style: &Style,
    mut sleep: impl FnMut(Duration),
) -> io::Result<()> {
    for (i, slide) in slides.iter().enumerate() {
        let accent = ACCENTS[i % ACCENTS.len()];
        let footer = format!(" {} / {} ", i + 1, slides.len());
        // Reveal: kicker, then headline, then each supporting line.
        for shown in 0..=slide.lines.len() + 1 {
            let frame = frame(slide, shown, accent, &footer, style);
            write!(out, "\x1b[H\x1b[J{frame}")?;
            out.flush()?;
            sleep(style.pause(match shown {
                0 => 700,
                1 => 1000,
                _ => 550,
            }));
        }
        sleep(style.pause(1600));
    }
    writeln!(
        out,
        "{}",
        style.paint("2", "  terminal-wrapped --plain shows it all at once.")
    )?;
    out.flush()
}

/// One full card. `shown` = how much is revealed: 0 = kicker only, 1 = +headline,
/// 2.. = +that many supporting lines.
fn frame(slide: &Slide, shown: usize, accent: &str, footer: &str, style: &Style) -> String {
    let mut rows: Vec<(String, usize)> = vec![(String::new(), 0)];
    rows.push((
        style.paint("2", &slide.kicker),
        slide.kicker.chars().count(),
    ));
    rows.push((String::new(), 0));
    if shown >= 1 {
        let h = &slide.headline;
        rows.push((style.paint(&format!("1;{accent}"), h), h.chars().count()));
    } else {
        rows.push((String::new(), 0));
    }
    rows.push((String::new(), 0));
    for line in slide.lines.iter().take(shown.saturating_sub(1)) {
        for wrapped in wrap(line, INNER - 4) {
            let n = wrapped.chars().count();
            rows.push((wrapped, n));
        }
    }
    rows.push((String::new(), 0));

    let border = |s: &str| style.paint(accent, s);
    let mut f = String::from("\n");
    f.push_str(&format!(
        "  {}\n",
        border(&format!("╭{}╮", "─".repeat(INNER)))
    ));
    for (text, visible) in rows {
        let pad = INNER.saturating_sub(2 + visible);
        f.push_str(&format!(
            "  {}  {text}{}{}\n",
            border("│"),
            " ".repeat(pad),
            border("│")
        ));
    }
    let rule = INNER.saturating_sub(footer.chars().count() + 3);
    f.push_str(&format!(
        "  {}{}{}\n\n",
        border(&format!("╰{}", "─".repeat(rule))),
        style.paint("2", footer),
        border("───╯")
    ));
    f
}

/// Greedy word wrap; words longer than `width` are left whole.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split(' ') {
        let line = lines.last_mut().unwrap();
        let len = line.chars().count();
        if len > 0 && len + 1 + word.chars().count() > width {
            lines.push(word.to_string());
        } else {
            if len > 0 {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slide() -> Slide {
        Slide {
            kicker: "Your #1 program was".into(),
            headline: "git".into(),
            lines: vec!["63 times.".into(), "x ".repeat(40).trim_end().into()],
        }
    }

    fn render(color: bool) -> (String, Vec<Duration>) {
        let mut out = Vec::new();
        let mut sleeps = Vec::new();
        let style = Style { color, fast: false };
        play(&mut out, &[slide()], &style, |d| sleeps.push(d)).unwrap();
        (String::from_utf8(out).unwrap(), sleeps)
    }

    #[test]
    fn reveals_step_by_step_and_pauses_between() {
        let (out, sleeps) = render(false);
        // kicker, headline, two lines, then the end-of-slide pause.
        assert_eq!(sleeps.len(), 5);
        assert_eq!(out.matches("╭").count(), 4);
        assert!(out.contains("63 times."));
    }

    #[test]
    fn no_color_means_no_color_codes() {
        let (plain, _) = render(false);
        assert!(!plain.contains("\x1b[1;"));
        assert!(!plain.contains("\x1b[0m"));
        let (colored, _) = render(true);
        assert!(colored.contains("\x1b[1;35mgit\x1b[0m"));
    }

    #[test]
    fn every_row_has_the_same_width() {
        let style = Style {
            color: false,
            fast: false,
        };
        let f = frame(&slide(), 3, "35", " 1 / 1 ", &style);
        let widths: Vec<usize> = f
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().count())
            .collect();
        assert!(widths.iter().all(|&w| w == widths[0]), "{widths:?}");
    }

    #[test]
    fn wrap_breaks_on_spaces() {
        assert_eq!(wrap("aa bb cc", 5), ["aa bb", "cc"]);
        assert_eq!(wrap("short", 50), ["short"]);
    }

    #[test]
    fn fast_mode_quarters_the_pauses() {
        let style = Style {
            color: false,
            fast: true,
        };
        assert_eq!(style.pause(1600), Duration::from_millis(400));
    }
}
