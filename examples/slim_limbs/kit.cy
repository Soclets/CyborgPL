# What the base was slimmed into (owner, 2026-10-07), each from the limb this kit lists: `brain16`
# (AI), `i128` and the other widths (WSMW), `deci` (Money), Unicode text and the case marks (`unicode`,
# which took in UniTect and CAPS on 2026-10-12). Testing (TAS) and range types (ASP) are the base's again.
case.any [function];

function Halve [$n: integer.signed.%128_bit] -> integer.signed.%128_bit:
    return $n divide.floor 2;
end of function Halve

test procedure halves []:
    must halve(10) equal_to 5;
end of test procedure halves

!MAIN:
    let $b: float.%brain.16_bit = 0.5;
    let $w: %not_std.u4 = 9;
    let $price: float.%decimal.64_bit = 19.99;
    let $g: character.%grapheme_cluster = `ก็`;
    let $age: integer.signed.32_bit[minimum: 0, maximum: 150] = 42;
    std::print.shortest(HALVE(1000000000000000000000), " ", $b, " ", $w, " ", $price, " ", $g, " ", $age, newline);
!end of MAIN
