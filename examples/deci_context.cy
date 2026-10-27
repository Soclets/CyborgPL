# deci beside other things (compiler claude's findings, 2026-09-29): a group of bare numbers beside a deci
# takes its width; a bin's useful form is the shortest at its own width; a deci map key is its value,
# so 1.50 and 1.5 are one key (maps.md: plain values by value); map.set's key takes the key type.
!MAIN:
    let $a: float.%decimal.64_bit = 1.00;
    std::print.stored($a plus (1.5 times 2), " ", $a plus 1.5 times 2, newline);
    let $f: float.binary.32_bit = 0.1;
    std::print.stored(to.shortest.float.decimal.64_bit($f), " ", to.stored.float.decimal.64_bit($f), newline);
    variable $m: std::map[float.%decimal.64_bit, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $m, key: 1.50, value: 7);
    std::print.stored(std::map.has(lend $m, key: 1.5), newline);
!end of MAIN
