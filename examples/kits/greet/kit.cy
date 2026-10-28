# A kit for uses_paths: other kits write its names with `greet::` (owner, 2026-10-22), and its operator `merge` bare
# or with the path, and its glyph `<+>` the same.

pub fn hello [] -> str.utf8:
    ret "hello";
;fn hello

pub fn wave [] -> str.utf8:
    ret "wave";
;fn wave

pub fn formal [] -> str.utf8:
    ret "good day";
;fn formal

pub rec Tally:
    var n: i32;
;rec Tally

pub op merge [lent $a: Tally, lent $b: Tally] -> Tally:
    ret Tally[n = $a.n + $b.n];
;op merge

pub rec Note:
    var text: str.utf8;
;rec Note

pub union.tag Tone:
    soft;
    loud;
;union.tag Tone

pub op <+> [lent $a: Tally, lent $b: Tally] -> Tally:
    ret Tally[n = $a.n + $b.n + 1];
;op <+>
