# A generic operator (owner, 2026-09-29): written once for any type that keeps its interface's
# promises, like a generic fn. An operator may also be written for several types by name; two
# meanings for the same type are an error, a generic one covering a specific one's type included.
interface Number <T>:
    op minus [$a: <T>, $b: <T>] -> <T>;
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface Number
op ~= <T: Number> [$a: <T>, $b: <T>] -> bool:
    if $a less_than $b:
        ret $b minus $a less_than 10;
    end of if
    ret $a minus $b less_than 10;
end of op ~=
rec A:
    var x: int.sign.32;
end of rec A
rec B:
    var y: int.sign.32;
end of rec B
op <+> [$a: A, $b: A] -> int.sign.32:
    ret $a.x plus $b.x;
end of op <+>
op <+> [$a: B, $b: B] -> int.sign.32:
    ret $a.y times $b.y;
end of op <+>
!MAIN:
    let $m: int.sign.32 := 100;
    std::print.shortest(95 ~= $m, " ", 50 ~= $m, newln);
    let $p := A[x = 3];
    let $q := B[y = 3];
    std::print.shortest($p <+> $p, " ", $q <+> $q, newln);
!end of MAIN
