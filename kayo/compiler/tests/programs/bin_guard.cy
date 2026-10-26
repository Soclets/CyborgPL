# A guardrail on a bin, and the `check` that says it is enough. Without the
# two `if`s the store is refused; with them it is proven and no check is
# emitted. The program exits 7 either way — what is being tested is that it
# builds at all, which only happens when the guardrail is believed.
%extern.C fn exit [$code: i32];
%extern.C fn getpid [] -> i32;

MAIN {
    no_check let $n := getpid();
    let $x: f32 = to.f32($n);
    if $x > 65_504.0 {
    } else if $x < -65_504.0 {
    } else {
        check let $y: f16 = $x;
    }
    no_check my.exit(7);
}
