rec Point:
    var x: int.sign.32;
    var y: int.sign.32;
end of rec Point
!MAIN:
    var $p := Point[x = 1, y = 2];
    for $v in lend.mut $p:
    end of for
!end of MAIN
