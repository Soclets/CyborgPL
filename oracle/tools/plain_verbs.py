#!/usr/bin/env python3
"""Take the `copy` / `give` / `lend` off a plain value where the oracle refuses one, and add the `lend` a conversion
needs for an owned value it only reads (both redesign three, 2026-10-28) (redesign three, owner 2026-10-28: a
`union.tag` whose cases hold no data is plain), until a program runs past it.

    python3 oracle/tools/plain_verbs.py <file.cy | kit folder> ...

Each round runs the oracle, reads "line N: 'path' is a plain … and owns nothing", and removes the verb before `$path`
on that line. A refusal of another kind ends the rounds for that program.
"""
import os
import re
import subprocess
import sys

ORACLE = os.path.join(os.path.dirname(__file__), '..', 'target', 'debug', 'oracle')
MSG = re.compile(r"^(.*?): line (\d+)(?::\d+)?: '([^']+)'((?:\.\w+)*) is a plain \w+ and owns nothing")
# a conversion that only reads an owned value (owner, 2026-10-28: a conversion is a call like any other)
LEND = re.compile(r"^(.*?): line (\d+)(?::\d+)?: `(to\.[\w.%]+)` only looks at `\$([\w.]+)`: pass `lend")


def fix(target):
    for _ in range(200):
        r = subprocess.run([ORACLE, target], capture_output=True, text=True, timeout=600)
        m = next((MSG.match(l) for l in r.stderr.splitlines() if MSG.match(l)), None)
        lm = next((LEND.match(l) for l in r.stderr.splitlines() if LEND.match(l)), None)
        if lm and not m:
            path, line, conv, name = lm.group(1), int(lm.group(2)), lm.group(3), lm.group(4)
            if os.path.isdir(path):
                path = os.path.join(path, 'kit.cy')
            lines = open(path).read().split('\n')
            before = lines[line - 1]
            after = re.sub(re.escape(conv) + r'\((\s*)\$' + re.escape(name) + r'(?![\w.])', lambda x: conv + '(' + x.group(1) + 'lend $' + name, before, count=1)
            if after == before:
                print(f'{path}:{line}: could not find {conv}(${name}', file=sys.stderr)
                return
            lines[line - 1] = after
            open(path, 'w').write('\n'.join(lines))
            print(f'{path}:{line}: {conv} lends ${name}')
            continue
        if not m:
            return
        path, line, name, fields = m.group(1), int(m.group(2)), m.group(3), m.group(4)
        if os.path.isdir(path):
            path = os.path.join(path, 'kit.cy')
        lines = open(path).read().split('\n')
        before = lines[line - 1]
        pat = r'\b(?:copy|give|lend\.mut|lend)\s+(\$' + re.escape(name) + re.escape(fields) + r')(?![\w.])'
        after = re.sub(pat, r'\1', before, count=1)
        if after == before:
            print(f'{path}:{line}: could not find the verb before ${name}{fields}', file=sys.stderr)
            return
        lines[line - 1] = after
        open(path, 'w').write('\n'.join(lines))
        print(f'{path}:{line}: ${name}{fields} goes bare')


for a in sys.argv[1:]:
    fix(a)
