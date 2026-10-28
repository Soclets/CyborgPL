# A `return` inside a `match` arm lands in the function's return type, never in the declaration the
# match sits in (found by check claude, seed 6868): the 64-bit shift below is a u64's, not a u16's.
union.tag Pick:
    quick;
    slow;
;union.tag Pick

fn f [] -> u64:
    let $v: u16 := match Pick.quick:
        case quick:
            ret 8997704354644785536 shr 42;
        ;case quick
        case slow:
            is 1;
        ;case slow
    ;match;
    ret to.u64($v);
;fn f

MAIN:
    std::print.stored(f(), newln);
;MAIN
