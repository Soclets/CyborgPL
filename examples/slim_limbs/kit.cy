# What the base was slimmed into (owner, 2026-10-07), each from the limb this kit lists: `brain16`
# (AI), `i128` and the other widths (WSMW), `deci` (Money), Unicode text and the case marks (`unicode`,
# which took in UniTect and CAPS on 2026-10-12). Testing (TAS) and range types (ASP) are the base's again.
case.any [fn];

fn Halve [$n: i128] -> i128 {
    return $n div.floor 2;
}

test fn halves [] {
    must halve(10) equal_to 5;
}

MAIN {
    let $b: brain16 = 0.5;
    let $w: not_std.u4 = 9;
    let $price: d64 = 19.99;
    let $g: char.grapheme = `ก็`;
    let $age: i32[min: 0, max: 150] = 42;
    std::print.useful(HALVE(1000000000000000000000), " ", $b, " ", $w, " ", $price, " ", $g, " ", $age, newln);
}
