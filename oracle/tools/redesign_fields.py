#!/usr/bin/env python3
"""After `oracle --redesign` (2026-10-27): a record's, union's or enumeration's fields written on one line, as a one-line
block's were, each get a line of their own. Only lines holding more than one field, inside those blocks, change.
`python3 oracle/tools/redesign_fields.py <file.cy | folder>…`"""
import os, re, sys

OPEN = re.compile(r'^\s*(?:public(?:\.kit)?\s+)?(?:%?layout\.%?\w+\s+)?(record|union\.tagged|union|enumeration)\s+\S+.*:\s*(#.*)?$')

def split_top(line):
    parts, cur, depth, text = [], "", 0, False
    for c in line:
        if c == '"': text = not text
        if not text:
            if c in "[(": depth += 1
            elif c in "])": depth -= 1
        cur += c
        if c == ';' and depth == 0 and not text:
            parts.append(cur.strip()); cur = ""
    return parts, cur.strip()

def fix(path):
    lines = open(path).read().split('\n')
    out, inside = [], False
    for l in lines:
        if inside and l.strip().startswith('end of '):
            inside = False
        elif inside and '#' not in l:
            parts, rest = split_top(l)
            if len(parts) > 1 and not rest:
                ind = l[:len(l) - len(l.lstrip())]
                out.extend(ind + p for p in parts)
                continue
        if OPEN.match(l):
            inside = True
        out.append(l)
    new = '\n'.join(out)
    if new != '\n'.join(lines):
        open(path, 'w').write(new)
        print("split", path)

for t in sys.argv[1:]:
    if os.path.isdir(t):
        for root, _, names in os.walk(t):
            for n in names:
                if n.endswith('.cy'): fix(os.path.join(root, n))
    else:
        fix(t)
