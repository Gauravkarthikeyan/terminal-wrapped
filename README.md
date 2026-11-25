# terminal-wrapped

**Spotify Wrapped, but for your terminal.**

Every December my music app tells me I listened to the same three songs 900
times. My terminal knows far more embarrassing things about me: how many times I
typed `gti`, how often I `git push --force` after midnight, and that `ls` is
somehow my second most-used "program". So I'm building the Wrapped it deserves.

```text
$ terminal-wrapped examples/night-owl.zsh_history
    154 commands  examples/night-owl.zsh_history

154 commands, 154 with timestamps
top programs:
      63  git
      26  ls
      20  cargo
      13  vim
      10  docker
git: status (28), push (12), commit (11)
busiest hour: 00:00
night owl: 71% of commands between 00:00 and 05:00
longest streak: 8 days
typos: sl → ls (4x), gti → git (1x)
sudo: 0  force pushes: 5  pipes per command: 0.03
```

That's real output from the sample history in [`examples/`](examples/), a
made-up week of a student who codes at night. (Run with `TZ=Asia/Kolkata`;
hours are bucketed in your local time zone.) The animated slide-by-slide
version is next.

## Try it

```sh
cargo run --release                       # finds your bash, zsh and fish history
cargo run --release -- ~/.zsh_history     # or point it at specific files
```

No dependencies, nothing leaves your machine. It reads the files and prints.

## Which shells

| Shell | Where it looks | What it understands |
|---|---|---|
| bash | `$HISTFILE`, `~/.bash_history` | `HISTTIMEFORMAT` timestamp lines (`#1764096000`) |
| zsh | `$HISTFILE`, `$ZDOTDIR/.zsh_history`, `~/.zsh_history`, `~/.histfile` | `EXTENDED_HISTORY`, multi-line commands, metafied bytes |
| fish | `$XDG_DATA_HOME/fish/fish_history` (or `~/.local/share/...`) | `- cmd:` / `when:` records, `\n` and `\\` escapes |

A file is matched by name first and by its first line otherwise, so a renamed
backup like `history-2025.txt` still works.

## How it works

```text
  ~/.bash_history ─┐
  ~/.zsh_history  ─┼─▶ detect ──▶ parse::{bash,zsh,fish} ──▶ Vec<Entry> ──▶ stats ──▶ output
  fish_history    ─┘   name,      one parser per format       command,       counts,
                       first line                              timestamp?     hours,
                                                               shell          streaks, typos
```

Every parser produces the same `Entry { command, timestamp, shell }`, so the
stats never care which shell a command came from.

### Things I didn't expect

- **zsh history isn't UTF-8.** zsh "metafies" its history file: any byte in
  `0x83..=0x9f` is written as `0x83` followed by the byte XOR `0x20`. Most UTF-8
  characters beyond ASCII contain such bytes, so reading the file naively turns
  `echo café 🚀` into mojibake. The parser unmetafies the raw bytes before
  decoding (`src/parse/zsh.rs`).
- **fish history looks like YAML but isn't.** fish writes it with its own
  serialiser that only escapes backslashes and newlines. A small line scanner is
  more faithful than a YAML crate that would choke on unquoted `: ` inside
  commands.
- **`git -C ../repo pull` broke my first git stats.** The subcommand isn't the
  first non-flag word: `-C`, `-c`, `--git-dir` and friends take a value.
- **Typo detection needs a frequency check.** `gti` is one transposition away
  from `git`, but so is `cp` from `cd` (one substitution). A program only counts
  as a typo if the command it's one edit away from is used at least 5× more.
- **The standard library can't tell you your time zone.** Rather than add a time
  crate for one number, it asks `date +%z`.

## Roadmap

- [x] bash, zsh and fish parsers, auto-detection
- [x] stats: programs, git subcommands, hour of day, streaks, night-owl score, typos, `--force`
- [ ] a terminal personality ("The 3 AM Git Pusher", "The Docker Whisperer", ...)
- [ ] animated slide-by-slide reveal, plus `--plain` and `--json`
- [ ] `--redact` to hide arguments, and a shareable SVG card

## License

MIT
