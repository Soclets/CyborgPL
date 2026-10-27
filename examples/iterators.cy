# Iterators (owner, 2026-10-08/09): a type with its own `next` returning `optional[…]` is walked by
# `for`. `fn Counter.next` belongs to `Counter` and is called `Counter.next[…]` ("`b`"); `for` takes
# the iterator with `lend.mut` (it remains, used up as far as the loop went) or `give` (gone after)
# ("`c`"). `optional` is the base's: `optional.value(…)` / `optional.none`, matched bare.

record Counter:
    now: integer.signed.32_bit;
    stop: integer.signed.32_bit;
end of record Counter

procedure Counter.next [lent.mutable $c: Counter] -> optional[integer.signed.32_bit]:
    if $c.now equal_to $c.stop:
        return optional.none;
    end of if
    change $c.now = $c.now plus 1;
    return optional.value($c.now);
end of procedure Counter.next

record Forever:
    n: integer.signed.32_bit;
end of record Forever

procedure Forever.next [lent.mutable $f: Forever] -> optional[integer.signed.32_bit]:
    change $f.n = $f.n plus 1;
    return optional.value($f.n);
end of procedure Forever.next

!MAIN:
    variable $c := Counter[now = 0, stop = 5];
    match Counter.next(lend.mutable $c):
        case value $x:
            std::print.shortest("by hand: ", $x, newline);
        end of case value
        case none:
            std::print.shortest("empty", newline);
        end of case none
    end of match
    for $x in lend.mutable $c:
        if $x equal_to 4:
            break;
        end of if
        std::print.shortest("lent: ", $x, newline);
    end of for
    for $x in give $c:
        std::print.shortest("given: ", $x, newline);
    end of for
    variable $f := Forever[n = 0];
    for permanent $y in lend.mutable $f:
        if $y equal_to 3:
            break;
        end of if
    end of for
    std::print.shortest("stopped at ", $y, newline);
!end of MAIN
