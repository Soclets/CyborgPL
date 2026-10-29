# sweeps.py gen|check sigma|dots FOLDER UCD: writes a sweep's program and texts, or checks its outputs (sweeps.sh).
# UCD is the folder of Unicode 17.0.0's files (.std-checks/ucd-17.0.0).
import sys

how, which, folder, ucd = sys.argv[1:5]


def ranges(path, name):
    s = set()
    for line in open(path, encoding="utf-8"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        f = [x.strip() for x in line.split(";")]
        if f[1] != name:
            continue
        a, _, b = f[0].partition("..")
        s.update(range(int(a, 16), int(b or a, 16) + 1))
    return s


def ok(c):
    return not (0xD800 <= c <= 0xDFFF)


def write(calls):
    """calls: (function, lang or None, text). The program prints `k answer` for each."""
    lines = ["MAIN:"]
    for k, (f, lang, t) in enumerate(calls, 1):
        word = f", lang: std::casing.{lang}" if lang else ""
        lines.append(f'    let $t{k} := "{t}";')
        lines.append(f'    std::print({k}, " ", std::str.{f}.full(lend $t{k}{word}), newln);')
    lines.append(";MAIN")
    open(f"{folder}/prog/kit.cy", "w", encoding="utf-8").write("\n".join(lines) + "\n")
    open(f"{folder}/texts.txt", "w", encoding="utf-8").write(
        "\n".join(f"{f} {lang or '-'} " + t.encode("unicode_escape").decode() for f, lang, t in calls))


def read(path):
    out = {}
    for line in open(path, encoding="utf-8").read().split("\n"):
        if line:
            k, _, v = line.partition(" ")
            out[int(k)] = v
    return out


def texts():
    out = []
    for line in open(f"{folder}/texts.txt", encoding="utf-8").read().split("\n"):
        f, lang, t = line.split(" ", 2)
        out.append((f, lang, t.encode().decode("unicode_escape")))
    return out


cased = ranges(f"{ucd}/DerivedCoreProperties.txt", "Cased")
ignorable = ranges(f"{ucd}/DerivedCoreProperties.txt", "Case_Ignorable")


def table_3_17(t):
    """The σ or ς each Σ in t lowers to, by Final_Sigma as the Standard writes it (owner, 2026-10-24)."""
    r = []
    for i, ch in enumerate(t):
        if ch != "Σ":
            continue
        before = after = False
        for x in reversed(t[:i]):
            if ord(x) in cased:
                before = True
                break
            if ord(x) not in ignorable:
                break
        for x in t[i + 1:]:
            if ord(x) in cased:
                after = True
                break
            if ord(x) not in ignorable:
                break
        r.append("ς" if before and not after else "σ")
    return r


if how == "gen" and which == "sigma":
    pool = sorted(c for c in cased & ignorable if ok(c))
    pool += sorted(c for c in cased - ignorable if ok(c))[::15]
    pool += sorted(c for c in ignorable - cased if ok(c))[::10]
    pool += [0x41, 0x20, 0x2E, 0x31]
    calls = []
    for c in pool:
        ch = chr(c)
        for t in [ch + "Σ", "ΑΣ" + ch, "Α" + ch + "Σ", "ΑΣ" + ch + "Α"]:
            calls.append(("lower", None, t))
    write(calls)

elif how == "gen" and which == "dots":
    ccc, soft = {}, set()
    for line in open(f"{ucd}/UnicodeData.txt", encoding="utf-8"):
        f = line.split(";")
        if f[3] != "0":
            ccc[int(f[0], 16)] = int(f[3])
    soft = ranges(f"{ucd}/PropList.txt", "Soft_Dotted")
    marks = sorted(ccc)
    pool = [c for c in marks if ccc[c] == 230][::6] + [c for c in marks if ccc[c] != 230][::3] + [0x41, 0x20]
    calls = []
    for m in pool:
        M = chr(m)
        calls += [("lower", "tr", "I" + M + "̇"), ("lower", "tr", "Ȧ" + M), ("lower", "lt", "I" + M),
                  ("lower", "lt", "J" + M + "́"), ("lower", "lt", "Į" + M + "̀"),
                  ("upper", "lt", "i" + M + "̇"), ("upper", "lt", "j" + M + "̇"), ("upper", "tr", "i" + M)]
    for s in sorted(soft)[::3]:
        calls += [("upper", "lt", chr(s) + "̇"), ("upper", "lt", chr(s) + "̣̇")]
    write(calls)

elif how == "check":
    O, K, T = read(f"{folder}/oracle.out"), read(f"{folder}/kayo.out"), texts()
    apart = [k for k in range(1, len(T) + 1) if O.get(k) != K.get(k)]
    line = f"{which}: {len(T)} texts | the oracle's built-in and std through kayo differ on {len(apart)}"
    if which == "sigma":
        sig = lambda s: [c for c in s if c in "σς"]
        wrong_o = [k for k, (_, _, t) in enumerate(T, 1) if sig(O.get(k, "")) != table_3_17(t)]
        wrong_k = [k for k, (_, _, t) in enumerate(T, 1) if sig(K.get(k, "")) != table_3_17(t)]
        line += f" | against Table 3-17: oracle {len(wrong_o)} wrong, std {len(wrong_k)} wrong"
    print(line)
    for k in apart[:8]:
        f, lang, t = T[k - 1]
        print("  ", f, lang, " ".join("U+%04X" % ord(c) for c in t), "| oracle", O.get(k), "| std", K.get(k))
