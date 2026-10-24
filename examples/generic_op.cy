# A generic operator (owner, 2026-09-29): written once for any type that keeps its interface's
# promises, like a generic fn. An operator may also be written for several types by name; two
# meanings for the same type are an error, a generic one covering a specific one's type included.
interface Number <T> {
    op minus [$a: <T>, $b: <T>] -> <T>;
    op < [$a: <T>, $b: <T>] -> bool;
}
op ~= <T: Number> [$a: <T>, $b: <T>] -> bool {
    if $a < $b { return $b - $a < 10; }
    return $a - $b < 10;
}
struct A { x: i32 }
struct B { y: i32 }
op <+> [$a: A, $b: A] -> i32 { return $a.x + $b.x; }
op <+> [$a: B, $b: B] -> i32 { return $a.y x $b.y; }
MAIN {
    let $m: i32 = 100;
    std::print.useful(95 ~= $m, " ", 50 ~= $m, newln);
    let $p := A[x = 3];
    let $q := B[y = 3];
    std::print.useful($p <+> $p, " ", $q <+> $q, newln);
}
