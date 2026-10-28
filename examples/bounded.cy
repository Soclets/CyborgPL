# A length bound on a list: `list[T][lo, hi]` (decided 2026-09-21).
# Like `i32[min: 0, max: 100]`: the thing, then its bounds. Its length is a `u64[lo, hi]`.

fn fill [let $count: i32] -> std::ls[i32][min: 0, max: 5] or fail[too_many]:
    if $count > 5:
        fail too_many;
    ;if
    var $out = std::ls.empty;
    for $i [from: 1, to: $count, by: 1]:
        std::ls.append(lend.mut $out, value: $i x $i);
    ;for
    ret give $out;
;fn fill

MAIN:
    var $squares: std::ls[i32][min: 0, max: 5] = fill(3) ?? std::ls.empty;
    let $n := std::ls.len(lend $squares);    # a u64[min: 0, max: 5]
    let $small: i8 := $n;                   # proven: the bound says at most 5
    std::println($squares, " has ", $small);
    std::ls.append(lend.mut $squares, value: 16);
    std::ls.append(lend.mut $squares, value: 25);       # five: still inside
    std::println(std::ls.len(lend $squares));
    let $big := fill(9)-
    otherwise:
        case too_many:
            std::println("9 is too many"); std::exit(code: 0);
        ;case too_many
    ;otherwise;
    std::println(std::ls.len(lend $big));
;MAIN
