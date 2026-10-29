#!/usr/bin/env python3
"""Take a verb off a plain value, and add the `lend` a conversion needs, where kayo refuses one (redesign three,
2026-10-28: a `union.tag` whose cases hold no data is plain; a conversion lends what it only reads). The redesign
converters' third step, `oracle/tools/plain_verbs.py`, does this by running each program in the oracle; this one
reads kayo's refusals, so nothing runs.

    python3 kayo/tools/verb_fix.py <file.cy | kit folder> ...

Each round builds the program with kayo (KAYO_STD as set), reads a `nothing_to_give` or a conversion's
`owned_value_bare`, and edits that line; any other refusal, or none, ends the rounds for that program.
"""

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
KAYO = REPO / "kayo/target/release/kayo"
AT = re.compile(r"error\[(nothing_to_give|owned_value_bare)\] at (.*):(\d+):(\d+)")


def fix(target: str) -> int:
    edits = 0
    out = Path(tempfile.mkdtemp()) / "built"
    for _ in range(500):
        r = subprocess.run([str(KAYO), "build", target, "-o", str(out)], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        m = AT.search(r.stderr)
        if not m:
            break
        kind, path, line, col = m.group(1), Path(m.group(2)), int(m.group(3)), int(m.group(4))
        if not path.is_absolute():
            path = Path.cwd() / path
        lines = path.read_text().split("\n")
        text = lines[line - 1]
        at = col - 1
        if kind == "owned_value_bare":
            if "only looks at" not in r.stderr and "(lend $" not in r.stderr:
                break
            lines[line - 1] = text[:at] + "lend " + text[at:]
        else:
            verb = re.match(r"(copy|give|lend\.mut|lend) ", text[at:])
            if not verb:
                break
            lines[line - 1] = text[:at] + text[at + len(verb.group(0)):]
        path.write_text("\n".join(lines))
        edits += 1
    return edits


def main():
    for target in sys.argv[1:]:
        n = fix(target)
        if n:
            print(f"{target}: {n} edit(s)")


if __name__ == "__main__":
    main()
