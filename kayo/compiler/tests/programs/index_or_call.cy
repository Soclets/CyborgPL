# `pos` is removed (owner, 2026-09-27): 'x'[…] reads a list when 'x' holds
# one and calls a function when it holds one; a position can be any
# whole number, a 65-bit exact answer included. 3 + 30 + 4 = 37 (oracle).
fn inc [$n: i32] -> i32 { return $n plus 1; }
fn add [$a: i32, $b: i32] -> i32 { return $a plus $b; }
%extern.C fn exit [$code: i32];
MAIN {
    let $f := inc;
    let $g := add;
    let $xs: std::list[i32] = [4, 5, 6];
    let $a := $f[pos: 2];
    let $c := $g(10, 20);
    no_check my.exit($a plus $c plus $xs[pos: $xs[pos: 1] minus 3]);
}
