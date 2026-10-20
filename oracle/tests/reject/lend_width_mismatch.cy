# a lend hands over the value where it is, so its sizes must match the parameter's (owner, 2026-09-24: a)
fn total [lent 'xs': list[i32]] -> u64 { return list.length[lend 'xs']; }
MAIN {
    let 'big': list[i64] = [1, 2, 3];
    let 'n' := total[lend 'big'];
}
