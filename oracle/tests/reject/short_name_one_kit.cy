# a shorter name for other kits, `pub fn short = area;`, is decided (2026-09-23) and not built in the oracle yet


function area [$r: float.binary.64_bit] -> float.binary.64_bit:
    return $r times $r;
end of function area
public function short = area;
!MAIN:
!end of MAIN
