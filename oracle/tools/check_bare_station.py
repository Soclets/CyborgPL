"""An independent check of examples/bare_station: the same log, read and worked out again in Python, its output
compared with the example's .out line by line. Run: python3 -I tools/check_bare_station.py ../examples."""
import sys
import zlib

ex = sys.argv[1] + "/bare_station"
raw = open(ex + "/data/log.txt", "rb").read()
temps = [int(l) for l in raw.decode().split("\n") if l.strip() and not l.startswith("#")]


def tenths(t):
    return ("-" if t < 0 else "") + f"{abs(t) // 10}.{abs(t) % 10}"


def hour(h):
    return f"{h:02}:00"


def hexw(x):
    s = f"{x:08x}"
    return f"0x{s[:4]}_{s[4:]}"


def div_near(a, b):
    """CyborgPL's `div.near`: to the nearest, a half to even."""
    q, r = divmod(a, b)
    if 2 * r > b or (2 * r == b and q % 2 == 1):
        q += 1
    return q


def alarm(t):
    return 2 if t > 250 else 1 if t > 220 else 0


names = ["calm", "warm", "hot"]
out = ["station 7, booting on a chip with no operating system",
       f"log: {len(raw)} bytes, CRC-32 {hexw(zlib.crc32(raw))}"]
line, now, changes = "alarm:  calm", 0, 0
for h, t in enumerate(temps):
    a = alarm(t)
    if a != now:
        line += f" -> {names[a]} at {hour(h)}"
        changes += 1
        now = a
out.append(line)
low = min(temps); high = max(temps)
steps = [(temps[h] - temps[h - 1], h) for h in range(1, len(temps))]
rise, rise_h = steps[0]
for s, h in steps:
    if abs(s) > abs(rise):
        rise, rise_h = s, h
mean = div_near(sum(temps), len(temps))
warm = sum(1 for t in temps if alarm(t) >= 1); hot = sum(1 for t in temps if alarm(t) == 2)
out += [f"readings: {len(temps)}, in tenths of a degree",
        f"lowest:  {tenths(low)} C at {hour(temps.index(low))}",
        f"highest: {tenths(high)} C at {hour(temps.index(high))}",
        f"mean:    {tenths(mean)} C, {tenths(div_near(mean * 9, 5) + 320)} F",
        f"steepest hour: {'+' if rise > 0 else ''}{tenths(rise)} C, from {hour(rise_h - 1)} to {hour(rise_h)}",
        f"warm for {warm} hours, hot for {hot}",
        f"status word: {hexw((len(temps) << 24) | (warm << 16) | (hot << 8) | changes)}",
        "switching off"]
want = "\n".join(out) + "\n"
have = open(sys.argv[1] + "/bare_station.out", encoding="utf-8").read()
if want == have:
    print("bare_station.out matches the independent check:", len(out), "lines")
else:
    import difflib
    sys.stdout.writelines(difflib.unified_diff(want.splitlines(True), have.splitlines(True), "python", "bare_station.out"))
    sys.exit(1)
