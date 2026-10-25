"""An independent check of examples/night_trains.cy: the same timetable, requests and rules written again in Python,
its output compared with the example's .out line by line. Run: python3 -I tools/check_night_trains.py ../examples."""
import sys
from decimal import Decimal, ROUND_HALF_EVEN

DEPART, CHANGE, NEAR_KM, BY_EIGHT = 1290, 10, 500, 1920
NEAR_RATE, FAR_RATE = Decimal("0.12"), Decimal("0.08")
ROWS = """Lisboa - Madrid 540 625
Madrid - Barcelona 170 621
Barcelona - Paris 395 1035
Paris - Bruxelles 85 312
Bruxelles - Amsterdam 110 210
Bruxelles - Köln 110 220
Amsterdam - Köln 160 260
Paris - Zürich 245 610
Köln - Berlin 260 580
Köln - München 270 590
Zürich - München 230 320
Zürich - Wien 470 750
München - Wien 240 435
Wien - Praha 240 330
Berlin - Praha 260 350
Wien - Kraków 400 440
Praha - Kraków 450 540
Berlin - København 420 440
København - Malmö 40 45
Malmö - Stockholm 270 615
Wien - Budapest two hours
Palermo - Messina 90 230""".split("\n")
REQUESTS = ["Ana: Lisboa > Wien, sleeper 2, youth", "Bram: Amsterdam > Kraków, couchette, adult",
            "Chloé: Paris > Stockholm, seat, senior", "Dario: Madrid > Madrid, seat, adult",
            "Eun: Berlin > Palermo, sleeper 1, adult", "Femi: Zürich > Oslo, seat, adult",
            "Gil: Praha > Lisboa, sleeper 4, adult", "Hana: Köln > Wien sleeper 2",
            "Ines: München > København, sleeper 1, senior", "Jo: Wien > Berlin, sleeper 1, youth",
            "Kai: Bruxelles > Zürich, sleeper 3, youth"]
out = []
say = out.append

def parse_link(row):
    sides = row.split(" - ")
    if len(sides) != 2: return None
    w = sides[1].split(" ")
    if len(w) < 3 or not (w[-2].isdigit() and w[-1].isdigit()): return None
    m, k = int(w[-2]), int(w[-1])
    if not (1 <= m <= 1440 and 1 <= k <= 3000): return None
    return (sides[0], " ".join(w[:-2]), m, k)

stations, links = [], []
say("the Nightjar night trains"); say(""); say("reading the timetable:")
for row in ROWS:
    l = parse_link(row)
    if l is None:
        say(f'  (skipped a row that does not read as "A - B minutes km": "{row}")'); continue
    for s in l[:2]:
        if s not in stations: stations.append(s)
    links.append(l)
say(f"  {len(links)} lines between {len(stations)} stations")

def quickest(frm):
    best = {frm: (0, 0, "")}; done = set()
    while True:
        pick, pm = None, -1
        for s in stations:
            if s in done or s not in best: continue
            if pm < 0 or best[s][0] < pm: pick, pm = s, best[s][0]
        if pm < 0: break
        done.add(pick); here = best[pick]; wait = 0 if pick == frm else CHANGE
        for a, b, m, k in links:
            if a == pick: nxt = b
            elif b == pick: nxt = a
            else: continue
            if nxt in done: continue
            mm = here[0] + wait + m
            if nxt in best and best[nxt][0] <= mm: continue
            best[nxt] = (mm, here[1] + k, pick)
    return best

def clock(t):
    day, r = divmod(t, 1440)
    txt = f"{r // 60:02}:{r % 60:02}"
    return txt if day == 0 else f"{txt} +{day}"

def board(frm):
    say(""); say(f"departures from {frm} at {clock(DEPART)}")
    say(f"  {'to':<12} {'arrives':<8} {'km':>6}")
    best = quickest(frm)
    arr = [(s, v[0], v[1]) for s, v in best.items() if s != frm]
    arr.sort(key=lambda a: a[1])          # stable, as the insertion sort is
    for s, m, k in arr: say(f"  {s:<12} {clock(DEPART + m):<8} {k:>6}")

board("Wien")

def fare(km, seat, age):
    dist = NEAR_RATE * km if km <= NEAR_KM else NEAR_RATE * NEAR_KM + FAR_RATE * (km - NEAR_KM)
    sup = {"seat": Decimal(0), "couchette": Decimal("25.00")}.get(seat) if not seat.startswith("sleeper") else Decimal("120.00") / int(seat.split()[-1])
    full = dist + sup
    off = {"adult": 0, "youth": 25, "senior": 30}[age]
    return (full - full * off / 100).quantize(Decimal("0.01"), ROUND_HALF_EVEN)

left = {"seat": 40, "couchette": 6, "sleeper for 1": 1, "sleeper for 2": 4, "sleeper for 3": 3, "sleeper for 4": 8}
say(""); say("bookings:")
bookings = []; ticket = 500
for line in REQUESTS:
    who = line.split(": ")
    parts = who[1].split(", ") if len(who) == 2 else []
    ends = parts[0].split(" > ") if len(parts) == 3 else []
    if len(ends) != 2:
        say(f'  (cannot read "{line}")'); continue
    seat_t, age = parts[1], parts[2]
    seat = seat_t if seat_t in ("seat", "couchette") else "sleeper for " + seat_t.split()[1]
    name, frm, to = who[0], ends[0], ends[1]
    if frm not in stations: say(f"  {name}: no station called {frm} on this network"); continue
    if to not in stations: say(f"  {name}: no station called {to} on this network"); continue
    if frm == to: say(f"  {name}: that journey starts where it ends"); continue
    best = quickest(frm)
    if to not in best: say(f"  {name}: no night train joins those two stations"); continue
    if left[seat] == 0: say(f"  {name}: no {seat} left tonight"); continue
    left[seat] -= 1
    stops, at = [], to
    while True:
        stops.append(at)
        if at == frm: break
        at = best[at][2]
    stops.reverse()
    m, km = best[to][0], best[to][1]
    f = fare(km, seat, age); ticket += 1
    say(f"  #{ticket} {name} ({age}), {seat}")
    say("    " + " → ".join(stops))
    say(f"    {km} km, {len(stops) - 2} change(s), leaves {clock(DEPART)}, arrives {clock(DEPART + m)}, fare {f}")
    bookings.append((name, km, f))

say(""); say("=== Nightjar, tonight ===")
say(f"tickets sold: {len(bookings)}")
tak = sum((b[2] for b in bookings), Decimal(0))
say(f"takings: {tak}")
say(f"average fare: {(tak / len(bookings)).quantize(Decimal('0.01'), ROUND_HALF_EVEN)}")
longest = max(bookings, key=lambda b: b[1])        # the first of the longest, as `>` keeps it
say(f"km travelled: {sum(b[1] for b in bookings)}, the longest {longest[1]} km ({longest[0]})")
say("places left:")
for k, v in left.items(): say(f"  {k:<14} {v:>3}")

def reached(hub):
    return sum(1 for s, v in quickest(hub).items() if s != hub and DEPART + v[0] <= BY_EIGHT)
say(""); say(f"stations reached by {clock(BY_EIGHT)}: Paris {reached('Paris')}, Köln {reached('Köln')}, Wien {reached('Wien')}")

say(""); say("stations by letter:")
for part in ["ö", "ü", "ó", "x"]:
    found = [s for s in stations if part in s]
    say(f"  with {part}: " + ", ".join(found) if found else f"  none with {part}")
say(""); say("the last quarter hours before midnight: " + " ".join(clock(t) for t in range(DEPART, 1440, 15)))
say("three fares of 0.10 as d64: 0.30")
say("the same in f64: " + format(Decimal(0.1 + 0.1 + 0.1)))
say("good night from Nightjar")

want = "\n".join(out) + "\n"
have = open(sys.argv[1] + "/night_trains.out", encoding="utf-8").read()
if want == have:
    print("night_trains.out matches the independent check:", len(out), "lines")
else:
    import difflib
    sys.stdout.writelines(difflib.unified_diff(want.splitlines(True), have.splitlines(True), "python", "night_trains.out"))
    sys.exit(1)
