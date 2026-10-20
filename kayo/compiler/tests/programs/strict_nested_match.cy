# In a #strict file a bare nested `match` is a statement, and a nested match
# that is the value says `giveS` (owner, 2026-09-29): "mail" is 2, the price
# 9 → 29 (examples/strict_nested_match.cpls as an exit code).
extern.C fn exit ['code': i32];
enum Order { cancelled, paid }
enum Ship { pickup, post }
fn handle ['o': Order, 's': Ship] -> i32 {
    let mut 'went': i32 = 0;
    match 'o' {
        cancelled { }
        paid {
            match 's' {
                pickup { change 'went' = 1; }
                post { change 'went' = 2; }
            }
        }
    }
    return 'went';
}
fn price ['o': Order, 's': Ship] -> i32 {
    let immut 'p': i32 = match 'o' {
        cancelled { is 0; }
        paid { is match 's' { pickup { is 5; } post { is 9; } }; }
    };
    return 'p';
}
MAIN {
    let immut 'w': i32 = handle[Order.paid, Ship.post];
    let immut 'n': i32 = price[Order.paid, Ship.post];
    no_check my.exit['w' times 10 plus 'n'];
}
