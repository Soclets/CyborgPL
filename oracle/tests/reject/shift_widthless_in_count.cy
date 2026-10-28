# a shift's count gives nothing inside it a width (owner, 2026-10-17: "(a)")
MAIN:
    let $r: i64 := to.i64(1) shl (5 shr to.u8(1));
    std::print($r);
;MAIN
