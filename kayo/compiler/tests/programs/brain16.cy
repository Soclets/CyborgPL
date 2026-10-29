# The brain float (owner, 2026-10-02): `brain16` (`bf16` and `bfloat16` were old spellings). Literals
# round to the nearest, each step rounds, and it converts to and from every
# number family and text, both forms of each. The oracle gives 66.
%extern.C proc exit [$code: i32];
MAIN:
    let $a: %bf16 := 0.1;
    let $b: %bf16 := 0.2;
    let $c: %bf16 := $a + $b;
    let $x: f64 := 300.7;
    let $w := to.%bf16($x);
    let $back := to.f32($w);
    let $n := to.i32($w) ?? -1;
    let $cc := to.i32(to.f64($c) x 100000000.0) ?? -1;
    let $s := to.shortest.str.utf8($a);
    let $t := to.stored.str.utf8($a);
    let $d := to.shortest.%d64($c);
    let $ds := to.stored.%d64($a);
    let $p := to.%bf16("2.5") ?? 0;
    let $q := to.%bf16(100000);
    let $xs: std::ls[%bf16] := [0.5, 1.5, 2.5];
    let $h := to.f16($w);
    let $k1 := to.i32($d x 10) ?? -1;
    let $k2 := to.i32($ds x 100000000000) ?? -1;
    let $k3 := to.i32($p x $xs[pos: 3]) ?? -1;
    let $k4 := to.i32($q) ?? -1;
    let $k5 := to.i32($h) ?? -1;
    let $k6 := to.i32($back) ?? -1;
    let $total := $n + ($cc mod.floor 1000) + std::str.bytes(lend $s) + std::str.bytes(lend $t) + $k1 + ($k2 mod.floor 1000) + $k3 + ($k4 mod.floor 1000) + $k5 + $k6;
    no_check my.exit(to.i32($total mod.floor 256));
;MAIN
