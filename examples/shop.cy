# A shop: an interface, an enum, structs and prices.
interface Priced <T>:
    function price [lent $x: <T>] -> integer.signed.32_bit;
end of interface Priced

union.tagged Stock:
    out;
    few: integer.signed.32_bit;
    plenty;
end of union.tagged Stock

record Item:
    name: string.utf8;
    cents: integer.signed.32_bit[minimum: 0, maximum: 100_000];
    stock: Stock;
end of record Item

function price [lent $i: Item] -> integer.signed.32_bit:
    return $i.cents;
end of function price

function cheapest <T: Priced> [lent $xs: std::list[<T>]] -> <T> or fail[empty]:
    if std::list.length(lend $xs) equal_to 0:
        fail empty;
    end of if
    variable $best := 1;
    for $k [from: 2, to: std::list.length(lend $xs), by: 1]:
        if price(lend $xs[position: $k]) less_than price(lend $xs[position: $best]):
            change $best = $k;
        end of if
    end of for
    return copy $xs[position: $best];
end of function cheapest

function describe [lent $i: Item] -> string.utf8:
    let $dollars := to.stored.string.utf8($i.cents divide.floor 100) concatenate "." concatenate to.stored.string.utf8($i.cents modulo.floor 100);
    match $i.stock:
        case out:
            return $i.name concatenate " — sold out";
        end of case out
        case few $n:
            return $i.name concatenate " — only " concatenate to.stored.string.utf8($n) concatenate " left at $" concatenate $dollars;
        end of case few
        case plenty:
            return $i.name concatenate " — $" concatenate $dollars;
        end of case plenty
    end of match
end of function describe

function keep [lent $xs: std::list[Item], $test: function[lent Item] -> boolean] -> std::list[Item]:
    variable $out: std::list[Item] = [];
    for $x in lend $xs:
        if $test(lend $x):
            std::list.append(lend.mutable $out, value: copy $x);
        end of if
    end of for
    return give $out;
end of function keep

!MAIN:
    let $shelf := [
        Item[name = "rope",   cents = 1_250, stock = Stock.plenty],
        Item[name = "helmet", cents = 8_999, stock = Stock.few(2)],
        Item[name = "torch",  cents = 450,   stock = Stock.out]
    ];
    for $i in lend $shelf:
        std::print.stored(describe(lend $i), newline);
    end of for

    let $limit := 5_000;
    let $affordable := keep(lend $shelf, function [lent $i: Item] -> boolean with [lend $limit]:
        return $i.cents at_most $limit;
    end of function);
    std::print.shortest(std::list.length(lend $affordable), " affordable", newline);

    let $pick := cheapest(lend $affordable) otherwise std::exit(code: 0);
    std::print.stored("cheapest: ", $pick.name, newline);

    let $text := std::input.line() otherwise string.empty;
    let $budget := to.integer.signed.32_bit[minimum: 0, maximum: 100_000]($text) otherwise:
        case not_a_number:
            std::print.stored("no budget given", newline); std::exit(code: 0);
        end of case not_a_number
        case out_of_range:
            std::print.stored("budget must be 0..100000 cents", newline); std::exit(code: 1);
        end of case out_of_range
    end of otherwise;
    std::print.shortest("budget ok: ", $budget, newline);
!end of MAIN
