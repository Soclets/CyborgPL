# A shop: an interface, an enum, structs and prices.
interface Priced <T>:
    fn price [lent $x: <T>] -> int.sign.32;
end of interface Priced

union.tag Stock:
    out;
    few: int.sign.32;
    plenty;
end of union.tag Stock

rec Item:
    name: str.utf8;
    cents: int.sign.32[min: 0, max: 100_000];
    stock: Stock;
end of rec Item

fn price [lent $i: Item] -> int.sign.32:
    ret $i.cents;
end of fn price

fn cheapest <T: Priced> [lent $xs: std::ls[<T>]] -> <T> or fail[empty]:
    if std::ls.len(lend $xs) eq 0:
        fail empty;
    end of if
    var $best := 1;
    for $k [from: 2, to: std::ls.len(lend $xs), by: 1]:
        if price(lend $xs[pos: $k]) less_than price(lend $xs[pos: $best]):
            change $best = $k;
        end of if
    end of for
    ret copy $xs[pos: $best];
end of fn cheapest

fn describe [lent $i: Item] -> str.utf8:
    let $dollars := to.stored.str.utf8($i.cents div.floor 100) concat "." concat to.stored.str.utf8($i.cents mod.floor 100);
    match $i.stock:
        case out:
            ret $i.name concat " — sold out";
        end of case out
        case few $n:
            ret $i.name concat " — only " concat to.stored.str.utf8($n) concat " left at $" concat $dollars;
        end of case few
        case plenty:
            ret $i.name concat " — $" concat $dollars;
        end of case plenty
    end of match
end of fn describe

fn keep [lent $xs: std::ls[Item], $test: fn[lent Item] -> bool] -> std::ls[Item]:
    var $out: std::ls[Item] = [];
    for $x in lend $xs:
        if $test(lend $x):
            std::ls.append(lend.mut $out, value: copy $x);
        end of if
    end of for
    ret give $out;
end of fn keep

!MAIN:
    let $shelf := [
        Item[name = "rope",   cents = 1_250, stock = Stock.plenty],
        Item[name = "helmet", cents = 8_999, stock = Stock.few(2)],
        Item[name = "torch",  cents = 450,   stock = Stock.out]
    ];
    for $i in lend $shelf:
        std::print.stored(describe(lend $i), newln);
    end of for

    let $limit := 5_000;
    let $affordable := keep(lend $shelf, fn [lent $i: Item] -> bool with [lend $limit]:
        ret $i.cents at_most $limit;
    end of fn);
    std::print.shortest(std::ls.len(lend $affordable), " affordable", newln);

    let $pick := cheapest(lend $affordable) otherwise std::exit(code: 0);
    std::print.stored("cheapest: ", $pick.name, newln);

    let $text := std::input.line() otherwise str.empty;
    let $budget := to.int.sign.32[min: 0, max: 100_000]($text)-
    otherwise:
        case not_a_number:
            std::print.stored("no budget given", newln); std::exit(code: 0);
        end of case not_a_number
        case out_of_range:
            std::print.stored("budget must be 0..100000 cents", newln); std::exit(code: 1);
        end of case out_of_range
    end of otherwise;
    std::print.shortest("budget ok: ", $budget, newln);
!end of MAIN
