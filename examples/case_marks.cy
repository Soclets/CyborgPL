# Names are exact unless marked (owner, 2026-10-06: "`a`, but `case.sensitive` shouldn't be an
# error"; `case.sensitive` is spelled `case.exact` since 2026-10-10). The writer marks kinds in `kit.cy` (here, the root file), or one item; an item's mark
# covers its own name, so `Mood`'s cases stay exact. `case.exact` may be written anyway.
case.any [fn, var];

case.any enum Mood {
    Happy,
    Grumpy
}

case.exact struct Exact {
    n: i32
}

fn Describe [$m: Mood] -> str.utf8 {
    return match $M { Happy { is "happy"; } Grumpy { is "grumpy"; } };
}

MAIN {
    let $Count := 2;
    let $e := Exact[n = $COUNT];
    std::print.useful(describe(MOOD.Happy), " ", DESCRIBE(mood.Grumpy), " ", $e.n, " ", $count, newln);
}
