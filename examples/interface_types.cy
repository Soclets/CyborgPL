# Generics announced after the name, each saying what it accepts (owner, 2026-10-02). An interface
# may list types, one per line, as well as operations; a blank may need several, joined with `and`;
# lifetime groups come after the generics, a `|` between the two kinds.
interface Number:
    integer.signed.32_bit;
    integer.signed.64_bit;
    float.binary.64_bit;
end of interface Number

interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean;
    operator equal_to [$a: <T>, $b: <T>] -> boolean;
end of interface Ordered

interface WholeOrdered <T>:
    integer.signed.32_bit;
    integer.signed.64_bit;
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface WholeOrdered

function bigger <T: Number and Ordered> [$a: <T>, $b: <T>] -> <T>:
    if $b less_than $a:
        return $a;
    end of if
    return $b;
end of function bigger

function smaller <T: WholeOrdered> [$a: <T>, $b: <T>] -> <T>:
    if $a less_than $b:
        return $a;
    end of if
    return $b;
end of function smaller

function whole <T: any> [@x] [lent @x $xs: std::list[<T>]] -> lent @x std::list[<T>]:
    return lend $xs;
end of function whole

record Box <T: Number>:
    item: <T>;
end of record Box

!MAIN:
    std::print.shortest(bigger(3, 9), " ", bigger(2.5, 1.5), " ", smaller(3, 9), newline);
    let $xs := ["a", "b"];
    std::print.shortest(whole(lend $xs), newline);
    let $b: Box[integer.signed.32_bit] = Box[item = 4];
    std::print.shortest($b.item, newline);
!end of MAIN
