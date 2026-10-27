# `...$xs` at a call: unpack a list into the variadic part (decided 2026-09-21).
# Only where the list's length is known — a literal, a `list[T][lo, hi]`, a bundle.

function sum [...$nums: integer.signed.32_bit] -> integer.signed.32_bit:
    variable $total := 0;
    for $n in lend $nums:
        change $total = $total plus $n;
    end of for
    return $total;
end of function sum

procedure show [lent $label: string.utf8, lent ...$parts]:
    std::print.shortest($label, ":");
    for $p in lend $parts:
        std::print.shortest(" ", $p);
    end of for
    std::print.stored(newline);
end of procedure show

procedure shout [lent ...$parts]:
    show("loud", ...$parts);
end of procedure shout      # a bundle passed on: its count is known

function keep [...$names: string.utf8] -> integer.unsigned.64_bit:
    return std::list.length(lend $names);
end of function keep   # takes ownership of each

!MAIN:
    let $xs := [1, 2, 3];
    std::print.shortest(sum(...$xs), newline);                   # same as sum[1, 2, 3]
    std::print.shortest(sum(10, ...$xs, 20), newline);           # mixed with hand-typed values
    variable $ys: std::list[integer.signed.32_bit][minimum: 0, maximum: 4] = std::list.empty;
    std::list.append(lend.mutable $ys, value: 7);
    std::list.append(lend.mutable $ys, value: 8);
    std::print.shortest(sum(...$ys), newline);                   # bounded: at most four
    let $words := ["a", "b"];
    show("letters", ...$words);                            # no verb: the elements are lent
    shout(1, true);
    std::print.shortest(keep(...copy $words), " kept, ", $words, " still here", newline);
    std::print.shortest(keep(...give $words), " kept, ", $words, " left", newline);
!end of MAIN
