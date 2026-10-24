"""Writes oracle/src/unicode_props.rs from Unicode's own files: `python3 -I tools/gen_unicode_props.py <ucd folder> src/unicode_props.rs`."""
import sys
ucd, out = sys.argv[1], sys.argv[2]
def derived(prop, file="DerivedCoreProperties.txt"):
    rs = []
    for line in open(f"{ucd}/{file}", encoding="utf-8"):
        line = line.split("#")[0].strip()
        if not line: continue
        parts = [x.strip() for x in line.split(";")]; cps, p = parts[0], parts[1]
        if p != prop: continue
        a, _, b = cps.partition("..")
        rs.append((int(a, 16), int(b or a, 16)))
    return merge(rs)
def merge(rs):
    rs.sort(); m = []
    for a, b in rs:
        if m and a <= m[-1][1] + 1: m[-1] = (m[-1][0], max(m[-1][1], b))
        else: m.append((a, b))
    return m
def ccc(test):
    rs = []; first = None
    for line in open(f"{ucd}/UnicodeData.txt", encoding="utf-8"):
        f = line.split(";"); cp = int(f[0], 16); c = int(f[3])
        if f[1].endswith("First>"): first = cp; continue
        if f[1].endswith("Last>"): lo = first
        else: lo = cp
        if test(c): rs.append((lo, cp))
    return merge(rs)
tables = {"CASED": derived("Cased"), "CASE_IGNORABLE": derived("Case_Ignorable"), "SOFT_DOTTED": derived("Soft_Dotted", "PropList.txt"),
          "CCC_ABOVE": ccc(lambda c: c == 230), "CCC_NONZERO": ccc(lambda c: c != 0)}
with open(out, "w", encoding="utf-8") as o:
    o.write("//! Unicode 17.0.0's character properties the oracle's case mapping needs, generated from Unicode's own files\n")
    o.write("//! (DerivedCoreProperties.txt, PropList.txt, UnicodeData.txt) by design claude's generator, std's being the same source (std claude,\n")
    o.write("//! 2026-10-24: the oracle's approximation was off past the blocks it knew). Unicode's licence: std/tools/UNICODE-LICENSE.\n\n")
    for name, rs in tables.items():
        o.write(f"pub const {name}: [(u32, u32); {len(rs)}] = [\n")
        for i in range(0, len(rs), 6):
            o.write("    " + " ".join(f"(0x{a:04X}, 0x{b:04X})," for a, b in rs[i:i+6]) + "\n")
        o.write("];\n\n")
    o.write("pub fn has(table: &[(u32, u32)], c: char) -> bool {\n    let c = c as u32;\n    table.binary_search_by(|&(a, b)| if c < a { std::cmp::Ordering::Greater } else if c > b { std::cmp::Ordering::Less } else { std::cmp::Ordering::Equal }).is_ok()\n}\n")
print({k: len(v) for k, v in tables.items()})
