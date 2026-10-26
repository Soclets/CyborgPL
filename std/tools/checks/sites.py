# sites.py REPORT LABEL: std's check sites in a kayo build report, by verdict, and whether Precog counted the
# program's inputs ("ran the program over what its inputs allow"). Without that, its proofs hold only for the
# inputs it ran, not for any caller.
import collections
import sys

report, label = sys.argv[1], sys.argv[2]
text = open(report, encoding="utf-8").read()
verdicts, unproven = collections.Counter(), []
for line in text.splitlines():
    if not line.startswith("| /"):
        continue
    cols = [c.strip() for c in line.strip().strip("|").split("|")]
    if "/std/" not in cols[0] or "/tools/checks/" in cols[0]:
        continue
    if len(cols) > 4 and cols[4] in ("proven", "checked at run time", "not checked on your word"):
        verdicts[cols[4]] += 1
        if cols[4] != "proven":
            unproven.append(f"  {cols[0]}: {cols[2]} ({cols[4]})")
inputs = "inputs counted" if "over what its inputs allow" in text else "INPUTS NOT COUNTED"
print(f"{label}: std's sites {dict(verdicts)}, {inputs}")
for u in unproven[:8]:
    print(u)
