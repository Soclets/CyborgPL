# Names are exact unless marked (owner, 2026-10-06). Kinds marked in the root
# file, one item marked, and `case.exact` written anyway; an item's mark
# covers its own name, so `Mood`'s cases stay exact. examples/case_marks.cpls
# without printing: 1 + 2 x 10 + 2 x 100 = 221.
case.any [fn, var];
%extern.C proc exit [let $code: i32];

case.any union.tag Mood:
    Happy;
    Grumpy;
;union.tag Mood

case.exact rec Exact:
    var n: i32;
;rec Exact

fn Describe [let $m: Mood] -> i32:
    ret match $M:
        case Happy:
            is 1;
        ;case Happy
        case Grumpy:
            is 2;
        ;case Grumpy
    ;match;
;fn Describe

MAIN:
    let $Count := 2;
    let $e := Exact[n = $COUNT];
    let $fn_lit := fn [let $x: i32] -> i32:
        let $Y := $X; ret $y;
    ;fn;
    no_check my.exit(describe(MOOD.Happy) + DESCRIBE(mood.Grumpy) x 10 + $e.n x 100 + $FN_LIT[pos: 0]);
;MAIN
