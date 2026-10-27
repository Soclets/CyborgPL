# Generics (design/generics.md): a blank, <T>, filled per use by the compiler. Every blank is
# announced after the name with what it accepts (owner, 2026-10-02): `<T>: Number`, `<T>: any`.
interface Number:
    integer.signed.32_bit;
    integer.signed.64_bit;
    float.binary.64_bit;
end of interface Number

function max <T: Number> [$a: <T>, $b: <T>] -> <T>:
    if $a greater_than $b:
        return give $a;
    end of if     # a <T> may be owned: generic code writes the verbs
    return give $b;
end of function max

function first <T: any> [lent $xs: std::list[<T>]] -> <T> or fail[empty]:
    if std::list.length(lend $xs) equal_to 0:
        fail empty;
    end of if
    return $xs[position: 1];
end of function first

record Pair <A: any, B: any>:
    left: <A>; right: <B>;
end of record Pair     # blanks in the order announced: A, B

union.tagged Maybe <T: any>:
    none; some: <T>;
end of union.tagged Maybe

function describe [lent $p: Pair[integer.signed.32_bit, string.utf8]] -> string.utf8:
    return to.stored.string.utf8($p.left) concatenate " " concatenate $p.right;
end of function describe

!MAIN:
    std::print.shortest(max(3, 9), " ", max(2.5, 1.5), newline);       # i32 and f64 versions, one source
    let $nums := [4, 5, 6];
    let $f := first(lend $nums) otherwise 0;
    std::print.shortest($f, newline);
    let $p: Pair[integer.signed.32_bit, string.utf8] = Pair[left = 7, right = "seven"];
    std::print.shortest(describe(lend $p), newline);
    let $m := Maybe.some(42);
    match $m:
        case none:
            std::print.stored("nothing", newline);
        end of case none
        case some $v:
            std::print.shortest("got ", $v, newline);
        end of case some
    end of match
!end of MAIN
