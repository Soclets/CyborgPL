# A generic operator (owner, 2026-09-29), and one operator for two types by
# name (examples/generic_op.cpls as an exit code): 95 ~= 100 is true (1), 50
# ~= 100 false, A 3 <+> A 3 is 6 and B 3 <+> B 3 is 9 → 1 + 60 + 9 x 100 = 961,
# which exits as 961 mod 256 = 193.
extern.C fn exit [$code: i32];
interface Number <T> {
    op minus [$a: <T>, $b: <T>] -> <T>;
    op < [$a: <T>, $b: <T>] -> bool;
}
op ~= <T: Number> [$a: <T>, $b: <T>] -> bool {
    if $a < $b { return $b minus $a < 10; }
    return $a minus $b < 10;
}
struct A { x: i32 }
struct B { y: i32 }
op <+> [$a: A, $b: A] -> i32 { return $a.x plus $b.x; }
op <+> [$a: B, $b: B] -> i32 { return $a.y times $b.y; }
MAIN {
    let $m: i32 = 100;
    let mut $k: i32 = 0;
    if 95 ~= $m { change $k = 1; }
    if 50 ~= $m { change $k = $k plus 2; }
    let $p := A[x = 3];
    let $q := B[y = 3];
    no_check my.exit($k plus ($p <+> $p) times 10 plus ($q <+> $q) times 100);
}
