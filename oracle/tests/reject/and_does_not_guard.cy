# both sides of `and` always run (owner, 2026-09-25: b), so this check does not
# protect the read on its right: position 7 is read, and it is out of range
!MAIN:
    let $xs := [3, 5, 8];
    let $i: integer.unsigned.64_bit = 7;
    if $i at_most std::list.length(lend $xs) and $xs[position: $i] equal_to 5:
    end of if
!end of MAIN
