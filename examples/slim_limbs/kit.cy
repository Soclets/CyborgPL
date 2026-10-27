# What the base was slimmed into (owner, 2026-10-07), each from the limb this kit lists: `brain16`
# (AI), `i128` and the other widths (WSMW), `deci` (Money), Unicode text and the case marks (`unicode`,
# which took in UniTect and CAPS on 2026-10-12). Testing (TAS) and range types (ASP) are the base's again.
case.any [fn];

fn Halve [let $n: int.sign.%128] -> int.sign.%128:
    ret $n div.floor 2;
end of fn Halve

test proc halves []:
    must halve(10) eq 5;
end of test proc halves

!MAIN:
    let $b: float.%brain.16 := 0.5;
    let $w: %not_std.u4 := 9;
    let $price: float.%deci.64 := 19.99;
    let $g: char.%cluster := `ก็`;
    let $age: int.sign.32[min: 0, max: 150] := 42;
    std::print.shortest(HALVE(1000000000000000000000), " ", $b, " ", $w, " ", $price, " ", $g, " ", $age, newln);
!end of MAIN
