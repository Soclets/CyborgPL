# f16/f32 math rounds after every step, and a bare literal takes the
# typed side's width (owner, 2026-09-24: a): 2^24 + 1.0 at f32 is 2^24.
# With a f64 it is done at the wider (the oracle's pick). A literal in a
# comparison stays at 64 bits: a f32 0.1 is a little more than 0.1.
%extern.C fn exit [$code: i32];
MAIN {
    let $x: f32 = 16777216.0;
    let mut $got: i32 = 0;
    let $a := $x plus 1.0;
    let $na := to.i32($a minus 16777216.0) otherwise 99;
    if $na equal_to 0 { change $got = $got plus 1; }
    let $b: f32 = $x plus 1.0 plus 1.0;
    let $nb := to.i32($b minus 16777216.0) otherwise 99;
    let $w: f64 = 1.0;
    let $nw := to.i32($x plus $w minus 16777216.0) otherwise 99;
    if $nb equal_to 0 and $nw equal_to 1 { change $got = $got plus 2; }
    let $c: f32 = 0.1;
    if $c > 0.1 and not ($c equal_to 0.1) { change $got = $got plus 4; }
    let $d := $c times 10.0;
    if $d > 0.99 and $d < 1.01 { change $got = $got plus 8; }
    let mut $h: i32 = 0;
    check if $x > 0.0 { change $h = to.i32($x div.near 2.0) otherwise 0; }
    if $h equal_to 8388608 { change $got = $got plus 16; }
    no_check my.exit($got);
}
