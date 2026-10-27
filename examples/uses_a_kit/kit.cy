# A kit that uses another (owner, 2026-09-29): CyJet.cyr lists it, `cyjet::dep.path["../kits/areas"]`,
# and its `pub` items are reached with `::` after its own 'prefix', `areas` (kits.md, tooling.md).
!MAIN:
    let $c: areas::Circle = areas::Circle[radius = 2.0];
    let $a := areas::circle(lend $c);
    std::print.shortest($a, " ", areas::shapes.square(4), " ", areas::$pi, newline);
    std::print.shortest(runnable::seven() plus 1, newline);      # runnable's own MAIN does not run
    let $x: float.binary.64_bit = 3.0;
    let $y: float.binary.64_bit = 4.0;
    std::print.shortest(($x mean $y), " ", ($x areas::mean $y), " ", ($x mean.low $y), " ", ($x areas::mean.low $y), newline);
    std::print.shortest($x mean $y plus 1.0, newline);     # areas' levels: ('x' mean 'y') plus 1.0
!end of MAIN
