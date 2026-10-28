# A shop: an interface, an enum, structs and prices.
sig Priced <T>:
    fn price [lent $x: <T>] -> i32;
;sig Priced

union.tag Stock:
    out;
    few: i32;
    plenty;
;union.tag Stock

rec Item:
    var name: str.utf8;
    var cents: i32[min: 0, max: 100_000];
    var stock: Stock;
;rec Item

fn price [lent $i: Item] -> i32:
    ret $i.cents;
;fn price

fn cheapest <T: Priced> [lent $xs: std::ls[<T>]] -> <T> or fail[empty]:
    if std::ls.len(lend $xs) == 0:
        fail empty;
    ;if
    var $best = 1;
    for $k [from: 2, to: std::ls.len(lend $xs), by: 1]:
        if price(lend $xs[pos: $k]) < price(lend $xs[pos: $best]):
            change $best to $k;
        ;if
    ;for
    ret copy $xs[pos: $best];
;fn cheapest

fn describe [lent $i: Item] -> str.utf8:
    let $dollars := to.stored.str.utf8($i.cents div.floor 100) ++ "." ++ to.stored.str.utf8($i.cents mod.floor 100);
    match $i.stock:
        case out:
            ret $i.name ++ " — sold out";
        ;case out
        case few $n:
            ret $i.name ++ " — only " ++ to.stored.str.utf8($n) ++ " left at $" ++ $dollars;
        ;case few
        case plenty:
            ret $i.name ++ " — $" ++ $dollars;
        ;case plenty
    ;match
;fn describe

fn keep [lent $xs: std::ls[Item], let $test: fn[lent Item] -> bool] -> std::ls[Item]:
    var $out: std::ls[Item] = [];
    for $x in lend $xs:
        if $test(lend $x):
            std::ls.append(lend.mut $out, value: copy $x);
        ;if
    ;for
    ret give $out;
;fn keep

MAIN:
    let $shelf := [
        Item[name = "rope",   cents = 1_250, stock = Stock.plenty],
        Item[name = "helmet", cents = 8_999, stock = Stock.few(2)],
        Item[name = "torch",  cents = 450,   stock = Stock.out]
    ];
    for $i in lend $shelf:
        std::print.stored(describe(lend $i), newln);
    ;for

    let $limit := 5_000;
    let $affordable := keep(lend $shelf, fn [lent $i: Item] -> bool with [lend $limit]:
        ret $i.cents <= $limit;
    ;fn);
    std::print(std::ls.len(lend $affordable), " affordable", newln);

    let $pick := cheapest(lend $affordable) ?? std::exit(code: 0);
    std::print.stored("cheapest: ", $pick.name, newln);

    let $text := std::input.line() ?? str.empty;
    let $budget := to.i32[min: 0, max: 100_000]($text)-
    otherwise:
        case not_a_number:
            std::print.stored("no budget given", newln); std::exit(code: 0);
        ;case not_a_number
        case out_of_range:
            std::print.stored("budget must be 0..100000 cents", newln); std::exit(code: 1);
        ;case out_of_range
    ;otherwise;
    std::print("budget ok: ", $budget, newln);
;MAIN
