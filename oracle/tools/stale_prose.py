#!/usr/bin/env python3
"""List comments and string literals in programs that still use an old spelling.

The old spellings come from spellings.cyr. Plain English words in a comment
("a list", "plus one") are not code, so only code in a comment is checked:
what sits in backticks, and dotted or symbol forms anywhere. A line that
looks like history (a date, "was", "owner") is listed apart, since history
keeps its old spellings.

  python3 oracle/tools/stale_prose.py [files or folders...]   (default: examples bench)
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def old_spellings():
    text = open(os.path.join(ROOT, "spellings.cyr"), encoding="utf-8").read()
    olds = set()
    for m in re.finditer(r'written = "((?:[^"\\]|\\.)*)"', text):
        olds.add(m.group(1).replace('\\"', '"').replace("\\\\", "\\"))
    m = re.search(r"retired = \[([^\]]*)\]", text)
    if m:
        olds.update(re.findall(r'"([^"]+)"', m.group(1)))
    # too short or too common to tell from today's code
    for s in ["\\", "S", "!", "/", "%", "*", "--", "'x'", "{ }", "\\n", "LLVM", "Assembly", "lt", "gt", "eq", "neg"]:
        olds.discard(s)
    return olds

def pattern(old):
    p = re.escape(old)
    if re.match(r"\w", old): p = r"(?<![\w.$'])" + p
    if re.search(r"\w$", old): p = p + r"(?![\w])"
    return re.compile(p)

DOTTED = re.compile(r"\b(int\.(sign|unsign)\.\d+|float\.%?(bin|deci|brain)\.\d+|print\.useful|to\.useful|end of (fn|if|for|while|MAIN|match|case|rec|proc)\b)|\+\+|\bprintln\b")
HISTORY = re.compile(r"\b(was|were|old|retired|refused|before|replaced|replacing|owner|DECISIONS)\b|20\d\d-\d\d-\d\d")

def pieces(line):
    """The comment and the string literals of a line, as (kind, text)."""
    out, i, n = [], 0, len(line)
    while i < n:
        c = line[i]
        if c == '"':
            j = i + 1
            while j < n and line[j] != '"':
                j += 2 if line[j] == "\\" else 1
            out.append(("string", line[i + 1:j])); i = j + 1
        elif c == "'" :
            j = line.find("'", i + 1); i = (j + 1) if j > 0 else n
        elif c == "#":
            out.append(("comment", line[i + 1:])); break
        else:
            i += 1
    return out

def check(path, pats):
    found = []
    for no, line in enumerate(open(path, encoding="utf-8"), 1):
        for kind, text in pieces(line.rstrip("\n")):
            code = " ".join(re.findall(r"`([^`]*)`", text)) if kind == "comment" else text
            hits = {m.group(0) for m in DOTTED.finditer(text)}
            hits |= {old for old, p in pats if p.search(code)}
            if hits:
                found.append((no, kind, sorted(hits), line.strip(), bool(HISTORY.search(text))))
    return found

def main():
    args = sys.argv[1:] or [os.path.join(ROOT, "examples"), os.path.join(ROOT, "bench")]
    files = []
    for a in args:
        if os.path.isdir(a):
            for d, _, fs in os.walk(a):
                files += [os.path.join(d, f) for f in fs if f.endswith(".cy")]
        else:
            files.append(a)
    pats = [(o, pattern(o)) for o in sorted(old_spellings())]
    stale, history = 0, 0
    for f in sorted(files):
        for no, kind, hits, line, hist in check(f, pats):
            tag = "history?" if hist else "stale"
            stale += not hist; history += hist
            print(f"{os.path.relpath(f, ROOT)}:{no}: [{tag}] {kind}: {', '.join(hits)}\n    {line}")
    print(f"{stale} stale, {history} that look like history")
    return 1 if stale else 0

if __name__ == "__main__":
    sys.exit(main())
