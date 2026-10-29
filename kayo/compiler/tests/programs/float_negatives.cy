# A negative literal in a f16, f32 or brain16 place is written at that
# width, as a positive one is: a brain16 -0.3 was stored as a f64 into its
# two bytes, so every negative brain16 went wrong (check claude, 13ca93c).
%extern.C proc exit [let $code: i32];
MAIN:
    let $a: %bf16 := -0.3;
    let $b: %bf16 := -1.75;
    let $c: f32 := -0.5;
    let $h: f16 := -2.5;
    let $n1 := to.i64($a x 10.0) ?? -100;
    let $n2 := to.i64(($b + $b) x 2.0) ?? -100;
    let $n3 := to.i64($c x 4.0) ?? -100;
    let $n4 := to.i64($h x 2.0) ?? -100;
    let $ta := to.shortest.str.utf8($a);
    let $tb := to.stored.str.utf8($b);
    let $t := to.i64(std::str.bytes(lend $ta) + std::str.bytes(lend $tb));
    no_check my.exit(to.i32($n1 + $n2 + $n3 + $n4 + 100 + $t));
;MAIN
