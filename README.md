# terminal-wrapped 🎁

**Spotify Wrapped, but for your terminal.**

Every December my music app tells me I listened to the same three songs 900
times. My terminal knows far more embarrassing things about me — how many times I
typed `gti`, how often I `git push --force` at 3 AM, and that `ls` is somehow my
most-used "program". So I'm building the Wrapped it deserves.

> 🚧 Work in progress — currently parses bash history and prints your top commands.

## Planned
- Parsers for **bash** (incl. `HISTTIMEFORMAT` timestamps), **zsh** extended history and **fish**
- Your year in numbers: top commands, git habits, hour-of-day heatmap, longest streak,
  night-owl score, `--force` counter, recurring typos
- A terminal **personality** ("The 3 AM Git Pusher", "The Docker Whisperer", ...)
- An animated, slide-by-slide reveal right in your terminal, plus `--json` and a shareable card
- 100% local. Your history never leaves your machine.

## Try it
```sh
cargo run --release -- ~/.bash_history
```

## License
MIT
