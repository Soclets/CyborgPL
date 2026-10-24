# A length bound on a list: `list[T][lo, hi]` (decided 2026-09-21).
# Like `i32[min: 0, max: 100]`: the thing, then its bounds. Its length is a `u64[lo, hi]`.

fn fill [$count: i32] -> std::list[i32][min: 0, max: 5] or fail[too_many] {
    if $count > 5 { fail too_many; }
    let mut $out := std::list.empty;
    for $i [from: 1, to: $count, by: 1] { std::list.append(lend.mut $out, value: $i x $i); }
    return give $out;
}

MAIN {
    let mut $squares: std::list[i32][min: 0, max: 5] = fill(3) otherwise std::list.empty;
    let $n := std::list.length(lend $squares);    # a u64[min: 0, max: 5]
    let $small: i8 = $n;                   # proven: the bound says at most 5
    std::print.useful($squares, " has ", $small, newln);
    std::list.append(lend.mut $squares, value: 16);
    std::list.append(lend.mut $squares, value: 25);       # five: still inside
    std::print.useful(std::list.length(lend $squares), newln);
    let $big := fill(9) otherwise { too_many { std::print.useful("9 is too many", newln); std::exit(code: 0); } };
    std::print.useful(std::list.length(lend $big), newln);
}
