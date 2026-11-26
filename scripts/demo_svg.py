#!/usr/bin/env python3
"""Turn a captured terminal-wrapped run into an animated SVG for the README.

    script -qc "terminal-wrapped --fast examples/night-owl.zsh_history" /dev/null \
        | python3 scripts/demo_svg.py > docs/demo.svg

Frames are split on the clear-screen sequence the animation writes, ANSI colours
become SVG fills, and CSS keyframes play the frames back on a loop. No
dependencies beyond the standard library.
"""
import html
import re
import sys

FG = {"31": "#ff6b6b", "32": "#69db7c", "33": "#ffd43b", "34": "#74c0fc", "35": "#e599f7", "36": "#66d9e8"}
TEXT, DIM, BG = "#e6e6e6", "#7d8590", "#0f1117"
CHAR_W, LINE_H, PAD = 8.43, 17, 18
STEP, SLIDE_END = 0.8, 2.6  # seconds per reveal step / on a slide's last frame

SGR = re.compile(r"\x1b\[([0-9;]*)m")


def esc(text):
    """Escape for XML; spaces become non-breaking so no renderer collapses them."""
    return html.escape(text).replace(" ", "\u00a0")


def spans(line):
    """Split one line into (text, fill, bold) runs."""
    out, fill, bold, pos = [], TEXT, False, 0
    for m in SGR.finditer(line):
        if m.start() > pos:
            out.append((line[pos:m.start()], fill, bold))
        for code in (m.group(1) or "0").split(";"):
            if code == "0":
                fill, bold = TEXT, False
            elif code == "1":
                bold = True
            elif code == "2":
                fill = DIM
            elif code in FG:
                fill = FG[code]
        pos = m.end()
    if pos < len(line):
        out.append((line[pos:], fill, bold))
    return out


BLOCKS = "▁▂▃▄▅▆▇█"


def draw_line(runs, top):
    """SVG elements for one line. Block characters become rectangles, because
    their glyphs vary wildly between fonts (gaps, outlines, wrong heights)."""
    out, col = [], 0
    baseline = top + LINE_H - 5
    for text, fill, bold in runs:
        for seg in re.findall(f"[{BLOCKS}]+|[^{BLOCKS}]+", text):
            if seg[0] in BLOCKS:
                # One rect per run of equal blocks (no seams), 80 % of the line
                # tall so bars on neighbouring rows don't merge into a slab.
                for m in re.finditer(r"(.)\1*", seg):
                    hgt = 0.8 * LINE_H * (BLOCKS.index(m.group(1)) + 1) / 8
                    out.append(
                        f'<rect x="{PAD + (col + m.start()) * CHAR_W:.1f}" y="{top + 0.9 * LINE_H - hgt:.1f}" '
                        f'width="{len(m.group(0)) * CHAR_W:.1f}" height="{hgt:.1f}" fill="{fill}"/>'
                    )
            elif seg.strip():
                weight = ' font-weight="700"' if bold else ""
                out.append(
                    f'<text x="{PAD + col * CHAR_W:.1f}" y="{baseline}" fill="{fill}"{weight}>{esc(seg)}</text>'
                )
            col += len(seg)
    return out


def main():
    raw = sys.stdin.buffer.read().decode("utf-8", "replace").replace("\r", "")
    frames = [f.split("\n") for f in raw.split("\x1b[H\x1b[J")[1:]]
    footer = lambda f: re.search(r"(\d+) / \d+", SGR.sub("", "\n".join(f))).group(1)
    durations = [
        SLIDE_END if i + 1 == len(frames) or footer(frames[i + 1]) != footer(f) else STEP
        for i, f in enumerate(frames)
    ]
    total = sum(durations)
    cols = max(len(SGR.sub("", l)) for f in frames for l in f)
    rows = max(len(f) for f in frames)
    w, h = cols * CHAR_W + 2 * PAD, rows * LINE_H + 2 * PAD

    css, groups, t = [], [], 0.0
    for i, (frame, d) in enumerate(zip(frames, durations)):
        a, b = 100 * t / total, 100 * (t + d) / total
        css.append(
            f"@keyframes f{i}{{0%,{a:.3f}%{{opacity:0}}{a + 0.001:.3f}%,{b:.3f}%{{opacity:1}}"
            f"{b + 0.001:.3f}%,100%{{opacity:0}}}}#f{i}{{animation:f{i} {total:.1f}s infinite}}"
        )
        texts = []
        for r, line in enumerate(frame):
            texts.extend(draw_line(spans(line), PAD + r * LINE_H))
        groups.append(f'<g id="f{i}" opacity="0">{"".join(texts)}</g>')
        t += d

    print(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.0f} {h:.0f}" width="{w:.0f}" '
        f'height="{h:.0f}" role="img" aria-label="terminal-wrapped revealing its stats one slide at a time">'
        f"<style>text{{font-size:14px;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,'DejaVu Sans Mono',monospace;"
        f"white-space:pre}}{''.join(css)}</style>"
        f'<rect width="100%" height="100%" rx="10" fill="{BG}"/>{"".join(groups)}</svg>'
    )


if __name__ == "__main__":
    main()
