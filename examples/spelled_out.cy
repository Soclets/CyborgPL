# Written out for the reader. This was a `#strict` file (owner, 2026-09-27); `#strict` is gone
# (owner, 2026-10-07) and what it wrote out became the language's: `pos` on every position,
# labelled ranges, every parameter's type and a return type. `immut` and `temp` may still be
# written where they help, as here.
function ratio [$n: integer.signed.32_bit[minimum: 1]] -> integer.signed.32_bit:
    return 100 divide.floor $n;
end of function ratio
!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [10, 20, 30, 40];
    variable $total: integer.signed.32_bit = 0;
    for temporary $i: integer.unsigned.64_bit [from: 1, to: 4, by: 1]:
        change $total = $total plus $xs[position: $i];
    end of for
    let $mid := std::list.slice(lend $xs, from: position 2, to: position 3);
    let $f := function [$n: integer.signed.32_bit[minimum: 0, maximum: 1000]] -> integer.signed.32_bit:
        return $n times 2;
    end of function;   # the range is why 'n' times 2 fits
    std::print.shortest($total, " ", $mid, " ", ratio(4), " ", $f(3), newline);
!end of MAIN
