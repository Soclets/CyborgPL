# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
!MAIN:
    variable $s: std::list[integer.signed.32_bit] = [1, 2, 3];
    for $x in lend $s:
        if $x equal_to 1:
            std::list.append(lend.mutable $s, value: 5);
        end of if
    end of for
!end of MAIN
