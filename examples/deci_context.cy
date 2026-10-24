# deci beside other things (compiler claude's findings, 2026-09-29): a group of bare numbers beside a deci
# takes its width; a bin's useful form is the shortest at its own width; a deci map key is its value,
# so 1.50 and 1.5 are one key (maps.md: plain values by value); map.set's key takes the key type.
MAIN {
    let $a: d64 = 1.00;
    std::print.stored($a + (1.5 x 2), " ", $a + 1.5 x 2, newln);
    let $f: f32 = 0.1;
    std::print.stored(to.useful.d64($f), " ", to.stored.d64($f), newln);
    let mut $m: std::map[d64, i32] = std::map.empty;
    std::map.set(lend.mut $m, key: 1.50, value: 7);
    std::print.stored(std::map.has(lend $m, key: 1.5), newln);
}
