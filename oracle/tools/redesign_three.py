#!/usr/bin/env python3
"""Rewrite CyborgPL from redesign two to redesign three (owner, 2026-10-28: "done, build it").

    python3 oracle/tools/redesign_three.py [--show] <file.cy | folder> ...

Only code changes: text in quotes, characters in backticks, comments and the code of an inline block are left as they
are. What it rewrites:
  closers         `end of fn add` / `!end of MAIN`        → `;fn add` / `;MAIN`
  `!` marks       `!MAIN:`, `!no_check`, `!case`, `h.!m`    → `MAIN:`, `no_check`, `case`, `h.*m`
  operators       `plus minus times pow eq not_eq lt gt at_most at_least` → `+ - x ^ == != < > <= >=`;
                  `neg 5` → `-5` (`and`, `or`, `concat` keep their words, owner 2026-10-28)
  number types    `int.sign.32`, `float.%deci.64`, …       → `i32`, `%d64`, …
  failures        `…) otherwise fail;`, `… otherwise 0;`   → `…)?;`, `… ?? 0;`
  changes         `change $x to $x + 1;`                    → `change $x += 1;` (only where it means the same)
  printing        `std::print.shortest(…)`                  → `std::print(…)` (a line still ends with `newln`)
A program it cannot finish (a `copy` on a plain union, say) is left for the oracle to name.
"""
import os
import re
import sys

OPS = [("not_eq", "!="), ("at_most", "<="), ("at_least", ">="),
       ("plus", "+"), ("minus", "-"), ("times", "x"), ("pow", "^"), ("eq", "=="), ("lt", "<"), ("gt", ">")]
INLINE_OPEN = re.compile(r'^\s*(?:!?no_check\s+)?inline\.%?(\w+)\b.*:\s*(#.*)?$')


def mask(src):
    """The source with every string, character, comment and inline block's code replaced by a placeholder, and the
    pieces taken out, in order."""
    out, pieces = [], []
    i, n = 0, len(src)
    lines_raw_until = None
    while i < n:
        c = src[i]
        if lines_raw_until is not None:
            # inside an inline block's code: up to its closing line
            j = src.find('\n', i)
            j = n if j < 0 else j
            line = src[i:j]
            if line.strip() == lines_raw_until or line.strip().startswith(('end of inline', '!end of', ';inline')):
                lines_raw_until = None
                continue
            pieces.append(src[i:j]); out.append(f'\x00{len(pieces) - 1}\x00')
            if j < n:
                out.append('\n')
            i = j + 1
            continue
        if c == '#':
            j = src.find('\n', i); j = n if j < 0 else j
            pieces.append(src[i:j]); out.append(f'\x00{len(pieces) - 1}\x00'); i = j
            continue
        if c == '$' and i + 1 < n and src[i + 1] == "'":
            j = i + 2
            while j < n and src[j] != "'":
                j += 2 if src[j] == '\\' else 1
            out.append(src[i:j + 1]); i = j + 1
            continue
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == '\\' and j + 1 < n and src[j + 1] in '"\\' else 1
            pieces.append(src[i:j + 1]); out.append(f'\x00{len(pieces) - 1}\x00'); i = j + 1
            continue
        if c == '`':
            j = src.find('`', i + 1); j = n - 1 if j < 0 else j
            pieces.append(src[i:j + 1]); out.append(f'\x00{len(pieces) - 1}\x00'); i = j + 1
            continue
        if c == '\n':
            # an inline block opens on the line just ended
            line_start = src.rfind('\n', 0, i) + 1
            m = INLINE_OPEN.match(src[line_start:i])
            if m:
                kind = re.search(r'inline\.%?\w+', src[line_start:i]).group(0)
                lines_raw_until = f'end of {kind}'
            out.append(c); i += 1
            continue
        out.append(c); i += 1
    return ''.join(out), pieces


def unmask(code, pieces):
    return re.sub(r'\x00(\d+)\x00', lambda m: pieces[int(m.group(1))], code)


def matching(code, open_at):
    """The index of the `)` closing the `(` at `open_at`."""
    d = 0
    for k in range(open_at, len(code)):
        if code[k] == '(':
            d += 1
        elif code[k] == ')':
            d -= 1
            if d == 0:
                return k
    return -1


def top_level_ops(e):
    """The operators written at depth 0 of an expression (redesign three's symbols)."""
    d, ops, k = 0, [], 0
    toks = re.findall(r'\x00\d+\x00|\(|\)|\[|\]|\+\+|==|!=|<=|>=|&&|\|\||[-+x^<>&|]|div\.\w+|mod\.\w+|\S', e)
    prev = None
    for t in toks:
        if t in '([':
            d += 1
        elif t in ')]':
            d -= 1
        elif d == 0 and t in ('+', '-', 'x', '^', '++', '==', '!=', '<', '>', '<=', '>=', '&', '|', '&&', '||') or (d == 0 and (t.startswith('div.') or t.startswith('mod.'))):
            if not (t == '-' and (prev is None or prev in ('+', '-', 'x', '^', '(', ','))):
                ops.append(t)
        prev = t
    return ops


def convert(src):
    code, pieces = mask(src)
    handlers = set(re.findall(r'coll\.handler\s+(\w+)', code))
    # closers and `!`
    code = re.sub(r'^(\s*)!?end of ', r'\1;', code, flags=re.M)
    code = re.sub(r'(?<![\w.])!(MAIN|no_check|NO_FUCKING_CHECKS|case)\b', r'\1', code)
    code = re.sub(r'\.!(?=\w)', '.*', code)
    code = re.sub(r'\botherwise !(?=\w)', 'otherwise *', code)
    # number types (the mark in front of the short name)
    for pat, rep in [(r'int\.sign\.%not_std\.(\d+)', r'%not_std.i\1'), (r'int\.unsign\.%not_std\.(\d+)', r'%not_std.u\1'),
                     (r'int\.sign\.%(\d+)', r'%i\1'), (r'int\.unsign\.%(\d+)', r'%u\1'), (r'float\.bin\.%(\d+)', r'%f\1'),
                     (r'float\.%deci\.(\d+)', r'%d\1'), (r'float\.%brain\.16', '%bf16'),
                     (r'int\.sign\.(\d+)', r'i\1'), (r'int\.unsign\.(\d+)', r'u\1'), (r'float\.bin\.(\d+)', r'f\1'),
                     (r'float\.deci\.(\d+)', r'%d\1'), (r'float\.brain\.16', '%bf16')]:
        code = re.sub(r'(?<![\w$])' + pat + r'(?!\w)', rep, code)
    # operators: a word between two values, never one in a path (`std::pow`) or narrowed (`times.wrap`)
    for w, s in OPS:
        code = re.sub(r'(?<![\w.:$%])' + re.escape(w) + r'(?![\w.])', s, code)
    code = re.sub(r'(?<![\w.:$])neg\s+(?=[\w$(\x00])', '-', code)
    # failures
    code = re.sub(r'\s+otherwise fail;', '?;', code)
    keep = r'(?:fail\b|ret\b|continue\b|break\b|:|\*|std::print' + ''.join(r'|' + re.escape(h) + r'\b' for h in handlers) + ')'
    code = re.sub(r'\botherwise (?!' + keep + ')', '?? ', code)
    # `change $x to $x + e;` → `change $x += e;`, only where the two mean the same
    def change(m):
        target, op, e = m.group(1), m.group(2), m.group(3)
        ops = top_level_ops(e)
        tighter = {'+': {'x', '^'}, '-': {'x', '^'}, 'x': {'^'}, '^': set()}
        if any(not (o in tighter[op] or o.startswith('div.') or o.startswith('mod.')) for o in ops):
            return m.group(0)
        if op in ('x', '^') and any(o.startswith('div.') or o.startswith('mod.') for o in ops):
            return m.group(0)
        return f'change {target} {op}= {e};'
    code = re.sub(r'change (\$[\w.$\[\]: ]+?) to \1 ([-+x^]) ([^;\n]+);', change, code)
    # printing: a bare `std::print` is `print.shortest`; a line still ends with `newln` (owner, 2026-10-28: `println` dropped)
    code = re.sub(r'std::print\.shortest\.auto_newline\((.*?)\)', lambda m: f'std::print({m.group(1)}, newln)', code)
    code = re.sub(r'std::print\.shortest\(', 'std::print(', code)
    # an inline block's closing line
    return unmask(code, pieces)


def main(argv):
    show = '--show' in argv
    paths = [a for a in argv if a != '--show']
    files = []
    for p in paths:
        if os.path.isdir(p):
            for root, _, names in os.walk(p):
                files += [os.path.join(root, f) for f in sorted(names) if f.endswith('.cy')]
        else:
            files.append(p)
    for f in files:
        src = open(f).read()
        new = convert(src)
        if show:
            sys.stdout.write(new)
        elif new != src:
            open(f, 'w').write(new)
            print('rewrote', f)


if __name__ == '__main__':
    main(sys.argv[1:])
