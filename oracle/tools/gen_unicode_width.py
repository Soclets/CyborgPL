"""Writes oracle/src/unicode_width.rs, the screen columns a character takes (owner, 2026-10-25: `cols`), from Unicode's
own files: `python3 -I tools/gen_unicode_width.py <ucd folder> src/unicode_width.rs`. The folder holds
EastAsianWidth.txt, emoji-data.txt and UnicodeData.txt (Unicode 17.0.0, kept in .design-checks/ucd-17.0.0/)."""
import sys
ucd, out = sys.argv[1], sys.argv[2]
def merge(rs):
    rs.sort(); m = []
    for a, b in rs:
        if m and a <= m[-1][1] + 1: m[-1] = (m[-1][0], max(m[-1][1], b))
        else: m.append((a, b))
    return m
def prop_file(file, want):
    rs = []
    for line in open(f"{ucd}/{file}", encoding="utf-8"):
        line = line.split("#")[0].strip()
        if not line: continue
        cps, p = [x.strip() for x in line.split(";")][:2]
        if p not in want: continue
        a, _, b = cps.partition("..")
        rs.append((int(a, 16), int(b or a, 16)))
    return merge(rs)
def categories(want):
    rs = []; first = None
    for line in open(f"{ucd}/UnicodeData.txt", encoding="utf-8"):
        f = line.split(";"); cp = int(f[0], 16)
        if f[1].endswith("First>"): first = cp; continue
        lo = first if f[1].endswith("Last>") else cp
        if f[2] in want: rs.append((lo, cp))
    return merge(rs)
# no width: combining and enclosing marks, format characters (the soft hyphen aside, shown as a hyphen), controls,
# and the Hangul vowels and finals that join the syllable before them
def without(rs, cp):
    return [p for a, b in rs for p in ([(a, b)] if not a <= cp <= b else [(a, cp - 1)] * (a < cp) + [(cp + 1, b)] * (cp < b))]
zero = without(categories({"Mn", "Me", "Cf", "Cc"}), 0xAD)
tables = {
    "ZERO": merge(zero + [(0x1160, 0x11FF), (0xD7B0, 0xD7FF), (0x200B, 0x200B)]),
    "WIDE": prop_file("EastAsianWidth.txt", {"W", "F"}),
    "AMBIGUOUS": prop_file("EastAsianWidth.txt", {"A"}),
    "EMOJI_PRESENTATION": prop_file("emoji-data.txt", {"Emoji_Presentation"}),
    "EMOJI": prop_file("emoji-data.txt", {"Emoji"}),
}
with open(out, "w", encoding="utf-8") as o:
    o.write("//! Unicode 17.0.0's character widths on a screen, for a format width in `cols` (owner, 2026-10-25), generated\n")
    o.write("//! from Unicode's own files (EastAsianWidth.txt, emoji-data.txt, UnicodeData.txt) by tools/gen_unicode_width.py.\n")
    o.write("//! Unicode's licence: std/tools/UNICODE-LICENSE.\n\n")
    for name, rs in tables.items():
        o.write(f"pub const {name}: [(u32, u32); {len(rs)}] = [\n")
        for i in range(0, len(rs), 6):
            o.write("    " + " ".join(f"(0x{a:04X}, 0x{b:04X})," for a, b in rs[i:i+6]) + "\n")
        o.write("];\n\n")
