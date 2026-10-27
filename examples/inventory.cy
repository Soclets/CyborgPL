# An inventory: range types, a function that takes any number of items, and failures.
rec Item:
    var name: str.utf8;
    var grams: int.sign.32[min: 1, max: 50_000];
end of rec Item

fn heaviest [var ...$items: Item] -> Item or fail[none]:
    if std::ls.len(lend $items) eq 0:
        fail none;
    end of if
    var $best = std::ls.remove(lend.mut $items, pos: 1);
    for $it in lend $items:
        if $it.grams greater_than $best.grams:
            change $best to copy $it;
        end of if
    end of for
    ret give $best;
end of fn heaviest

fn kilos [let $grams: int.sign.32] -> int.sign.32 or fail[not_whole, out_of_range]:
    let $k: float.bin.64 := to.float.bin.64($grams) div.near 1000.0;
    ret to.int.sign.32($k) otherwise fail;
end of fn kilos

fn initial [lent $name: str.utf8] -> char.%point:
    let $first := std::str.at.point(lend $name, pos: 1) otherwise `?`;
    let $code := char.code($first);
    if $code at_least 97 and $code at_most 122:
        ret char.%point.from_code($code minus 32) otherwise `?`;
    end of if
    ret $first;
end of fn initial

!MAIN:
    var $shelf: std::ls[Item][min: 0, max: 4] = std::ls.empty;
    std::ls.append(lend.mut $shelf, value: Item[name = "rope", grams = 1_250]);
    std::ls.append(lend.mut $shelf, value: Item[name = "helmet", grams = 900]);
    std::ls.append(lend.mut $shelf, value: Item[name = "tent", grams = 4_000]);

    let $count := std::ls.len(lend $shelf);
    let $left: int.sign.8 := 4 minus $count;
    std::print.shortest($count, " items, room for ", $left, " more", newln);

    let $top := heaviest(...copy $shelf)-
    otherwise:
        case none:
            std::print.shortest("empty shelf", newln); std::exit(code: 0);
        end of case none
    end of otherwise;
    std::print.shortest("heaviest: ", initial(lend $top.name), " ", $top.name, newln);

    let $kg := kilos($top.grams) otherwise neg 1;
    std::print.shortest($top.name, " is ", $kg, " kg", newln);

    let $gone := std::ls.remove(lend.mut $shelf, pos: 2);
    std::print.shortest("removed ", $gone.name, ", ", std::ls.len(lend $shelf), " left", newln);
!end of MAIN
