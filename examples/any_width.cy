# Integers of any width, marked as not standard (owner, 2026-10-03): `int.sign.%not_std.24` / `int.unsign.%not_std.4` since 2026-10-27. Each holds exactly its width's range; `plus.wrap` wraps at it.
!MAIN:
    let $sample: int.sign.%not_std.24 := neg 8_000_000;
    let $flags: int.unsign.%not_std.4 := 9;
    let $big: int.sign.%not_std.100 := 1_000_000_000_000_000_000;
    std::print.shortest($sample, " ", $flags, " ", $big, " ", ($flags plus.wrap 9), newln);
!end of MAIN
