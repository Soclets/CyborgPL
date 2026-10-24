# Generics (design/generics.md): a blank, <T>, filled per use by the compiler. Every blank is
# announced after the name with what it accepts (owner, 2026-10-02): `<T>: Number`, `<T>: any`.
interface Number {
    i32
    i64
    f64
}

fn max <T: Number> [$a: <T>, $b: <T>] -> <T> {
    if $a > $b { return give $a; }     # a <T> may be owned: generic code writes the verbs
    return give $b;
}

fn first <T: any> [lent $xs: std::list[<T>]] -> <T> or fail[empty] {
    if std::list.length(lend $xs) equal_to 0 { fail empty; }
    return $xs[pos: 1];
}

struct Pair <A: any, B: any> { left: <A>, right: <B> }     # blanks in the order announced: A, B

enum Maybe <T: any> { none, some: <T> }

fn describe [lent $p: Pair[i32, str.utf8]] -> str.utf8 {
    return to.stored.str.utf8($p.left) concat " " concat $p.right;
}

MAIN {
    std::print.useful(max(3, 9), " ", max(2.5, 1.5), newln);       # i32 and f64 versions, one source
    let $nums := [4, 5, 6];
    let $f := first(lend $nums) otherwise 0;
    std::print.useful($f, newln);
    let $p: Pair[i32, str.utf8] = Pair[left = 7, right = "seven"];
    std::print.useful(describe(lend $p), newln);
    let $m := Maybe.some(42);
    match $m {
        none     { std::print.stored("nothing", newln); }
        some $v { std::print.useful("got ", $v, newln); }
    }
}
