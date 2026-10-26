# Integers of any width, marked as not standard (owner, 2026-10-03): `not_std.i24` / `not_std.u4`. Each holds exactly its width's range; `plus.wrap` wraps at it.
MAIN {
    let $sample: %not_std.i24 = -8_000_000;
    let $flags: %not_std.u4 = 9;
    let $big: %not_std.i100 = 1_000_000_000_000_000_000;
    std::print.useful($sample, " ", $flags, " ", $big, " ", ($flags plus.wrap 9), newln);
}
