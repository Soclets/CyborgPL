# `no_cheat` covers everything the statement runs, however it is called
# (owner, 2026-10-19: "(a)"): a function held in a variable (fib[20], 6765),
# a literal function borrowing a value (fib[10] + 3, 58), and the body of a
# task started inside (fib[15], 610), each a `no_cheat` copy. 6765 + 58 +
# 610 = 7433, and 7433 mod 256 is 9.
extern.C fn exit [$code: i32];

fn fib [$n: i64[min: 0, max: 92]] -> i64 {
    if $n < 2 { return $n; }
    return fib($n - 1) + fib($n - 2);
}

MAIN {
    let $f := fib;
    let mut $a: i64 = 0;
    no_cheat change $a = $f(20);
    let $k: i64 = 3;
    let $g := fn [$n: i64[min: 0, max: 92]] -> i64 with [lend $k] { return fib($n) plus $k; };
    let mut $b: i64 = 0;
    no_cheat change $b = $g(10);
    let $out := channel[i64];
    let mut $c: i64 = 0;
    no_cheat {
        concur with [lend $out] { channel.send(lend $out, value: fib(15)); }
        change $c = channel.take(lend $out) otherwise 0;
    }
    no_check my.exit(to.i32(($a plus $b plus $c) mod.floor 256));
}
