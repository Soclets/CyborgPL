# An inventory: range types, a function that takes any number of items, and failures.
rec Item:
    var name: str.utf8;
    var grams: i32[min: 1, max: 50_000];
;rec Item

fn heaviest [var ...$items: Item] -> Item or fail[none]:
    if std::ls.len(lend $items) == 0:
        fail none;
    ;if
    var $best = std::ls.remove(lend.mut $items, pos: 1);
    for $it in lend $items:
        if $it.grams > $best.grams:
            change $best to copy $it;
        ;if
    ;for
    ret give $best;
;fn heaviest

fn kilos [let $grams: i32] -> i32 or fail[not_whole, out_of_range]:
    let $k: f64 := to.f64($grams) div.near 1000.0;
    ret to.i32($k)?;
;fn kilos

fn initial [lent $name: str.utf8] -> char.%point:
    let $first := std::str.at.point(lend $name, pos: 1) ?? `?`;
    let $code := char.code($first);
    if $code >= 97 & $code <= 122:
        ret char.%point.from_code($code - 32) ?? `?`;
    ;if
    ret $first;
;fn initial

MAIN:
    var $shelf: std::ls[Item][min: 0, max: 4] = std::ls.empty;
    std::ls.append(lend.mut $shelf, value: Item[name = "rope", grams = 1_250]);
    std::ls.append(lend.mut $shelf, value: Item[name = "helmet", grams = 900]);
    std::ls.append(lend.mut $shelf, value: Item[name = "tent", grams = 4_000]);

    let $count := std::ls.len(lend $shelf);
    let $left: i8 := 4 - $count;
    std::println($count, " items, room for ", $left, " more");

    let $top := heaviest(...copy $shelf)-
    otherwise:
        case none:
            std::println("empty shelf"); std::exit(code: 0);
        ;case none
    ;otherwise;
    std::println("heaviest: ", initial(lend $top.name), " ", $top.name);

    let $kg := kilos($top.grams) ?? -1;
    std::println($top.name, " is ", $kg, " kg");

    let $gone := std::ls.remove(lend.mut $shelf, pos: 2);
    std::println("removed ", $gone.name, ", ", std::ls.len(lend $shelf), " left");
;MAIN
