"""Marks the limb words in the CyborgPL code blocks of Markdown notes (owner, 2026-10-26: `%d64`, `inline.%MSL`,
`80 percent`), through `oracle --mark-limbs`: a block the oracle can lex is marked whole, else line by line, and a line
it cannot lex is left as it is. Blocks in another language are skipped. Review what it changes.

    python3 -I tools/mark_notes.py <oracle binary> <scratch folder> <note.md>...
"""
import os
import re
import subprocess
import sys

FOREIGN = re.compile(r'#include|kernel void|\bint main\b|\bvoid\b|fn main\(\)|::new\(|\bimpl\b|\bpub fn [a-z_]+\(|'
                     r'printf\("%|^\s*\[[a-z_ .]+\]\s*$|^my [a-z ]+:\s*$|\buint32_t\b', re.M)


def mark(oracle, scratch, text):
    path = os.path.join(scratch, "block.cy")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    r = subprocess.run([oracle, "--mark-limbs", path], capture_output=True, text=True)
    if r.returncode != 0:
        return None
    return open(path, encoding="utf-8").read()


def notes(oracle, scratch, path):
    lines = open(path, encoding="utf-8").read().split("\n")
    out, i, changed = [], 0, 0
    while i < len(lines):
        if lines[i].strip().startswith("```"):
            j = i + 1
            while j < len(lines) and not lines[j].strip().startswith("```"):
                j += 1
            block = lines[i + 1:j]
            out.append(lines[i])
            text = "\n".join(block)
            if FOREIGN.search(text) or lines[i].strip() not in ("```", "```cy", "```cyborgpl"):
                out.extend(block)
            else:
                whole = mark(oracle, scratch, text)
                new = whole.split("\n") if whole is not None else [mark(oracle, scratch, l) or l for l in block]
                changed += sum(a != b for a, b in zip(new, block))
                out.extend(new)
            if j < len(lines):
                out.append(lines[j])
            i = j + 1
            continue
        out.append(lines[i])
        i += 1
    if changed:
        open(path, "w", encoding="utf-8").write("\n".join(out))
    return changed


if __name__ == "__main__":
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    for p in sys.argv[3:]:
        n = notes(sys.argv[1], sys.argv[2], p)
        if n:
            print(n, p)
