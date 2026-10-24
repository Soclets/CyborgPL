# A `match` alone in a case follows the outer match (owner, 2026-09-29: a):
# in a statement match it runs; in a value match it is the case's value
# (examples/nested_match.cpls as an exit code: "mail" is 2, the price 9 →
# 2 x 10 + 9 = 29).
extern.C fn exit [$code: i32];
enum Order { cancelled, paid }
enum Ship { pickup, post }
MAIN {
    let $o := Order.paid;
    let $s := Ship.post;
    let mut $went: i32 = 0;
    match $o {
        cancelled { }
        paid { match $s { pickup { change $went = 1; } post { change $went = 2; } } }
    }
    let $price := match $o { cancelled { is 0; } paid { is match $s { pickup { is 5; } post { is 9; } }; } };
    no_check my.exit($went times 10 plus $price);
}
