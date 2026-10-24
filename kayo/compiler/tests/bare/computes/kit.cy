# Runs to its end on a bare chip (owner, 2026-10-09: `aarch64-none`), so
# the CPU sleeps in the entry. A wrong answer would take a u8 below 0, a
# failed check, and freeze it on a trap instead: there is no exit code to
# carry the answer, so the trap is the test.
fn fib [$n: i64] -> i64 {
    if $n < 2 { return $n; }
    return fib($n minus 1) plus fib($n minus 2);
}

struct Counter {
    now: i32,
    stop: i32
}

fn Counter.next [lent.mut $c: Counter] -> optional[i32] {
    if $c.now equal_to $c.stop { return optional.none; }
    change $c.now = $c.now plus 1;
    return optional.value($c.now);
}

MAIN {
    let mut $wrong: u8 = 0;
    if fib(24) not_equal 46368 { change $wrong = $wrong minus 1; }
    let $half: f64 = 3.0 times 0.5;
    if $half not_equal 1.5 { change $wrong = $wrong minus 1; }
    let mut $sum: i32 = 0;
    let mut $c := Counter[now = 0, stop = 10];
    for $x in lend.mut $c {
        change $sum = $sum plus $x;
    }
    if $sum not_equal 55 { change $wrong = $wrong minus 1; }
}
