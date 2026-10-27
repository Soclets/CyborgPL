# a shorter name for other kits, `pub fn short = area;`, is decided (2026-09-23) and not built in the oracle yet


fn area [let $r: float.bin.64] -> float.bin.64:
    ret $r times $r;
end of fn area
pub fn short = area;
!MAIN:
!end of MAIN
