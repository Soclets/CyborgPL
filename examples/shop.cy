# A shop: an interface, an enum, structs and prices.
interface Priced <T> {
    fn price [lent 'x': <T>] -> i32;
}

enum Stock {
    out,
    few: i32,
    plenty
}

struct Item {
    name: str.utf8,
    cents: i32[min: 0, max: 100_000],
    stock: Stock
}

fn price [lent 'i': Item] -> i32 { return 'i'.cents; }

fn cheapest <T: Priced> [lent 'xs': std::list[<T>]] -> <T> or fail[empty] {
    if std::list.length[lend 'xs'] equal_to 0 { fail empty; }
    let 'best' := 1;
    for 'k' [from: 2, to: std::list.length[lend 'xs'], by: 1] {
        if price[lend 'xs'[pos: 'k']] < price[lend 'xs'[pos: 'best']] { change 'best' = 'k'; }
    }
    return copy 'xs'[pos: 'best'];
}

fn describe [lent 'i': Item] -> str.utf8 {
    let 'dollars' := to.stored.str.utf8['i'.cents div.floor 100] concat "." concat to.stored.str.utf8['i'.cents mod.floor 100];
    match 'i'.stock {
        out      { return 'i'.name concat " — sold out"; }
        few 'n'  { return 'i'.name concat " — only " concat to.stored.str.utf8['n'] concat " left at $" concat 'dollars'; }
        plenty   { return 'i'.name concat " — $" concat 'dollars'; }
    }
}

fn keep [lent 'xs': std::list[Item], 'test': fn[lent Item] -> bool] -> std::list[Item] {
    let 'out': std::list[Item] = [];
    for 'x' in lend 'xs' { if 'test'[lend 'x'] { std::list.append[lend.mut 'out', value: copy 'x']; } }
    return give 'out';
}

MAIN {
    let 'shelf' := [
        Item[name = "rope",   cents = 1_250, stock = Stock.plenty],
        Item[name = "helmet", cents = 8_999, stock = Stock.few[2]],
        Item[name = "torch",  cents = 450,   stock = Stock.out]
    ];
    for 'i' in lend 'shelf' { std::print.stored[describe[lend 'i'], newln]; }

    let 'limit' := 5_000;
    let 'affordable' := keep[lend 'shelf', fn [lent 'i': Item] -> bool with [lend 'limit'] { return 'i'.cents at_most 'limit'; }];
    std::print.useful[std::list.length[lend 'affordable'], " affordable", newln];

    let 'pick' := cheapest[lend 'affordable'] otherwise std::exit[0];
    std::print.stored["cheapest: ", 'pick'.name, newln];

    let 'text' := std::input.line[] otherwise str.empty;
    let 'budget' := to.i32[min: 0, max: 100_000]['text'] otherwise {
        not_a_number { std::print.stored["no budget given", newln]; std::exit[0]; }
        out_of_range { std::print.stored["budget must be 0..100000 cents", newln]; std::exit[1]; }
    };
    std::print.useful["budget ok: ", 'budget', newln];
}
