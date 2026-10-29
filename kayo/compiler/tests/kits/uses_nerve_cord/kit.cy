# Lists the Nerve cord (owner, 2026-10-14): `std::twice` is its (another kit's names take its path, 2026-10-22), `std::print` falls
# back to the built-in, which it has not written, its module `str` is reached as `std::str.shout` and `std::str.at.byte[…, pos 2]` (2 + 103), and its `exit` takes over, so the run ends 0, not 7.

MAIN:
    let $xs: std::ls[i64] := [1, 2, 3];
    std::print(std::twice(4), " ", std::twice(5), " ", helper::doubled(6), " ", std::total(lend $xs), newln);
    let $h := "hi";
    let $i := std::str.at.byte(lend $h, pos: 2) ?? 0;
    std::print(std::str.shout(lend $h), " ", $i, newln);
    std::exit(code: 7);
;MAIN
