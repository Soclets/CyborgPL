# A generic operator (owner, 2026-09-29): written once for any type that keeps its interface's
# promises, like a generic fn. An operator may also be written for several types by name; two
# meanings for the same type are an error, a generic one covering a specific one's type included.
interface Number <T>:
    operator minus [$a: <T>, $b: <T>] -> <T>;
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface Number
operator ~= <T: Number> [$a: <T>, $b: <T>] -> boolean:
    if $a less_than $b:
        return $b minus $a less_than 10;
    end of if
    return $a minus $b less_than 10;
end of operator ~=
record A:
    x: integer.signed.32_bit;
end of record A
record B:
    y: integer.signed.32_bit;
end of record B
operator <+> [$a: A, $b: A] -> integer.signed.32_bit:
    return $a.x plus $b.x;
end of operator <+>
operator <+> [$a: B, $b: B] -> integer.signed.32_bit:
    return $a.y times $b.y;
end of operator <+>
!MAIN:
    let $m: integer.signed.32_bit = 100;
    std::print.shortest(95 ~= $m, " ", 50 ~= $m, newline);
    let $p := A[x = 3];
    let $q := B[y = 3];
    std::print.shortest($p <+> $p, " ", $q <+> $q, newline);
!end of MAIN
