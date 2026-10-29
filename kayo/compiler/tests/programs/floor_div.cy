# `/` floors and `mod` takes the divisor's sign (operators.md). The values
# come from C so that the division happens at run time, not in Precog.
%extern.C proc exit [let $code: i32];
%extern.C proc abs [let $v: i32] -> i32;
MAIN:
    no_check let $seven: i32 := abs(-7);
    let $a: i32 := (0 - $seven) div.floor 2;       # -4
    let $b: i32 := (0 - $seven) mod.floor 2;     # 1
    let $c: i32 := $seven div.floor -2;            # -4
    let $d: i32 := $seven mod.floor -2;          # -1
    no_check my.exit(($a + 4) x 1000 + ($b - 1) x 100 + ($c + 4) x 10 + ($d + 1) + 42);
;MAIN
