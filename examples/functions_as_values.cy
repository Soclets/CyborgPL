# Functions as values (design/functions-as-values.md, 2026-09-21).
rec Locker:
    var number: int.sign.32;
    var owner: str.utf8;
end of rec Locker

fn by_number [lent $a: Locker, lent $b: Locker] -> bool:
    ret $a.number less_than $b.number;
end of fn by_number
fn by_owner  [lent $a: Locker, lent $b: Locker] -> bool:
    ret std::str.bytes(lend $a.owner) less_than std::str.bytes(lend $b.owner);
end of fn by_owner

# a function that takes a function: the type is the signature without the names
fn first_by [lent $xs: std::ls[Locker], let $less: fn[lent Locker, lent Locker] -> bool] -> str.utf8:
    var $best = 1;
    for $i [from: 2, to: std::ls.len(lend $xs), by: 1]:
        if $less(lend $xs[pos: $i], lend $xs[pos: $best]):
            change $best to $i;
        end of if
    end of for
    let $winner := lend $xs[pos: $best];
    ret copy $winner.owner;
end of fn first_by

fn keep [lent $xs: std::ls[int.sign.32], let $test: fn[int.sign.32] -> bool] -> std::ls[int.sign.32]:
    var $out: std::ls[int.sign.32] = [];
    for $x in lend $xs:
        if $test($x):
            std::ls.append(lend.mut $out, value: $x);
        end of if
    end of for
    ret give $out;
end of fn keep

!MAIN:
    let $lockers := [Locker[number = 7, owner = "Cy"], Locker[number = 2, owner = "Ann"], Locker[number = 5, owner = "Bo"]];
    std::print.stored(first_by(lend $lockers, by_number), newln);          # a named function, as a value: its bare name
    std::print.stored(first_by(lend $lockers, by_owner), newln);

    let $f := by_number;                                                 # held in a variable
    std::print.shortest($f(lend $lockers[pos: 1], lend $lockers[pos: 2]), newln);   # called: 'f'[…]

    let $limit := 60;
    let $scores := [72, 40, 95, 55];
    let $passing := keep(lend $scores, fn [$s: int.sign.32] -> bool with [lend $limit]:
        ret $s at_least $limit;
    end of fn);
    std::print.shortest($passing, newln);

    let $evens := keep(lend $scores, fn [$s: int.sign.32] -> bool:
        ret ($s mod.floor 2) eq 0;
    end of fn);   # captures nothing
    std::print.shortest($evens, newln);
!end of MAIN
