!MAIN:
    variable $a: integer.signed.64_bit = 0;
    for no_skip $i: integer.signed.64_bit from 1 to 3 by 1:
        change $a = $a plus $i;
    end of for
!end of MAIN
