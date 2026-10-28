# One name, several versions, picked by how many arguments and their types (owner, 2026-10-20),
# written together in a `coll.fn` (owner: b), in a `versions` block that names them once (owner, 2026-10-21).
coll.fn describe:
    versions describe:
        [let $n: i64] -> str.utf8:
            ret "a whole number";
        ;version
        [let $x: f64] -> str.utf8:
            ret "a binary float";
        ;version
        [lent $s: str.utf8] -> str.utf8:
            ret "a text";
        ;version
    ;versions describe
;coll.fn describe

coll.fn width:
    versions width:
        [let $n: i16] -> str.utf8:
            ret "16 bits";
        ;version
        [let $n: i64] -> str.utf8:
            ret "64 bits";
        ;version
        [let $x: f32] -> str.utf8:
            ret "a f32";
        ;version
        [let $x: f64] -> str.utf8:
            ret "a f64";
        ;version
    ;versions width
;coll.fn width

coll.fn which:
    versions which:
        [let $x: f32] -> str.utf8:
            ret "f32";
        ;version
        [let $x: f64] -> str.utf8:
            ret "f64";
        ;version
    ;versions which
;coll.fn which

coll.fn area:
    versions area.rect:
        [let $side: i64] -> i64:
            ret $side x $side;
        ;version
        [let $w: i64, let $h: i64] -> i64:
            ret $w x $h;
        ;version
    ;versions area
;coll.fn area

MAIN:
    let $n: i64 := 7;
    let $x: f64 := 2.5;
    let $name := "Ann";
    std::println(describe($n));
    std::println(describe($x));
    std::println(describe(lend $name));
    std::println(area.rect(3), " ", area.rect(3, 4));
    let $small: i16 := 5;
    let $half: f32 := 0.5;
    let $y: f64 := 2.5;
    std::println(which(0.0 - $y), " ", which(-$half), " ", which(($half x 2.0)));
    std::println(width($small), ", ", width($n), ", ", width($half), ", ", width($x), ", ", width(2.5: f32));
;MAIN
