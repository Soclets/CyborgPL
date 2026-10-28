# Generics announced after the name, each saying what it accepts (owner, 2026-10-02). A `set` lists
# types, one per line, a `sig` operations, never both (owner, 2026-10-27); a blank may need several,
# joined with `and`; lifetime groups come after the generics, a `|` between the two kinds.
set Number:
    i32;
    i64;
    f64;
;set Number

sig Ordered <T>:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
    op == [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered

set Whole:
    i32;
    i64;
;set Whole

fn bigger <T: Number and Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b < $a:
        ret $a;
    ;if
    ret $b;
;fn bigger

fn smaller <T: Whole and Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $a < $b:
        ret $a;
    ;if
    ret $b;
;fn smaller

fn whole <T: any> [@x] [lent @x $xs: std::ls[<T>]] -> lent @x std::ls[<T>]:
    ret lend $xs;
;fn whole

rec Box <T: Number>:
    var item: <T>;
;rec Box

MAIN:
    std::print(bigger(3, 9), " ", bigger(2.5, 1.5), " ", smaller(3, 9), newln);
    let $xs := ["a", "b"];
    std::print(whole(lend $xs), newln);
    let $b: Box[i32] := Box[item = 4];
    std::print($b.item, newln);
;MAIN
