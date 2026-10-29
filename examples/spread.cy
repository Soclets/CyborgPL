# `...$xs` at a call: unpack a list into the variadic part (decided 2026-09-21).
# Only where the list's length is known — a literal, a `std::ls[T][lo, hi]`, a bundle.

fn sum [...$nums: i32] -> i32:
    var $total = 0;
    for $n in lend $nums:
        change $total to $total + $n;
    ;for
    ret $total;
;fn sum

proc show [lent $label: str.utf8, lent ...$parts]:
    std::print($label, ":");
    for $p in lend $parts:
        std::print(" ", $p);
    ;for
    std::print.stored(newln);
;proc show

proc shout [lent ...$parts]:
    show("loud", ...$parts);
;proc shout      # a bundle passed on: its count is known

fn keep [...$names: str.utf8] -> u64:
    ret std::ls.len(lend $names);
;fn keep   # takes ownership of each

MAIN:
    let $xs := [1, 2, 3];
    std::print(sum(...$xs), newln);                   # same as sum[1, 2, 3]
    std::print(sum(10, ...$xs, 20), newln);           # mixed with hand-typed values
    var $ys: std::ls[i32][min: 0, max: 4] = std::ls.empty;
    std::ls.append(lend.mut $ys, value: 7);
    std::ls.append(lend.mut $ys, value: 8);
    std::print(sum(...$ys), newln);                   # bounded: at most four
    let $words := ["a", "b"];
    show("letters", ...$words);                            # no verb: the elements are lent
    shout(1, true);
    std::print(keep(...copy $words), " kept, ", $words, " still here", newln);
    std::print(keep(...give $words), " kept, ", $words, " left", newln);
;MAIN
