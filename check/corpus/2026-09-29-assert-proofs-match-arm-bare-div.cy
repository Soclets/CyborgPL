# Finding 2026-09-29 (check run, seed 1011, shrunk then cut by hand): in a match arm's value,
# (14576284062462565771 div 6) mod 51 is 25 (bare numbers alone are exact, dc83c05). CCA's
# --assert-proofs binary prints 16, as if the literal were a signed i64; its normal build, the
# oracle, and the same group in a let, a list item or an argument all give 25. Only the build
# with --assert-proofs is wrong. In the generated program it printed 126 for 110.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
fn put_u ['m': u64] {
    let mut 'buf': std::list[u8] = [];
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        std::list.append[lend.mut 'buf', value: to.u8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', std::list.length[lend 'buf']];
}
enum E { a, b }
MAIN {
    let 'v': u64 = match E.a { a { is (14576284062462565771 div.floor 6) mod.floor 51; } b { is 1; } };
    put_u['v'];
}
