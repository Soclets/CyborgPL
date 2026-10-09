# A check that fails with no OS freezes the CPU on a trap, `brk` (owner,
# 2026-10-08: "yea, freeze there").
fn down ['n': uint8] -> uint8 {
    return 'n' minus 1;
}

MAIN {
    let mut 'k': uint8 = 3;
    while true {
        change 'k' = down['k'];
    }
}
