# Functions as values (design/functions-as-values.md, 2026-09-21).
record Locker:
    number: integer.signed.32_bit;
    owner: string.utf8;
end of record Locker

function by_number [lent $a: Locker, lent $b: Locker] -> boolean:
    return $a.number less_than $b.number;
end of function by_number
function by_owner  [lent $a: Locker, lent $b: Locker] -> boolean:
    return std::string.bytes(lend $a.owner) less_than std::string.bytes(lend $b.owner);
end of function by_owner

# a function that takes a function: the type is the signature without the names
function first_by [lent $xs: std::list[Locker], $less: function[lent Locker, lent Locker] -> boolean] -> string.utf8:
    variable $best := 1;
    for $i [from: 2, to: std::list.length(lend $xs), by: 1]:
        if $less(lend $xs[position: $i], lend $xs[position: $best]):
            change $best = $i;
        end of if
    end of for
    let $winner := lend $xs[position: $best];
    return copy $winner.owner;
end of function first_by

function keep [lent $xs: std::list[integer.signed.32_bit], $test: function[integer.signed.32_bit] -> boolean] -> std::list[integer.signed.32_bit]:
    variable $out: std::list[integer.signed.32_bit] = [];
    for $x in lend $xs:
        if $test($x):
            std::list.append(lend.mutable $out, value: $x);
        end of if
    end of for
    return give $out;
end of function keep

!MAIN:
    let $lockers := [Locker[number = 7, owner = "Cy"], Locker[number = 2, owner = "Ann"], Locker[number = 5, owner = "Bo"]];
    std::print.stored(first_by(lend $lockers, by_number), newline);          # a named function, as a value: its bare name
    std::print.stored(first_by(lend $lockers, by_owner), newline);

    let $f := by_number;                                                 # held in a variable
    std::print.shortest($f(lend $lockers[position: 1], lend $lockers[position: 2]), newline);   # called: 'f'[…]

    let $limit := 60;
    let $scores := [72, 40, 95, 55];
    let $passing := keep(lend $scores, function [$s: integer.signed.32_bit] -> boolean with [lend $limit]:
        return $s at_least $limit;
    end of function);
    std::print.shortest($passing, newline);

    let $evens := keep(lend $scores, function [$s: integer.signed.32_bit] -> boolean:
        return ($s modulo.floor 2) equal_to 0;
    end of function);   # captures nothing
    std::print.shortest($evens, newline);
!end of MAIN
