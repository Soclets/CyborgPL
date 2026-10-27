# A length bound on a list: `list[T][lo, hi]` (decided 2026-09-21).
# Like `i32[min: 0, max: 100]`: the thing, then its bounds. Its length is a `u64[lo, hi]`.

fn fill [let $count: int.sign.32] -> std::ls[int.sign.32][min: 0, max: 5] or fail[too_many]:
    if $count gt 5:
        fail too_many;
    end of if
    var $out = std::ls.empty;
    for $i [from: 1, to: $count, by: 1]:
        std::ls.append(lend.mut $out, value: $i times $i);
    end of for
    ret give $out;
end of fn fill

!MAIN:
    var $squares: std::ls[int.sign.32][min: 0, max: 5] = fill(3) otherwise std::ls.empty;
    let $n := std::ls.len(lend $squares);    # a u64[min: 0, max: 5]
    let $small: int.sign.8 := $n;                   # proven: the bound says at most 5
    std::print.shortest($squares, " has ", $small, newln);
    std::ls.append(lend.mut $squares, value: 16);
    std::ls.append(lend.mut $squares, value: 25);       # five: still inside
    std::print.shortest(std::ls.len(lend $squares), newln);
    let $big := fill(9)-
    otherwise:
        case too_many:
            std::print.shortest("9 is too many", newln); std::exit(code: 0);
        end of case too_many
    end of otherwise;
    std::print.shortest(std::ls.len(lend $big), newln);
!end of MAIN
