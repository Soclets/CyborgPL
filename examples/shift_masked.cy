# `shl.masked` / `shr.masked` shift as the chip does (owner, 2026-10-07): the count is masked to the
# width (70 on 64 bits is 6) and bits pushed past the width are cut off; no check site. Plain
# `shl` / `shr` stay checked. "masked" over "wrap", which reads as rotating the bits ("Whatever's accurate").
!MAIN:
    let $b: int.unsign.8 := 255;
    let $x: int.sign.64 := 1;
    let $up: int.unsign.8 := $b shl.masked 1;        # 510 cut to 8 bits: 254
    let $far: int.sign.64 := $x shl.masked 70;      # 70 masked to 6: 64
    let $down: int.unsign.8 := $b shr.masked 9;      # 9 masked to 1: 127
    let $neg: int.unsign.32 := (0 minus 1) shr.masked 1;   # -1's bits, then shifted: 2147483647
    std::print.shortest($up, " ", $far, " ", $down, " ", $neg, newln);
!end of MAIN
