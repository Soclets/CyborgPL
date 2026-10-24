# `/` floors and `mod` takes the divisor's sign (operators.md). The values
# come from C so that the division happens at run time, not in Precog.
extern.C fn exit [$code: i32];
extern.C fn abs [$v: i32] -> i32;
MAIN {
    no_check let $seven: i32 = abs(-7);
    let $a: i32 = (0 minus $seven) div.floor 2;       # -4
    let $b: i32 = (0 minus $seven) mod.floor 2;     # 1
    let $c: i32 = $seven div.floor -2;            # -4
    let $d: i32 = $seven mod.floor -2;          # -1
    no_check my.exit(($a plus 4) times 1000 plus ($b minus 1) times 100 plus ($c plus 4) times 10 plus ($d plus 1) plus 42);
}
