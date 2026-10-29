case.any;
# A file's mark covers every name written in it: functions, types, fields,
# cases, consts and variables (owner, 2026-10-06). examples/case_file.cpls
# without printing: 6 + 6 x 10 + 2 = 68.
%extern.C proc exit [let $code: i32];
const $Limit := 3;

rec Point:
    var X: i32;
    var Y: i32;
;rec Point

union.tag Mood:
    Happy;
    Grumpy;
;union.tag Mood

fn Sum [let $p: Point] -> i32:
    ret $P.x + $p.y + $LIMIT;
;fn Sum

fn Score [let $m: Mood] -> i32:
    ret match $M:
        case HAPPY:
            is 2;
        ;case HAPPY
        case grumpy:
            is 1;
        ;case grumpy
    ;match;
;fn Score

MAIN:
    let $pt := POINT[x = 1, y = 2];
    let $Total := SUM(copy $PT) + sum(copy $Pt) x 10;
    no_check my.EXIT($total + score(mood.happy));
;MAIN
