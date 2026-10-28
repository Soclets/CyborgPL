# Generics (design/generics.md): a blank, <T>, filled per use by the compiler. Every blank is
# announced after the name with what it accepts (owner, 2026-10-02): `<T>: Number`, `<T>: any`.
set Number:
    i32;
    i64;
    f64;
;set Number

fn max <T: Number> [let $a: <T>, let $b: <T>] -> <T>:
    if $a > $b:
        ret give $a;
    ;if     # a <T> may be owned: generic code writes the verbs
    ret give $b;
;fn max

fn first <T: any> [lent $xs: std::ls[<T>]] -> <T> or fail[empty]:
    if std::ls.len(lend $xs) == 0:
        fail empty;
    ;if
    ret $xs[pos: 1];
;fn first

rec Pair <A: any, B: any>:
    var left: <A>;
    var right: <B>;
;rec Pair     # blanks in the order announced: A, B

union.tag Maybe <T: any>:
    none;
    some: <T>;
;union.tag Maybe

fn describe [lent $p: Pair[i32, str.utf8]] -> str.utf8:
    ret to.stored.str.utf8($p.left) ++ " " ++ $p.right;
;fn describe

MAIN:
    std::println(max(3, 9), " ", max(2.5, 1.5));       # i32 and f64 versions, one source
    let $nums := [4, 5, 6];
    let $f := first(lend $nums) ?? 0;
    std::println($f);
    let $p: Pair[i32, str.utf8] := Pair[left = 7, right = "seven"];
    std::println(describe(lend $p));
    let $m := Maybe.some(42);
    match $m:
        case none:
            std::println.stored("nothing");
        ;case none
        case some $v:
            std::println("got ", $v);
        ;case some
    ;match
;MAIN
