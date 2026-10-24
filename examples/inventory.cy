# An inventory: range types, a function that takes any number of items, and failures.
struct Item { name: str.utf8, grams: i32[min: 1, max: 50_000] }

fn heaviest [...$items: Item] -> Item or fail[none] {
    if std::list.length(lend $items) equal_to 0 { fail none; }
    let mut $best := std::list.remove[lend.mut $items, pos: 1];
    for $it in lend $items {
        if $it.grams > $best.grams { change $best = copy $it; }
    }
    return give $best;
}

fn kilos [$grams: i32] -> i32 or fail[not_whole, out_of_range] {
    let $k: f64 = to.f64($grams) div.near 1000.0;
    return to.i32($k) otherwise fail;
}

fn initial [lent $name: str.utf8] -> char.point {
    let $first := std::str.at.point(lend $name, pos: 1) otherwise `?`;
    let $code := char.code($first);
    if $code at_least 97 and $code at_most 122 {
        return char.point.from_code($code - 32) otherwise `?`;
    }
    return $first;
}

MAIN {
    let mut $shelf: std::list[Item][min: 0, max: 4] = std::list.empty;
    std::list.append(lend.mut $shelf, value: Item[name = "rope", grams = 1_250]);
    std::list.append(lend.mut $shelf, value: Item[name = "helmet", grams = 900]);
    std::list.append(lend.mut $shelf, value: Item[name = "tent", grams = 4_000]);

    let $count := std::list.length(lend $shelf);
    let $left: i8 = 4 - $count;
    std::print.useful($count, " items, room for ", $left, " more", newln);

    let $top := heaviest(...copy $shelf) otherwise { none { std::print.useful("empty shelf", newln); std::exit(0); } };
    std::print.useful("heaviest: ", initial(lend $top.name), " ", $top.name, newln);

    let $kg := kilos($top.grams) otherwise -1;
    std::print.useful($top.name, " is ", $kg, " kg", newln);

    let $gone := std::list.remove[lend.mut $shelf, pos: 2];
    std::print.useful("removed ", $gone.name, ", ", std::list.length(lend $shelf), " left", newln);
}
