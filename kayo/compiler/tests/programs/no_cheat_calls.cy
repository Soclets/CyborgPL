# `no_cheat` in front of a statement covers the calls it makes, all the way
# down (owner, 2026-10-18: "(b)"): `fib[20]` really runs, in a copy of `fib`
# whose parameters LLVM cannot see through, so its answer is never folded
# in and its look-alike calls are not shared. 6765 mod 256 is 109.
%extern.C fn exit [$code: i32];

fn fib [$n: i64[min: 0, max: 92]] -> i64 {
    if $n < 2 { return $n; }
    return fib($n - 1) + fib($n - 2);
}

MAIN {
    no_cheat let $f := fib(20);
    no_check my.exit(to.i32($f mod.floor 256));
}
