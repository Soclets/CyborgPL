# A `return` inside a `match` arm lands in the function's return type, never in the declaration the
# match sits in (found by check claude, seed 6868): the 64-bit shift below is a u64's, not a u16's.
union.tag Pick:
    quick;
    slow;
end of union.tag Pick

fn f [] -> int.unsign.64:
    let $v: int.unsign.16 := match Pick.quick:
        case quick:
            ret 8997704354644785536 shr 42;
        end of case quick
        case slow:
            is 1;
        end of case slow
    end of match;
    ret to.int.unsign.64($v);
end of fn f

!MAIN:
    std::print.stored(f(), newln);
!end of MAIN
