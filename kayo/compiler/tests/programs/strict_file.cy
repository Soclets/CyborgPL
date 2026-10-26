# A `#strict` file (owner, 2026-09-27): `pos` on every position, every
# parameter's and return type, `mut`/`immut` on every declaration,
# `temp`/`perm` on every `for`, labelled ranges, and each site proven from its own function —
# the range on 'n' is why 100 / 'n' is safe (examples/strict.cpls, read out
# as an exit code; the closure's 'n' carries a range, so its multiply is
# proven there too). 100 + (20 + 30) + 25 + 6 = 181.
%extern.C fn exit [$code: i32];
fn ratio [$n: i32[min: 1]] -> i32 { return 100 div.floor $n; }
MAIN {
    let immut $xs: std::list[i32] = [10, 20, 30, 40];
    let mut $total: i32 = 0;
    for temp $i: u64 [from: 1, to: 4, by: 1] {
        change $total = $total plus $xs[pos: $i];
    }
    let immut $mid := std::list.slice(lend $xs, from: pos 2, to: pos 3);
    let immut $f := fn [$n: i32[min: 0, max: 1000]] -> i32 { return $n times 2; };
    let immut $r := ratio(4);
    let immut $six := $f[pos: 3];
    no_check my.exit($total plus $mid[pos: 1] plus $mid[pos: 2] plus $r plus $six);
}
