# `...$xs` at a call: unpack a list into the variadic part (decided 2026-09-21).
# Only where the list's length is known — a literal, a `list[T][lo, hi]`, a bundle.

fn sum [let ...$nums: int.sign.32] -> int.sign.32:
    var $total := 0;
    for $n in lend $nums:
        change $total = $total plus $n;
    end of for
    ret $total;
end of fn sum

proc show [lent $label: str.utf8, lent ...$parts]:
    std::print.shortest($label, ":");
    for $p in lend $parts:
        std::print.shortest(" ", $p);
    end of for
    std::print.stored(newln);
end of proc show

proc shout [lent ...$parts]:
    show("loud", ...$parts);
end of proc shout      # a bundle passed on: its count is known

fn keep [let ...$names: str.utf8] -> int.unsign.64:
    ret std::ls.len(lend $names);
end of fn keep   # takes ownership of each

!MAIN:
    let $xs := [1, 2, 3];
    std::print.shortest(sum(...$xs), newln);                   # same as sum[1, 2, 3]
    std::print.shortest(sum(10, ...$xs, 20), newln);           # mixed with hand-typed values
    var $ys: std::ls[int.sign.32][min: 0, max: 4] = std::ls.empty;
    std::ls.append(lend.mut $ys, value: 7);
    std::ls.append(lend.mut $ys, value: 8);
    std::print.shortest(sum(...$ys), newln);                   # bounded: at most four
    let $words := ["a", "b"];
    show("letters", ...$words);                            # no verb: the elements are lent
    shout(1, true);
    std::print.shortest(keep(...copy $words), " kept, ", $words, " still here", newln);
    std::print.shortest(keep(...give $words), " kept, ", $words, " left", newln);
!end of MAIN
