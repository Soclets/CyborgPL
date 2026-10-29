#!/usr/bin/env python3
"""errors.cyr's lists of tests, one name per kind of error (owner, 2026-10-22/24): which name each reject test is.

  python3 kayo/tools/error_names.py move NAME TEST ...            put the tests under NAME, out of any other
  python3 kayo/tools/error_names.py new NAME 'means' TEST ...     a name not listed yet, with what it means
  python3 kayo/tools/error_names.py drop TEST ...                  tests that are gone
"""

import re
import sys
from pathlib import Path

PATH = Path(__file__).resolve().parents[2] / "errors.cyr"
ENTRY = re.compile(r'^(\s*errors::error\[name = "([^"]+)",.*?tests = \[)([^\]]*)(\]\],?)$')


def tests_of(body: str):
    return re.findall(r'"([^"]+)"', body)


def render(tests):
    return "".join(f'"{t}", ' for t in tests).rstrip(" ")


def main():
    cmd, args = sys.argv[1], sys.argv[2:]
    lines = PATH.read_text().split("\n")
    if cmd == "new":
        name, means, tests = args[0], args[1], args[2:]
        at = max(i for i, l in enumerate(lines) if ENTRY.match(l))
        lines.insert(at + 1, f'        errors::error[name = "{name}", means = "{means}", tests = [{render(tests)}]],')
        cmd, args = "move", [name] + tests
    if cmd in ("move", "drop"):
        name = args[0] if cmd == "move" else None
        tests = args[1:] if cmd == "move" else args
        found = False
        for i, l in enumerate(lines):
            m = ENTRY.match(l)
            if not m:
                continue
            have = [t for t in tests_of(m.group(3)) if t not in tests]
            if m.group(2) == name:
                have = sorted(set(have) | set(tests))
                found = True
            lines[i] = m.group(1) + render(have) + m.group(4)
        if cmd == "move" and not found:
            raise SystemExit(f"errors.cyr has no name `{name}`: add it with `new`")
    PATH.write_text("\n".join(lines))


if __name__ == "__main__":
    main()
