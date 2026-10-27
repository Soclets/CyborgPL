# A `return` inside a `match` arm lands in the function's return type, never in the declaration the
# match sits in (found by check claude, seed 6868): the 64-bit shift below is a u64's, not a u16's.
union.tagged Pick:
    quick; slow;
end of union.tagged Pick

function f [] -> integer.unsigned.64_bit:
    let $v: integer.unsigned.16_bit = match Pick.quick:
        case quick:
            return 8997704354644785536 shift_right 42;
        end of case quick
        case slow:
            is 1;
        end of case slow
    end of match;
    return to.integer.unsigned.64_bit($v);
end of function f

!MAIN:
    std::print.stored(f(), newline);
!end of MAIN
