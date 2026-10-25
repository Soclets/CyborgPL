"""Rewrites CyborgPL code snippets (in messages, comments and notes) to the 2026-10-22 syntax: calls in `( )`, plain
quoted names as `$name`. A heuristic, for text the oracle cannot parse; a whole program is better converted by
`oracle --to-paren-calls`, which reads it exactly. Review what it changes.

    python3 -I tools/snip_syntax.py notes <file.md>...   code blocks and code spans of Markdown notes, in place
"""
import re
import sys

# names whose `[ ]` describes or holds (a type, a building, a list of things), never a call
TYPE_HEADS = {
    'list', 'array', 'map', 'map.ordered', 'map.unordered', 'ptr', 'ptr.mut', 'C.str', 'str.utf8', 'str.ascii', 'channel',
    'simd', 'fail', 'optional', 'fn', 'range', 'labels', 'defaults', 'versions', 'size.of', 'align.of', 'incl.bytes',
    'incl.str.utf8', 'incl.C', 'incl.C.system', 'incl.C.local', 'inline.C', 'inline.asm', 'inline.LL', 'inline.MSL', 'asm',
    'for', 'in', 'bool', 'char', 'brain16', 'percent', 'spellings::old', 'spellings::call', 'spellings::table',
    'cyjet::limb', 'cyjet::target', 'cyjet::dep', 'cyjet::manifest', 'C.int', 'C.long', 'layout.C', 'union', 'struct',
    'enum', 'interface', 'returns', 'yield', 'or', 'with', 'str', 'lent', 'lent.mut', 'std::list', 'std::map',
    'std::map.ordered', 'std::map.unordered', 'ptr.cast', 'duration', 'std::duration', 'whole', 'any', 'errors::error',
    'errors::table', 'manual', 'manual.mut', 'error', 'check', 'note',
}
CALLS_ANYWAY = {'C.str.make', 'C.str.read'}
NUM = re.compile(r'^(i|u|f|d)\d+$|^(bin|hex|oct)$')
NAME = r"[A-Za-z_][A-Za-z0-9_]*"
CHAIN = re.compile(r"(?<![A-Za-z0-9_'$.:])((?:" + NAME + r"::)?" + NAME + r"(?:\." + NAME + r")*)\[")
VAR = re.compile(r"'([A-Za-z_][A-Za-z0-9_]*)'")
BACKTICKS = re.compile(r"`([^`\n]*)`")


def match_close(s, i):
    d = 0
    for j in range(i, len(s)):
        if s[j] == '[':
            d += 1
        elif s[j] == ']':
            d -= 1
            if d == 0:
                return j
    return None


def is_type(chain):
    if chain in CALLS_ANYWAY:
        return False
    if chain in TYPE_HEADS or chain.startswith(('spellings::', 'cyjet::', 'errors::')):
        return True
    last = chain.split('::')[-1]
    head = last.split('.')[0]
    return head[:1].isupper() or bool(NUM.match(head) or NUM.match(last))


def calls(s):
    out = []
    i = 0
    while True:
        m = CHAIN.search(s, i)
        if not m:
            out.append(s[i:])
            break
        chain, b = m.group(1), m.end() - 1
        e = match_close(s, b)
        if e is None:
            out.append(s[i:])
            break
        inner = s[b + 1:e]
        out.append(s[i:m.start()])
        before = s[max(0, m.start() - 5):m.start()]
        keeps = before.endswith(('fn ', 'op ', 'fail '))
        if chain.startswith('to.') and re.match(r'\s*(min|max|places|min_places|max_places)\s*:', inner):
            # `to.i32[min: 1, max: 99][$x]`: the limits describe, the bracket after them is the call
            out.append(chain + '[' + calls(inner) + ']')
            j = e + 1
            if j < len(s) and s[j] == '[':
                e2 = match_close(s, j)
                if e2 is not None:
                    out.append('(' + calls(s[j + 1:e2]) + ')')
                    i = e2 + 1
                    continue
            i = j
            continue
        if keeps or is_type(chain):
            out.append(chain + '[' + calls(inner) + ']')
            if chain == 'ptr.cast' and e + 1 < len(s) and s[e + 1] == '[':
                e2 = match_close(s, e + 1)
                if e2 is not None:
                    out.append('(' + calls(s[e + 2:e2]) + ')')
                    i = e2 + 1
                    continue
            i = e + 1
            continue
        out.append(chain + '(' + calls(inner) + ')')
        i = e + 1
    return ''.join(out)


# the later rulings, written as text: a channel made with `channel.empty` (2026-10-22), `code:` on exit (2026-10-23),
# `secs` and `mins` (2026-10-24), and `concat` for the old `++` between texts (2026-10-16)
LATER = [
    (re.compile(r"let (mut )?(\$\w+|'[^']+') := channel\[([^\]]+)\](\[max: \d+\])?;"), r"let \1\2: channel[\3]\4 = channel.empty;"),
    (re.compile(r"std::exit\((?!code:)"), "std::exit(code: "),
    (re.compile(r"(\d) seconds\b"), r"\1 secs"), (re.compile(r"(\d) second\b"), r"\1 sec"),
    (re.compile(r"(\d) minutes\b"), r"\1 mins"), (re.compile(r"(\d) minute\b"), r"\1 min"),
    (re.compile(r" \+\+ "), " concat "),
]


def snippet(s):
    s = VAR.sub(lambda m: '$' + m.group(1), calls(s))
    for pat, rep in LATER:
        s = pat.sub(rep, s)
    return s


def in_backticks(s):
    return BACKTICKS.sub(lambda m: '`' + snippet(m.group(1)) + '`', s)


# a block that is another language's: C, Metal, Rust, TOML-like manifests
FOREIGN = re.compile(r'#include|kernel void|\bint main\b|\bvoid\b|fn main\(\)|::new\(|\bimpl\b|\bpub fn [a-z_]+\(|'
                     r'printf\("%|^\s*\[[a-z_ .]+\]\s*$|^my [a-z ]+:\s*$|\buint32_t\b', re.M)


def notes(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    out, i, changed = [], 0, 0
    while i < len(lines):
        if lines[i].strip().startswith('```'):
            j = i + 1
            while j < len(lines) and not lines[j].strip().startswith('```'):
                j += 1
            block = lines[i + 1:j]
            out.append(lines[i])
            if FOREIGN.search('\n'.join(block)):
                out.extend(block)
            else:
                for line in block:
                    code, hsh, com = line.partition('#')
                    new = snippet(code) + hsh + (in_backticks(com) if hsh else '')
                    changed += new != line
                    out.append(new)
            if j < len(lines):
                out.append(lines[j])
            i = j + 1
            continue
        out.append(lines[i])
        i += 1
    if changed:
        open(path, 'w', encoding='utf-8').write('\n'.join(out))
    return changed


if __name__ == '__main__':
    if len(sys.argv) < 3 or sys.argv[1] != 'notes':
        sys.exit(__doc__)
    for p in sys.argv[2:]:
        n = notes(p)
        if n:
            print(n, p)
