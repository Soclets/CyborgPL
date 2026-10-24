# The brain float (owner, 2026-10-02): `brain16` (`bf16` and `bfloat16` were old spellings). Literals
# round to the nearest, each step rounds, and it converts to and from every
# number family and text, both forms of each. The oracle gives 66.
extern.C fn exit [$code: i32];
MAIN {
    let $a: brain16 = 0.1;
    let $b: brain16 = 0.2;
    let $c: brain16 = $a plus $b;
    let $x: f64 = 300.7;
    let $w := to.brain16($x);
    let $back := to.f32($w);
    let $n := to.i32($w) otherwise -1;
    let $cc := to.i32(to.f64($c) times 100000000.0) otherwise -1;
    let $s := to.useful.str.utf8($a);
    let $t := to.stored.str.utf8($a);
    let $d := to.useful.d64($c);
    let $ds := to.stored.d64($a);
    let $p := to.brain16("2.5") otherwise 0;
    let $q := to.brain16(100000);
    let $xs: std::list[brain16] = [0.5, 1.5, 2.5];
    let $h := to.f16($w);
    let $k1 := to.i32($d times 10) otherwise -1;
    let $k2 := to.i32($ds times 100000000000) otherwise -1;
    let $k3 := to.i32($p times $xs[pos: 3]) otherwise -1;
    let $k4 := to.i32($q) otherwise -1;
    let $k5 := to.i32($h) otherwise -1;
    let $k6 := to.i32($back) otherwise -1;
    let $total := $n plus ($cc mod.floor 1000) plus std::str.bytes(lend $s) plus std::str.bytes(lend $t) plus $k1 plus ($k2 mod.floor 1000) plus $k3 plus ($k4 mod.floor 1000) plus $k5 plus $k6;
    no_check my.exit(to.i32($total mod.floor 256));
}
