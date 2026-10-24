# A negative literal in a f16, f32 or brain16 place is written at that
# width, as a positive one is: a brain16 -0.3 was stored as a f64 into its
# two bytes, so every negative brain16 went wrong (check claude, 13ca93c).
extern.C fn exit [$code: i32];
MAIN {
    let $a: brain16 = -0.3;
    let $b: brain16 = -1.75;
    let $c: f32 = -0.5;
    let $h: f16 = -2.5;
    let $n1 := to.i64($a times 10.0) otherwise -100;
    let $n2 := to.i64(($b plus $b) times 2.0) otherwise -100;
    let $n3 := to.i64($c times 4.0) otherwise -100;
    let $n4 := to.i64($h times 2.0) otherwise -100;
    let $ta := to.useful.str.utf8($a);
    let $tb := to.stored.str.utf8($b);
    let $t := to.i64(std::str.bytes(lend $ta) plus std::str.bytes(lend $tb));
    no_check my.exit(to.i32($n1 plus $n2 plus $n3 plus $n4 plus 100 plus $t));
}
