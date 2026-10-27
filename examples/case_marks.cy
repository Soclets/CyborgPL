# Names are exact unless marked (owner, 2026-10-06: "`a`, but `case.sensitive` shouldn't be an
# error"; `case.sensitive` is spelled `case.exact` since 2026-10-10). The writer marks kinds in `kit.cy` (here, the root file), or one item; an item's mark
# covers its own name, so `Mood`'s cases stay exact. `case.exact` may be written anyway.
case.any [function, variable];

case.any union.tagged Mood:
    Happy,
    Grumpy
end of case

case.exact record Exact:
    n: integer.signed.32_bit
end of case

function Describe [$m: Mood] -> string.utf8:
    return match $M:
        case Happy:
            is "happy";
        end of case Happy
        case Grumpy:
            is "grumpy";
        end of case Grumpy
    end of match;
end of function Describe

!MAIN:
    let $Count := 2;
    let $e := Exact[n = $COUNT];
    std::print.shortest(describe(MOOD.Happy), " ", DESCRIBE(mood.Grumpy), " ", $e.n, " ", $count, newline);
!end of MAIN
