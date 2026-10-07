# nothing converts by itself: an array into a list is said (owner, 2026-09-23)
fn grow ['xs': list[int32]] -> uint64 { return list.length[lend 'xs']; }
MAIN {
    let 'f': array[int32] = [1, 2];
    let 'n' := grow[give 'f'];
}
