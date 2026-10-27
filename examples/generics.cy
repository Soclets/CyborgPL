# Generics (design/generics.md): a blank, <T>, filled per use by the compiler. Every blank is
# announced after the name with what it accepts (owner, 2026-10-02): `<T>: Number`, `<T>: any`.
sig Number:
    int.sign.32;
    int.sign.64;
    float.bin.64;
end of sig Number

fn max <T: Number> [let $a: <T>, let $b: <T>] -> <T>:
    if $a gt $b:
        ret give $a;
    end of if     # a <T> may be owned: generic code writes the verbs
    ret give $b;
end of fn max

fn first <T: any> [lent $xs: std::ls[<T>]] -> <T> or fail[empty]:
    if std::ls.len(lend $xs) eq 0:
        fail empty;
    end of if
    ret $xs[pos: 1];
end of fn first

rec Pair <A: any, B: any>:
    var left: <A>;
    var right: <B>;
end of rec Pair     # blanks in the order announced: A, B

union.tag Maybe <T: any>:
    none;
    some: <T>;
end of union.tag Maybe

fn describe [lent $p: Pair[int.sign.32, str.utf8]] -> str.utf8:
    ret to.stored.str.utf8($p.left) concat " " concat $p.right;
end of fn describe

!MAIN:
    std::print.shortest(max(3, 9), " ", max(2.5, 1.5), newln);       # i32 and f64 versions, one source
    let $nums := [4, 5, 6];
    let $f := first(lend $nums) otherwise 0;
    std::print.shortest($f, newln);
    let $p: Pair[int.sign.32, str.utf8] := Pair[left = 7, right = "seven"];
    std::print.shortest(describe(lend $p), newln);
    let $m := Maybe.some(42);
    match $m:
        case none:
            std::print.stored("nothing", newln);
        end of case none
        case some $v:
            std::print.shortest("got ", $v, newln);
        end of case some
    end of match
!end of MAIN
