!MAIN:
    var $a: int.sign.64 := 0;
    for no_skip $i: int.sign.64 from 1 to 3 by 1:
        change $a to $a plus $i;
    end of for
!end of MAIN
