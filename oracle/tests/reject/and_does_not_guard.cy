# both sides of `and` always run (owner, 2026-09-25: b), so this check does not
# protect the read on its right: position 7 is read, and it is out of range
MAIN {
    let 'xs' := [3, 5, 8];
    let 'i': u64 = 7;
    if 'i' at_most list.length[lend 'xs'] and 'xs'[pos 'i'] equal_to 5 { }
}
