# deci beside other things (compiler claude's findings, 2026-09-29): a group of bare numbers beside a deci
# takes its width; a bin's useful form is the shortest at its own width; a deci map key is its value,
# so 1.50 and 1.5 are one key (maps.md: plain values by value); map.set's key takes the key type.
!MAIN:
    let $a: float.%deci.64 := 1.00;
    std::print.stored($a plus (1.5 times 2), " ", $a plus 1.5 times 2, newln);
    let $f: float.bin.32 := 0.1;
    std::print.stored(to.shortest.float.%deci.64($f), " ", to.stored.float.%deci.64($f), newln);
    var $m: std::map[float.%deci.64, int.sign.32] = std::map.empty;
    std::map.set(lend.mut $m, key: 1.50, value: 7);
    std::print.stored(std::map.has(lend $m, key: 1.5), newln);
!end of MAIN
