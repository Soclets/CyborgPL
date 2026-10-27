# a shift's count gives nothing inside it a width (owner, 2026-10-17: "(a)")
!MAIN:
    let $r: int.sign.64 := to.int.sign.64(1) shl (5 shr to.int.unsign.8(1));
    std::print.shortest($r);
!end of MAIN
