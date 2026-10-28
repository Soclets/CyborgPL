# What the base was slimmed into (owner, 2026-10-07), each from the limb this kit lists: `brain16`
# (AI), `i128` and the other widths (WSMW), `deci` (Money), Unicode text (`unicode`,
# which took in UniTect and CAPS on 2026-10-12). Testing (TAS) and range types (ASP) are the base's again.
fn Halve [let $n: %i128] -> %i128:
    ret $n div.floor 2;
;fn Halve

test proc halves []:
    must Halve(10) == 5;
;test proc halves

MAIN:
    let $b: %bf16 := 0.5;
    let $w: %not_std.u4 := 9;
    let $price: %d64 := 19.99;
    let $g: char.%cluster := `ก็`;
    let $age: i32[min: 0, max: 150] := 42;
    std::println(Halve(1000000000000000000000), " ", $b, " ", $w, " ", $price, " ", $g, " ", $age);
;MAIN
