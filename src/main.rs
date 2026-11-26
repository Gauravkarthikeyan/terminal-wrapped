mod args;
mod detect;
mod parse;
mod persona;
mod render;
mod report;
mod spark;
mod stats;
mod typos;
mod tz;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use args::{Command, Output};
use report::Report;

fn main() -> ExitCode {
    let (files, output, card) = match args::parse(std::env::args_os().skip(1)) {
        Ok(Command::Run {
            files,
            output,
            card,
        }) => (files, output, card),
        Ok(Command::Help) => {
            print!("{}", args::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("terminal-wrapped {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };

    let explicit = !files.is_empty();
    let paths: Vec<PathBuf> = if explicit {
        files
    } else {
        detect::candidates(|k| std::env::var(k).ok())
            .into_iter()
            .filter(|p| p.is_file())
            .collect()
    };
    if paths.is_empty() {
        eprintln!("no shell history found; pass a file: terminal-wrapped [HISTORY_FILE...]");
        return ExitCode::from(2);
    }

    let mut entries = Vec::new();
    let mut sources = Vec::new();
    for path in paths {
        match std::fs::read(&path) {
            Ok(raw) => {
                let found = detect::parse_file(&path, &raw);
                sources.push((path, found.len()));
                entries.extend(found);
            }
            Err(err) if explicit => {
                eprintln!("could not read {}: {err}", path.display());
                return ExitCode::FAILURE;
            }
            Err(_) => {}
        }
    }

    let report = Report::build(sources, &entries, tz::local_offset().unwrap_or(0));
    if let Some(path) = &card {
        if let Err(e) = std::fs::write(path, render::card::svg(&report)) {
            eprintln!("could not write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        eprintln!("wrote your card to {}", path.display());
    }
    let mut out = std::io::stdout().lock();
    let result = match output {
        Output::Json => render::json::write(&mut out, &report),
        Output::Animated { fast } if out.is_terminal() => {
            let style = render::animated::Style {
                // https://no-color.org: any non-empty value disables colour.
                color: std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty()),
                fast,
            };
            let slides = render::slides::slides(&report);
            render::animated::play(&mut out, &slides, &style, std::thread::sleep)
        }
        // Piped or redirected: animation would just be escape-code soup.
        Output::Plain | Output::Animated { .. } => render::plain::write(&mut out, &report),
    };
    // A closed pipe (`terminal-wrapped | head`) is not an error worth reporting.
    match result {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}
