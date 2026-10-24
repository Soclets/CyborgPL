# A kit that uses another (owner, 2026-09-29): CyJet.cyr lists it, `cyjet::dep.path("../kits/areas")`,
# and its `pub` items are reached with `::` after its own 'prefix', `areas` (kits.md, tooling.md).
MAIN {
    let $c: areas::Circle = areas::Circle[radius = 2.0];
    let $a := areas::circle(lend $c);
    std::print.useful($a, " ", areas::shapes.square(4), " ", areas::$pi, newln);
    std::print.useful(runnable::seven() + 1, newln);      # runnable's own MAIN does not run
    let $x: f64 = 3.0;
    let $y: f64 = 4.0;
    std::print.useful(($x mean $y), " ", ($x areas::mean $y), " ", ($x mean.low $y), " ", ($x areas::mean.low $y), newln);
    std::print.useful($x mean $y + 1.0, newln);     # areas' levels: ('x' mean 'y') plus 1.0
}
