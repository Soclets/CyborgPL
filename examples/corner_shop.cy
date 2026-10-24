# A day at a corner shop: a longer CyborgPL program, written to be read.
#
# It keeps a small catalog, takes orders written as text, prices them in exact decimals (`d64`,
# so 0.10 plus 0.20 is 0.30), applies discounts that a range type keeps between 0 and 50 percent,
# prints receipts, tracks stock and loyalty points in maps, and ends the day with a report, a
# restocking plan, the day's busy hours, the week's trade and a catalog search. Along the way it
# shows most of CyborgPL's everyday features:
#
#   structs and enums with data, `match`         ownership said aloud: `give`, `copy`, `lend`
#   failures a caller must handle, `otherwise`    generics with an interface, and `Number`
#   functions as values                          a type's own function as an iterator (`Ticket.next`)
#   range types, `i32[min: 0, max: 50]`        counting loops, `for $i [from: 1, to: $n, by: 1]`
#   slices, `from: pos 1, to: pos 3`             consts, at the top and inside a body
#   tasks and channels, `concur`                 `print.useful`, `print.stored`, `print.format`
#
# Nothing here is clever on purpose. A shop's code is read far more often than it is written.

# ---------------------------------------------------------------------------------------------------
# The shop's fixed facts
# ---------------------------------------------------------------------------------------------------

const $shop_name := "Corner Shop";
const $points_per_item := 2;           # loyalty points earned for every item bought
const $low_stock := 3;                 # at or below this, an item goes on the reorder list

# ---------------------------------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------------------------------

enum Aisle {
    fresh,
    bakery,
    pantry,
    household
}

struct Item {
    name: str.utf8,
    aisle: Aisle,
    price: d64
}

# One line of an order: what, and how many.
struct Line {
    name: str.utf8,
    count: i32[min: 1, max: 99]
}

# How a customer pays. A card carries its last four digits, a voucher its value.
enum Payment {
    cash,
    card: str.utf8,
    voucher: d64
}

struct Order {
    customer: str.utf8,
    lines: std::list[Line],
    payment: Payment,
    discount: i32[min: 0, max: 50]
}

# What one order came to, kept for the end-of-day report.
struct Sale {
    ticket: i32,
    customer: str.utf8,
    units: i32,
    total: d64
}

# Receipt numbers come from a counter that hands out the next one each time it is asked.
struct Ticket {
    now: i32,
    last: i32
}

fn Ticket.next [lent.mut $t: Ticket] -> optional[i32] {
    if $t.now equal_to $t.last { return optional.none; }
    change $t.now = $t.now + 1;
    return optional.value($t.now);
}

# ---------------------------------------------------------------------------------------------------
# Small generic helpers
# ---------------------------------------------------------------------------------------------------

interface Number {
    i32
    i64
    f64
}

# The larger of two numbers, for any kind of number.
fn larger <T: Number> [$a: <T>, $b: <T>] -> <T> {
    if $a > $b { return $a; }
    return $b;
}

# The total of a list of numbers.
fn total_of <T: Number> [lent $xs: std::list[<T>]] -> <T> {
    let $sum: <T> = 0;
    for $x in lend $xs {
        change $sum = $sum + $x;
    }
    return $sum;
}

# The list sorted from largest to smallest: a plain insertion sort, easy to check by eye.
fn sorted_down [lent $xs: std::list[i32]] -> std::list[i32] {
    let $out: std::list[i32] = std::list.empty;
    for $x in lend $xs {
        std::list.append(lend.mut $out, value: $x);
        let $i := std::list.length(lend $out);
        while $i > 1 and.then $out[pos: $i] > $out[pos: $i - 1] {
            let $held := $out[pos: $i];
            change $out[pos: $i] = $out[pos: $i - 1];
            change $out[pos: $i - 1] = $held;
            change $i = $i - 1;
        }
    }
    return give $out;
}

# ---------------------------------------------------------------------------------------------------
# The catalog and the stock room
# ---------------------------------------------------------------------------------------------------

fn catalog [] -> std::list[Item] {
    return [
        Item[name = "apple", aisle = Aisle.fresh, price = 0.45],
        Item[name = "banana", aisle = Aisle.fresh, price = 0.30],
        Item[name = "milk", aisle = Aisle.fresh, price = 1.20],
        Item[name = "bread", aisle = Aisle.bakery, price = 2.10],
        Item[name = "croissant", aisle = Aisle.bakery, price = 1.35],
        Item[name = "rice", aisle = Aisle.pantry, price = 3.80],
        Item[name = "beans", aisle = Aisle.pantry, price = 0.95],
        Item[name = "coffee", aisle = Aisle.pantry, price = 6.50],
        Item[name = "soap", aisle = Aisle.household, price = 1.75],
        Item[name = "candles", aisle = Aisle.household, price = 4.20]
    ];
}

fn opening_stock [] -> std::map[str.utf8, i32] {
    let $stock: std::map[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $stock, key: "apple", value: 40);
    std::map.set(lend.mut $stock, key: "banana", value: 30);
    std::map.set(lend.mut $stock, key: "milk", value: 12);
    std::map.set(lend.mut $stock, key: "bread", value: 8);
    std::map.set(lend.mut $stock, key: "croissant", value: 6);
    std::map.set(lend.mut $stock, key: "rice", value: 10);
    std::map.set(lend.mut $stock, key: "beans", value: 24);
    std::map.set(lend.mut $stock, key: "coffee", value: 5);
    std::map.set(lend.mut $stock, key: "soap", value: 9);
    std::map.set(lend.mut $stock, key: "candles", value: 4);
    return give $stock;
}

# The catalog entry for a name, or a failure the caller must handle.
fn find_item [lent $items: std::list[Item], lent $name: str.utf8] -> Item or fail[unknown_item] {
    for $it in lend $items {
        if $it.name equal_to $name { return copy $it; }
    }
    fail unknown_item;
}

fn aisle_name [$a: Aisle] -> str.utf8 {
    match $a {
        fresh     { return "fresh"; }
        bakery    { return "bakery"; }
        pantry    { return "pantry"; }
        household { return "household"; }
    }
}

# ---------------------------------------------------------------------------------------------------
# Reading orders written as text: "apple 3, bread 1, coffee 2"
# ---------------------------------------------------------------------------------------------------

# One "name count" piece into a line; a count that is not a number, or not 1 to 99, fails.
fn parse_line [lent $piece: str.utf8] -> Line or fail[bad_line: str.utf8] {
    let $words := std::str.split(lend $piece, at: " ");
    if std::list.length(lend $words) not_equal 2 { fail bad_line[copy $piece]; }
    let $name := copy $words[pos: 1];
    let $count: i32[min: 1, max: 99] = to.i32[min: 1, max: 99]($words[pos: 2]) otherwise {
        not_a_number { fail bad_line[copy $piece]; }
        out_of_range { fail bad_line[copy $piece]; }
    };
    return Line[name = give $name, count = $count];
}

fn parse_order [lent $text: str.utf8] -> std::list[Line] or fail[bad_line: str.utf8] {
    let $lines: std::list[Line] = std::list.empty;
    let $pieces := std::str.split(lend $text, at: ", ");
    for $piece in lend $pieces {
        let $line := parse_line(lend $piece) otherwise fail;
        std::list.append(lend.mut $lines, value: give $line);
    }
    return give $lines;
}

# ---------------------------------------------------------------------------------------------------
# Pricing
# ---------------------------------------------------------------------------------------------------

# A pricing rule takes a line's price before any rule and gives what the customer pays for it.
fn no_rule [$line_total: d64] -> d64 { return $line_total; }

# Bakery goods are 20% off after six in the evening; the shop runs this rule all day here.
fn evening_bakery [$line_total: d64] -> d64 { return $line_total x 0.80; }

fn rule_for [$a: Aisle] -> fn[d64] -> d64 {
    match $a {
        bakery { return evening_bakery; }
        fresh { return no_rule; }
        pantry { return no_rule; }
        household { return no_rule; }
    }
}

# A whole-order discount, which the range type keeps between 0 and 50 percent.
fn apply_discount [$amount: d64, $percent: i32[min: 0, max: 50]] -> d64 {
    let $pct: d64 = to.d64($percent);
    let $off: d64 = ($amount x $pct) div.near 100;
    return $amount - $off;
}

# ---------------------------------------------------------------------------------------------------
# Taking one order
# ---------------------------------------------------------------------------------------------------

# Every line of an order, checked before anything happens: an item the shop sells, and enough of it once the
# lines before it are counted. A failed order then changes no stock and uses no receipt number (multifinity's
# review, 2026-10-22: it used to fail halfway, with stock already taken and #105 spent).
fn check_order [
    lent $items: std::list[Item],
    lent $stock: std::map[str.utf8, i32],
    lent $order: Order
] -> fail[out_of_stock: str.utf8, unknown_item: str.utf8] {
    let mut $left := copy $stock;
    for $line in lend $order.lines {
        let $item := find_item(lend $items, lend $line.name) otherwise {
            unknown_item { fail unknown_item[copy $line.name]; }
        };
        let $have := std::map.get(lend $left, key: copy $line.name) otherwise 0;
        if $have < $line.count { fail out_of_stock[copy $line.name]; }
        std::map.set(lend.mut $left, key: copy $line.name, value: $have - $line.count);
    }
}

fn take_order [
    lent $items: std::list[Item],
    lent.mut $stock: std::map[str.utf8, i32],
    lent $order: Order,
    $ticket: i32
] -> Sale or fail[out_of_stock: str.utf8, unknown_item: str.utf8] {
    std::print.useful("--- receipt #", $ticket, " for ", $order.customer, " ---", newln);
    let $subtotal: d64 = 0;
    let $units := 0;
    for $line in lend $order.lines {
        let $item := find_item(lend $items, lend $line.name) otherwise {
            unknown_item { fail unknown_item[copy $line.name]; }
        };
        let $have := std::map.get(lend $stock, key: copy $line.name) otherwise 0;
        if $have < $line.count { fail out_of_stock[copy $line.name]; }
        std::map.set(lend.mut $stock, key: copy $line.name, value: $have - $line.count);
        let $rule := rule_for(copy $item.aisle);
        let $count: d64 = to.d64($line.count);
        let $line_total := $rule($item.price x $count);
        change $subtotal = $subtotal + $line_total;
        change $units = $units + $line.count;
        std::print.format("  {:<10} x{:>2}  {:>8.2}\n", $line.name, $line.count, $line_total);
    }
    let $total := apply_discount($subtotal, $order.discount);
    if $order.discount > 0 {
        std::print.format("  discount {}%\n", $order.discount);
    }
    std::print.format("  {:<14} {:>8.2}\n", "total", $total);
    std::print.useful("  paid by ", payment_text(lend $order.payment, $total), newln);
    return Sale[ticket = $ticket, customer = copy $order.customer, units = $units, total = $total];
}

fn payment_text [lent $p: Payment, $total: d64] -> str.utf8 {
    match $p {
        cash { return "cash"; }
        card $last4 { return "card ending " concat $last4; }
        voucher $worth {
            if $worth at_least $total { return "voucher, nothing left to pay"; }
            return std::format("voucher, then {:.2} in cash", $total - $worth);
        }
    }
}

# ---------------------------------------------------------------------------------------------------
# The day's orders
# ---------------------------------------------------------------------------------------------------

fn todays_orders [] -> std::list[Order] {
    let $orders: std::list[Order] = std::list.empty;
    let $texts := [
        "apple 6, bread 1, milk 2",
        "coffee 2, croissant 4",
        "rice 1, beans 3, soap 2, candles 1",
        "banana 12, milk 1",
        "coffee 4",
        "pineapple 1",
        "apple two",
        "candles 2, bread 2, croissant 2"
    ];
    let $customers := ["Ann", "Bo", "Cy", "Dee", "Eli", "Fay", "Gus", "Hal"];
    for $i [from: 1, to: std::list.length(lend $texts), by: 1] {
        let $lines := parse_order(lend $texts[pos: $i]) otherwise {
            bad_line $what {
                std::print.useful("(order for ", $customers[pos: $i], " skipped: cannot read \"", $what, "\")", newln);
                continue;
            }
        };
        let $payment := choose_payment($i);
        let $discount: i32[min: 0, max: 50] = 0;
        if $i equal_to 3 { change $discount = 10; }
        if $i equal_to 8 { change $discount = 25; }
        std::list.append(lend.mut $orders, value: Order[customer = copy $customers[pos: $i], lines = give $lines, payment = give $payment, discount = $discount]);
    }
    return give $orders;
}

fn choose_payment [$i: i32] -> Payment {
    if ($i mod.floor 3) equal_to 0 { return Payment.voucher(5.00); }
    if ($i mod.floor 2) equal_to 0 { return Payment.card("4242"); }
    return Payment.cash;
}

# ---------------------------------------------------------------------------------------------------
# Loyalty points
# ---------------------------------------------------------------------------------------------------

fn add_points [lent.mut $points: std::map[str.utf8, i32], lent $sale: Sale] {
    let $earned := $sale.units x $points_per_item;
    let $before := std::map.get(lend $points, key: copy $sale.customer) otherwise 0;
    std::map.set(lend.mut $points, key: copy $sale.customer, value: $before + $earned);
}

# ---------------------------------------------------------------------------------------------------
# The end-of-day report
# ---------------------------------------------------------------------------------------------------

fn reorder_list [lent $items: std::list[Item], lent $stock: std::map[str.utf8, i32]] -> std::list[str.utf8] {
    let $out: std::list[str.utf8] = std::list.empty;
    for $it in lend $items {
        let $left := std::map.get(lend $stock, key: copy $it.name) otherwise 0;
        if $left at_most $low_stock {
            std::list.append(lend.mut $out, value: copy $it.name concat " (" concat to.stored.str.utf8($left) concat " left)");
        }
    }
    return give $out;
}

fn takings_by_aisle [lent $items: std::list[Item], lent $opening: std::map[str.utf8, i32], lent $closing: std::map[str.utf8, i32], $a: Aisle] -> d64 {
    let $sum: d64 = 0;
    for $it in lend $items {
        if $it.aisle equal_to $a {
            let $had := std::map.get(lend $opening, key: copy $it.name) otherwise 0;
            let $has := std::map.get(lend $closing, key: copy $it.name) otherwise 0;
            let $sold := $had - $has;
            change $sum = $sum + $it.price x to.d64($sold);
        }
    }
    return $sum;
}

fn report [lent $sales: std::list[Sale], lent $items: std::list[Item], lent $opening: std::map[str.utf8, i32], lent $stock: std::map[str.utf8, i32], lent $points: std::map[str.utf8, i32]] {
    std::print.useful(newln, "=== ", $shop_name, ", end of day ===", newln);

    # Orders and units
    let $units: std::list[i32] = std::list.empty;
    let $takings: d64 = 0;
    for $s in lend $sales {
        std::list.append(lend.mut $units, value: $s.units);
        change $takings = $takings + $s.total;
    }
    std::print.useful("orders served: ", std::list.length(lend $sales), ", units sold: ", total_of(lend $units), newln);
    std::print.format("takings: {:.2}\n", $takings);

    # The biggest baskets, by units: the first three of the list sorted largest first
    let $ranked := sorted_down(lend $units);
    if std::list.length(lend $ranked) at_least 3 {
        let $top := std::list.slice[lend $ranked, from: pos 1, to: pos 3];
        std::print.useful("biggest baskets (units): ", $top, newln);
    }

    # The best sale of the day
    let $best_total: d64 = 0;
    let $best_ticket := 0;
    let $best_customer := "nobody";
    for $s in lend $sales {
        if $s.total > $best_total {
            change $best_total = $s.total;
            change $best_ticket = $s.ticket;
            change $best_customer = copy $s.customer;
        }
    }
    std::print.format("best sale: #{}, {}, {:.2}\n", $best_ticket, $best_customer, $best_total);

    # Takings by aisle, counted from what left the shelves (at list price, before discounts)
    std::print.useful("by aisle, at list price:", newln);
    let $aisles := [Aisle.fresh, Aisle.bakery, Aisle.pantry, Aisle.household];
    for $a in lend $aisles {
        std::print.format("  {:<10} {:>8.2}\n", aisle_name(copy $a), takings_by_aisle(lend $items, lend $opening, lend $stock, copy $a));
    }

    # Reorders
    let $reorder := reorder_list(lend $items, lend $stock);
    if std::list.length(lend $reorder) equal_to 0 {
        std::print.useful("nothing to reorder", newln);
    } else {
        std::print.useful("reorder: ", std::str.join(lend $reorder, between: ", "), newln);
    }

    # Loyalty points, for the customers who were served
    std::print.useful("loyalty points:", newln);
    for $s in lend $sales {
        let $p := std::map.get(lend $points, key: copy $s.customer) otherwise 0;
        std::print.format("  {:<5} {:>3}\n", $s.customer, $p);
    }
}

# ---------------------------------------------------------------------------------------------------
# Counting at the same time: each aisle's shelf count is tallied by its own task, which reports back
# over a channel. The tasks run alongside each other; MAIN waits for every answer.
# ---------------------------------------------------------------------------------------------------

fn shelf_count [lent $items: std::list[Item], lent $stock: std::map[str.utf8, i32], $a: Aisle] -> i32 {
    let $n := 0;
    for $it in lend $items {
        if $it.aisle equal_to $a {
            let $on_shelf := std::map.get(lend $stock, key: copy $it.name) otherwise 0;
            change $n = $n + $on_shelf;
        }
    }
    return $n;
}

fn count_shelves [lent $items: std::list[Item], lent $stock: std::map[str.utf8, i32]] {
    let $fresh_ch := channel[i32];
    let $bakery_ch := channel[i32];
    let $pantry_ch := channel[i32];
    let $household_ch := channel[i32];
    concur with [lend $fresh_ch, lend $items, lend $stock] { channel.send(lend $fresh_ch, value: shelf_count(lend $items, lend $stock, Aisle.fresh)); }
    concur with [lend $bakery_ch, lend $items, lend $stock] { channel.send(lend $bakery_ch, value: shelf_count(lend $items, lend $stock, Aisle.bakery)); }
    concur with [lend $pantry_ch, lend $items, lend $stock] { channel.send(lend $pantry_ch, value: shelf_count(lend $items, lend $stock, Aisle.pantry)); }
    concur with [lend $household_ch, lend $items, lend $stock] { channel.send(lend $household_ch, value: shelf_count(lend $items, lend $stock, Aisle.household)); }
    let $f := channel.take(lend $fresh_ch) otherwise 0;
    let $b := channel.take(lend $bakery_ch) otherwise 0;
    let $p := channel.take(lend $pantry_ch) otherwise 0;
    let $h := channel.take(lend $household_ch) otherwise 0;
    std::print.useful("on the shelves tonight: fresh ", $f, ", bakery ", $b, ", pantry ", $p, ", household ", $h, newln);
}

# ---------------------------------------------------------------------------------------------------
# Looking things up: finding catalog items by what their name starts with or contains
# ---------------------------------------------------------------------------------------------------

fn names_starting [lent $items: std::list[Item], lent $start: str.utf8] -> std::list[str.utf8] {
    let $out: std::list[str.utf8] = std::list.empty;
    for $it in lend $items {
        if std::str.starts_with(lend $it.name, part: lend $start) {
            std::list.append(lend.mut $out, value: copy $it.name);
        }
    }
    return give $out;
}

fn names_containing [lent $items: std::list[Item], lent $part: str.utf8] -> std::list[str.utf8] {
    let $out: std::list[str.utf8] = std::list.empty;
    for $it in lend $items {
        if std::str.contains(lend $it.name, part: lend $part) {
            std::list.append(lend.mut $out, value: copy $it.name);
        }
    }
    return give $out;
}

fn show_search [lent $items: std::list[Item]] {
    std::print.useful(newln, "searching the catalog:", newln);
    let $starts := ["b", "c", "z"];
    for $s in lend $starts {
        let $found := names_starting(lend $items, lend $s);
        if std::list.length(lend $found) equal_to 0 {
            std::print.useful("  nothing starts with \"", $s, "\"", newln);
        } else {
            std::print.useful("  starting with \"", $s, "\": ", std::str.join(lend $found, between: ", "), newln);
        }
    }
    let $has_an := names_containing(lend $items, "an");
    std::print.useful("  containing \"an\": ", std::str.join(lend $has_an, between: ", "), newln);
    let $sign := $shop_name;                     # a const is a fresh value each use, so it is named to lend
    let $loud := std::str.upper.full(lend $sign);
    std::print.useful("  the sign over the door says ", $loud, newln);
}

# ---------------------------------------------------------------------------------------------------
# Restocking: what to order so every shelf is back to its target, in whole cases
# ---------------------------------------------------------------------------------------------------

struct Restock {
    name: str.utf8,
    cases: i32,
    cost: d64
}

# How many items one case holds, by aisle.
fn case_size [$a: Aisle] -> i32 {
    let $size := match $a {
        fresh { is 10; }
        bakery { is 6; }
        pantry { is 12; }
        household { is 4; }
    };
    return $size;
}

# The supplier charges 60% of the shop's price for each item in a case.
fn case_cost [$price: d64, $size: i32] -> d64 {
    let $each: d64 = $price x 0.60;
    let $n: d64 = to.d64($size);
    return $each x $n;
}

fn plan_restock [lent $items: std::list[Item], lent $opening: std::map[str.utf8, i32], lent $stock: std::map[str.utf8, i32]] -> std::list[Restock] {
    let $plan: std::list[Restock] = std::list.empty;
    for $it in lend $items {
        let $target := std::map.get(lend $opening, key: copy $it.name) otherwise 0;
        let $left := std::map.get(lend $stock, key: copy $it.name) otherwise 0;
        let $missing := $target - $left;
        if $missing > 0 {
            let $size := case_size(copy $it.aisle);
            # whole cases only: round the count of cases up
            let $cases := ($missing + $size - 1) div.floor $size;
            let $one_case := case_cost($it.price, $size);
            let $n: d64 = to.d64($cases);
            std::list.append(lend.mut $plan, value: Restock[name = copy $it.name, cases = $cases, cost = $one_case x $n]);
        }
    }
    return give $plan;
}

fn show_restock [lent $plan: std::list[Restock]] {
    std::print.useful(newln, "tomorrow's delivery:", newln);
    let $bill: d64 = 0;
    for $r in lend $plan {
        std::print.format("  {:<10} {:>2} case(s)  {:>7.2}\n", $r.name, $r.cases, $r.cost);
        change $bill = $bill + $r.cost;
    }
    std::print.format("  {:<22} {:>7.2}\n", "to the supplier", $bill);
}

# ---------------------------------------------------------------------------------------------------
# A week of trade: seven days of made-up but repeatable numbers, averaged and compared
# ---------------------------------------------------------------------------------------------------

# Customers on day 'd' (1 is Monday): busier towards the weekend, quieter on Sunday.
fn customers_on [$d: i32[min: 1, max: 7]] -> i32 {
    if $d equal_to 7 { return 18; }
    return 20 + $d x 4;
}

fn show_week [] {
    const $days := 7;
    std::print.useful(newln, "this week:", newln);
    let $counts: std::list[i32] = std::list.empty;
    for $d [from: 1, to: $days, by: 1] {
        let $n := customers_on($d);
        std::list.append(lend.mut $counts, value: $n);
    }
    let $names := ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let $busiest := 0;
    for $i, $n in lend $counts {
        std::print.format("  {} {:>3}  ", $names[pos: $i], $n);
        for $k [from: 0, before: $n, by: 4] {
            std::print.useful("#");
        }
        std::print.useful(newln);
        change $busiest = larger($busiest, $n);
    }
    let $sum := total_of(lend $counts);
    let $average: f64 = to.f64($sum) div.near to.f64($days);
    std::print.format("  {} customers, {:.1} a day, {} on the busiest\n", $sum, $average, $busiest);
    let $weekend := std::list.slice[lend $counts, from: pos 6, to: pos 7];
    let $weekdays := std::list.slice[lend $counts, from: pos 1, before: pos 6];
    std::print.useful("  weekdays ", total_of(lend $weekdays), ", weekend ", total_of(lend $weekend), newln);
}

# ---------------------------------------------------------------------------------------------------
# Sales by the hour: a bar for each opening hour, counted with `before`
# ---------------------------------------------------------------------------------------------------

fn show_hours [$open: i32, $close: i32] {
    std::print.useful(newln, "customers by the hour:", newln);
    let $peak := 0;
    let $peak_hour := $open;
    for $h [from: $open, before: $close, by: 1] {
        # a lunch rush at noon and an evening rush at five, quiet otherwise
        let $n := 2;
        if $h equal_to 12 or $h equal_to 17 { change $n = 9; }
        if $h equal_to 13 or $h equal_to 18 { change $n = 6; }
        std::print.format("  {:>2}:00 ", $h);
        let $bar := std::list.fill[value: "*", count: $n];
        std::print.useful(std::str.join(lend $bar, between: str.empty), newln);
        if $n > $peak {
            change $peak = $n;
            change $peak_hour = $h;
        }
    }
    std::print.useful("  busiest from ", $peak_hour, ":00", newln);
}

# ---------------------------------------------------------------------------------------------------
# The day
# ---------------------------------------------------------------------------------------------------

MAIN {
    const $opening_hour := 8;
    const $closing_hour := 20;
    std::print.useful($shop_name, ", open ", $opening_hour, ":00 to ", $closing_hour, ":00", newln);

    let $items := catalog();
    let $stock := opening_stock();
    let $opening := copy $stock;
    let $points: std::map[str.utf8, i32] = std::map.empty;
    let $sales: std::list[Sale] = std::list.empty;

    # The price list, three columns of a table
    std::print.useful(newln, "today's prices:", newln);
    for $it in lend $items {
        std::print.format("  {:<10} {:<10} {:>6.2}\n", $it.name, aisle_name(copy $it.aisle), $it.price);
    }
    std::print.useful(newln);

    # Hours the shop is open, counted the new way: from opening, stopping before closing
    let $hours := 0;
    for $h [from: $opening_hour, before: $closing_hour, by: 1] {
        change $hours = $hours + 1;
    }
    std::print.useful("open for ", $hours, " hours", newln, newln);

    # Take every order, each with the next receipt number
    let $orders := todays_orders();
    std::print.useful(newln);
    let $tickets := Ticket[now = 100, last = 199];
    for $order in lend $orders {
        check_order(lend $items, lend $stock, lend $order) otherwise {
            out_of_stock $what {
                std::print.useful("  sorry, ", $order.customer, ": not enough ", $what, newln, newln);
                continue;
            }
            unknown_item $what {
                std::print.useful("  sorry, ", $order.customer, ": we do not sell ", $what, newln, newln);
                continue;
            }
        };
        let $n := Ticket.next(lend.mut $tickets);
        let $ticket := match $n {
            value $number { is $number; }
            none { is 0; }
        };
        let $sale := take_order(lend $items, lend.mut $stock, lend $order, $ticket) otherwise {
            out_of_stock $what {
                std::print.useful("  sorry, ", $order.customer, ": not enough ", $what, newln, newln);
                continue;
            }
            unknown_item $what {
                std::print.useful("  sorry, ", $order.customer, ": we do not sell ", $what, newln, newln);
                continue;
            }
        };
        add_points(lend.mut $points, lend $sale);
        std::list.append(lend.mut $sales, value: give $sale);
        std::print.useful(newln);
    }

    report(lend $sales, lend $items, lend $opening, lend $stock, lend $points);
    count_shelves(lend $items, lend $stock);
    let $plan := plan_restock(lend $items, lend $opening, lend $stock);
    show_restock(lend $plan);
    show_hours($opening_hour, $closing_hour);
    show_week();
    show_search(lend $items);

    # Exact decimals: the reason money is `d64` and not `f64`
    let $dime: d64 = 0.10;
    let $cents_sum: d64 = $dime + 0.20;
    let $float_sum: f64 = 0.1 + 0.2;
    std::print.useful(newln, "0.10 plus 0.20, stored exactly as money: ");
    std::print.stored($cents_sum, newln);
    std::print.useful("0.1 plus 0.2, stored as a binary float: ");
    std::print.stored($float_sum, newln);
    std::print.useful("good night from the ", $shop_name, newln);
}
