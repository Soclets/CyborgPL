# A length bound on a list: `list[T][lo, hi]` (decided 2026-09-21).
# Like `i32[min: 0, max: 100]`: the thing, then its bounds. Its length is a `u64[lo, hi]`.

function fill [$count: integer.signed.32_bit] -> std::list[integer.signed.32_bit][minimum: 0, maximum: 5] or fail[too_many]:
    if $count greater_than 5:
        fail too_many;
    end of if
    variable $out := std::list.empty;
    for $i [from: 1, to: $count, by: 1]:
        std::list.append(lend.mutable $out, value: $i times $i);
    end of for
    return give $out;
end of function fill

!MAIN:
    variable $squares: std::list[integer.signed.32_bit][minimum: 0, maximum: 5] = fill(3) otherwise std::list.empty;
    let $n := std::list.length(lend $squares);    # a u64[min: 0, max: 5]
    let $small: integer.signed.8_bit = $n;                   # proven: the bound says at most 5
    std::print.shortest($squares, " has ", $small, newline);
    std::list.append(lend.mutable $squares, value: 16);
    std::list.append(lend.mutable $squares, value: 25);       # five: still inside
    std::print.shortest(std::list.length(lend $squares), newline);
    let $big := fill(9) otherwise:
        case too_many:
            std::print.shortest("9 is too many", newline); std::exit(code: 0);
        end of case too_many
    end of otherwise;
    std::print.shortest(std::list.length(lend $big), newline);
!end of MAIN
