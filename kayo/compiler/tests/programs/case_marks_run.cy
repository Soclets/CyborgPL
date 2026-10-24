# Names are exact unless marked (owner, 2026-10-06). Kinds marked in the root
# file, one item marked, and `case.exact` written anyway; an item's mark
# covers its own name, so `Mood`'s cases stay exact. examples/case_marks.cpls
# without printing: 1 + 2 x 10 + 2 x 100 = 221.
case.any [fn, var];
extern.C fn exit [$code: i32];

case.any enum Mood {
    Happy,
    Grumpy
}

case.exact struct Exact {
    n: i32
}

fn Describe [$m: Mood] -> i32 {
    return match $M { Happy { is 1; } Grumpy { is 2; } };
}

MAIN {
    let $Count := 2;
    let $e := Exact[n = $COUNT];
    let $fn_lit := fn [$x: i32] -> i32 { let $Y := $X; return $y; };
    no_check my.exit(describe(MOOD.Happy) plus DESCRIBE(mood.Grumpy) times 10 plus $e.n times 100 plus $FN_LIT[pos: 0]);
}
