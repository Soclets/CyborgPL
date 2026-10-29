#!/usr/bin/env python3
"""kayo/compiler/tests/reject_phrases.txt against the oracle's reject tests (`name [fnv:HASH] :: phrase`, HASH the
FNV-1a 64 of the `.err` the phrase was reviewed against).

  python3 kayo/tools/phrases.py report                 tests with no line, lines with no test, and lines whose .err moved
  python3 kayo/tools/phrases.py set NAME 'phrase'      write NAME's line with this phrase, reviewed against today's .err
  python3 kayo/tools/phrases.py refresh NAME ...       keep NAME's phrase, record today's .err as reviewed
  python3 kayo/tools/phrases.py drop NAME ...          take out the lines of tests that are gone
"""

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REJECT = REPO / "oracle/tests/reject"
PHRASES = REPO / "kayo/compiler/tests/reject_phrases.txt"


def fnv(path: Path) -> str:
    h = 0xcbf29ce484222325
    for b in path.read_bytes():
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def tests():
    out = {}
    for p in REJECT.iterdir():
        # A folder is a test when it holds CyborgPL (`incl_assets/` holds files a test reads).
        if p.suffix == ".cy" or (p.is_dir() and any(c.suffix in (".cy", ".cyr", ".nrv", ".nrp") for c in p.iterdir())):
            name = p.stem if p.suffix == ".cy" else p.name
            out[name] = REJECT / f"{name}.err"
    return out


def read():
    lines = PHRASES.read_text().split("\n")
    entries = {}
    for i, line in enumerate(lines):
        if " :: " in line and not line.startswith("#"):
            head, phrase = line.split(" :: ", 1)
            name = head.split()[0]
            mark = head[len(name):].strip()
            entries[name] = (i, mark, phrase)
    return lines, entries


def write(lines):
    PHRASES.write_text("\n".join(lines))


def main():
    cmd, args = sys.argv[1], sys.argv[2:]
    lines, entries = read()
    have = tests()
    if cmd == "report":
        for n in sorted(set(have) - set(entries)):
            print(f"no line: {n} :: {have[n].read_text().strip() if have[n].is_file() else '(no .err)'}")
        for n in sorted(set(entries) - set(have)):
            print(f"no test: {n}")
        for n in sorted(set(entries) & set(have)):
            _, mark, phrase = entries[n]
            if have[n].is_file() and mark != f"[fnv:{fnv(have[n])}]":
                print(f"moved: {n} :: {phrase}\n        .err: {have[n].read_text().strip()}")
    elif cmd == "set":
        name, phrase = args[0], args[1]
        line = f"{name} [fnv:{fnv(have[name])}] :: {phrase}"
        if name in entries:
            lines[entries[name][0]] = line
        else:
            while lines and lines[-1] == "":
                lines.pop()
            lines.append(line)
            lines.append("")
        write(lines)
    elif cmd == "refresh":
        for name in args:
            i, _, phrase = entries[name]
            lines[i] = f"{name} [fnv:{fnv(have[name])}] :: {phrase}"
        write(lines)
    elif cmd == "drop":
        for i in sorted((entries[n][0] for n in args), reverse=True):
            del lines[i]
        write(lines)


if __name__ == "__main__":
    main()
