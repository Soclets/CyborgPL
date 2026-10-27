# a bin never holds a NaN; 0.0 div.floor 0.0 is a division by zero (owner, 2026-09-29: a)
!MAIN:
    let $z: float.binary.64_bit = 0.0;
    let $q: float.binary.64_bit = $z divide.nearest $z;
!end of MAIN
