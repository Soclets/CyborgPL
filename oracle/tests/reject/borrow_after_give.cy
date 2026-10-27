# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
!MAIN:
    let $s: std::list[integer.signed.32_bit] = [1, 2, 3];
    let $r := lend $s;
    let $t := give $s;
    std::print.shortest($r[position: 1], newline);
!end of MAIN
