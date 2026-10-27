case.any;
# A file's mark, its first line, covers every name written in it: functions, types, fields, cases,
# consts and variables (owner, 2026-10-06: "`b`", at every level the `bare` marks have).
constant $Limit := 3;

record Point:
    X: integer.signed.32_bit;
    Y: integer.signed.32_bit;
end of record Point

function Sum [$p: Point] -> integer.signed.32_bit:
    return $P.x plus $p.y plus $LIMIT;
end of function Sum

!MAIN:
    let $pt := POINT[x = 1, y = 2];
    std::print.shortest(SUM(copy $PT), " ", sum(copy $Pt), newline);
!end of MAIN
