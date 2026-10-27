case.any;
# A file's mark, its first line, covers every name written in it: functions, types, fields, cases,
# consts and variables (owner, 2026-10-06: "`b`", at every level the `bare` marks have).
const $Limit := 3;

rec Point:
    var X: int.sign.32;
    var Y: int.sign.32;
end of rec Point

fn Sum [let $p: Point] -> int.sign.32:
    ret $P.x plus $p.y plus $LIMIT;
end of fn Sum

!MAIN:
    let $pt := POINT[x = 1, y = 2];
    std::print.shortest(SUM(copy $PT), " ", sum(copy $Pt), newln);
!end of MAIN
