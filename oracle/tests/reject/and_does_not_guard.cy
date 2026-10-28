# both sides of `and` always run (owner, 2026-09-25: b), so this check does not
# protect the read on its right: position 7 is read, and it is out of range
MAIN:
    let $xs := [3, 5, 8];
    let $i: u64 := 7;
    if $i <= std::ls.len(lend $xs) & $xs[pos: $i] == 5:
    ;if
;MAIN
