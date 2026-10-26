# A Kit's own operators (owner, 2026-10-03): areas' `mean` and `mean.low`,
# bare or with the Kit's path, `'x' areas::mean.low 'y'`. 3 and 4: the mean
# is 3.5 and the low mean 3.0, so 3.5 times 2 is 7, the low one 3, and 3
# times 10 is 30; and areas' own `precedence` puts `mean` at `times`' level, so
# 'x' mean 'y' plus 1.0 is 4.5, and twice that 9 (9cc5e25): 7 + 3 + 30 + 9 = 49.
%extern.C fn exit [$code: i32];
MAIN {
    let $x: f64 = 3.0;
    let $y: f64 = 4.0;
    let $a: i32 = to.i32(($x mean $y) times 2.0) otherwise 0;
    let $b: i32 = to.i32($x areas::mean.low $y) otherwise 0;
    let $c: i32 = to.i32(($x mean.low $y) times 10.0) otherwise 0;
    let $d: i32 = to.i32(($x areas::mean $y) times 2.0) otherwise 0;
    let $e: i32 = to.i32(($x mean $y plus 1.0) times 2.0) otherwise 0;
    no_check my.exit($a plus $b plus $c plus $d minus 7 plus $e);
}
