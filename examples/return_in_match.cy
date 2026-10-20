# A `return` inside a `match` arm lands in the function's return type, never in the declaration the
# match sits in (found by check claude, seed 6868): the 64-bit shift below is a u64's, not a u16's.
enum Pick { quick, slow }

fn f [] -> u64 {
    let 'v': u16 = match Pick.quick {
        quick { return 8997704354644785536 shr 42; }
        slow { is 1; }
    };
    return to.u64['v'];
}

MAIN {
    std::print.stored[f[], newln];
}
