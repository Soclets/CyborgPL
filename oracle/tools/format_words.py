"""Rewrites `print.format` / `format` holes from Rust's shapes (`{:>8.2}`, `{2:x}`) to words (owner, 2026-10-25:
`{2, align: right, width: 8 cols, places: 2}`), the same translation as the oracle's message refusing the old one.
Only lines that call a format are touched; review what it changes.

    python3 -I tools/format_words.py <file>...
"""
import re
import sys

KINDS = {"x": "base: hex", "X": "base: hex.upper", "o": "base: oct", "b": "base: bin", "e": "form: science", "?": "form: stored"}
HOLE = re.compile(r"\{(\d*):([^}]*)\}")


def words(shape):
    m = re.fullmatch(r"(?:(.)?([<^>]))?(\+)?(0)?(\d+)?(?:\.(\d+))?([xXobe?])?", shape)
    if not m:
        return None
    fill, align, plus, zero, width, places, kind = m.groups()
    if align and zero:
        width, zero = (zero or "") + (width or ""), None
    w = []
    if kind: w.append(KINDS[kind])
    if places: w.append(f"places: {places}")
    if plus: w.append("sign")
    if zero: w.append("zeros")
    if align: w.append("align: " + {"<": "left", ">": "right", "^": "center"}[align])
    if width: w.append(f"width: {width} cols")
    if fill and fill != " ": w.append(f"fill: `{fill}`")
    return w


def hole(m):
    w = words(m.group(2))
    if w is None:
        return m.group(0)
    w = ([m.group(1)] if m.group(1) else []) + w
    return "{" + ", ".join(w) + "}"


for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8").read().split("\n")
    n = 0
    for i, line in enumerate(lines):
        if "format" not in line:
            continue
        new = HOLE.sub(hole, line)
        n += new != line
        lines[i] = new
    if n:
        open(path, "w", encoding="utf-8").write("\n".join(lines))
        print(n, path)
