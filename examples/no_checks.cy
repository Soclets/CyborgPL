# `NO_FUCKING_CHECKS` (owner, 2026-09-27): the fourth tier. Everything `no_check` turns off, and
# what C hands back is trusted too, with no `out_of_range` check on a C enum or range. Goes
# where `no_check` goes, a statement or a block, never a file; listed like `no_check`.
%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
!MAIN:
    let $msg := "hello";
    !NO_FUCKING_CHECKS let $n := write(1, lend $msg, std::str.bytes(lend $msg));
    !NO_FUCKING_CHECKS:
        let $a: int.sign.32 := 10;
        let $b: int.sign.32 := 2;
        let $q := $a div.floor $b;
        std::print.shortest(newln, $q, newln);
    !end of NO_FUCKING_CHECKS
!end of MAIN
