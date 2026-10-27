# Iterators (owner, 2026-10-08/09): a type with its own `next` returning `optional[…]` is walked by
# `for`. `fn Counter.next` belongs to `Counter` and is called `Counter.next[…]` ("`b`"); `for` takes
# the iterator with `lend.mut` (it remains, used up as far as the loop went) or `give` (gone after)
# ("`c`"). `optional` is the base's: `optional.value(…)` / `optional.none`, matched bare.

rec Counter:
    var now: int.sign.32;
    var stop: int.sign.32;
end of rec Counter

proc Counter.next [lent.mut $c: Counter] -> optional[int.sign.32]:
    if $c.now eq $c.stop:
        ret optional.none;
    end of if
    change $c.now to $c.now plus 1;
    ret optional.value($c.now);
end of proc Counter.next

rec Forever:
    var n: int.sign.32;
end of rec Forever

proc Forever.next [lent.mut $f: Forever] -> optional[int.sign.32]:
    change $f.n to $f.n plus 1;
    ret optional.value($f.n);
end of proc Forever.next

!MAIN:
    var $c = Counter[now = 0, stop = 5];
    match Counter.next(lend.mut $c):
        case value $x:
            std::print.shortest("by hand: ", $x, newln);
        end of case value
        case none:
            std::print.shortest("empty", newln);
        end of case none
    end of match
    for $x in lend.mut $c:
        if $x eq 4:
            break;
        end of if
        std::print.shortest("lent: ", $x, newln);
    end of for
    for $x in give $c:
        std::print.shortest("given: ", $x, newln);
    end of for
    var $f = Forever[n = 0];
    for perm $y in lend.mut $f:
        if $y eq 3:
            break;
        end of if
    end of for
    std::print.shortest("stopped at ", $y, newln);
!end of MAIN
