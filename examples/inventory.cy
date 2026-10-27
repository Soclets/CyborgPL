# An inventory: range types, a function that takes any number of items, and failures.
record Item:
    name: string.utf8;
    grams: integer.signed.32_bit[minimum: 1, maximum: 50_000];
end of record Item

function heaviest [...$items: Item] -> Item or fail[none]:
    if std::list.length(lend $items) equal_to 0:
        fail none;
    end of if
    variable $best := std::list.remove(lend.mutable $items, position: 1);
    for $it in lend $items:
        if $it.grams greater_than $best.grams:
            change $best = copy $it;
        end of if
    end of for
    return give $best;
end of function heaviest

function kilos [$grams: integer.signed.32_bit] -> integer.signed.32_bit or fail[not_whole, out_of_range]:
    let $k: float.binary.64_bit = to.float.binary.64_bit($grams) divide.nearest 1000.0;
    return to.integer.signed.32_bit($k) otherwise fail;
end of function kilos

function initial [lent $name: string.utf8] -> character.%code_point:
    let $first := std::string.at.code_point(lend $name, position: 1) otherwise `?`;
    let $code := character.code($first);
    if $code at_least 97 and $code at_most 122:
        return character.%code_point.from_code($code minus 32) otherwise `?`;
    end of if
    return $first;
end of function initial

!MAIN:
    variable $shelf: std::list[Item][minimum: 0, maximum: 4] = std::list.empty;
    std::list.append(lend.mutable $shelf, value: Item[name = "rope", grams = 1_250]);
    std::list.append(lend.mutable $shelf, value: Item[name = "helmet", grams = 900]);
    std::list.append(lend.mutable $shelf, value: Item[name = "tent", grams = 4_000]);

    let $count := std::list.length(lend $shelf);
    let $left: integer.signed.8_bit = 4 minus $count;
    std::print.shortest($count, " items, room for ", $left, " more", newline);

    let $top := heaviest(...copy $shelf) otherwise:
        case none:
            std::print.shortest("empty shelf", newline); std::exit(code: 0);
        end of case none
    end of otherwise;
    std::print.shortest("heaviest: ", initial(lend $top.name), " ", $top.name, newline);

    let $kg := kilos($top.grams) otherwise negative 1;
    std::print.shortest($top.name, " is ", $kg, " kg", newline);

    let $gone := std::list.remove(lend.mutable $shelf, position: 2);
    std::print.shortest("removed ", $gone.name, ", ", std::list.length(lend $shelf), " left", newline);
!end of MAIN
