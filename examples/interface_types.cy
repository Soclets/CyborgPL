# Generics announced after the name, each saying what it accepts (owner, 2026-10-02). An interface
# may list types, one per line, as well as operations; a blank may need several, joined with `and`;
# lifetime groups come after the generics, a `|` between the two kinds.
interface Number:
    int.sign.32;
    int.sign.64;
    float.bin.64;
end of interface Number

interface Ordered <T>:
    op less_than [$a: <T>, $b: <T>] -> bool;
    op eq [$a: <T>, $b: <T>] -> bool;
end of interface Ordered

interface WholeOrdered <T>:
    int.sign.32;
    int.sign.64;
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface WholeOrdered

fn bigger <T: Number and Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b less_than $a:
        ret $a;
    end of if
    ret $b;
end of fn bigger

fn smaller <T: WholeOrdered> [let $a: <T>, let $b: <T>] -> <T>:
    if $a less_than $b:
        ret $a;
    end of if
    ret $b;
end of fn smaller

fn whole <T: any> [@x] [lent @x $xs: std::ls[<T>]] -> lent @x std::ls[<T>]:
    ret lend $xs;
end of fn whole

rec Box <T: Number>:
    var item: <T>;
end of rec Box

!MAIN:
    std::print.shortest(bigger(3, 9), " ", bigger(2.5, 1.5), " ", smaller(3, 9), newln);
    let $xs := ["a", "b"];
    std::print.shortest(whole(lend $xs), newln);
    let $b: Box[int.sign.32] := Box[item = 4];
    std::print.shortest($b.item, newln);
!end of MAIN
