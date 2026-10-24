# Finding 2026-09-29 (check run, seeds 2875 and 2937, cut down by hand): in a #strict file, 16 mod 'p'
# behind `if 'p' !== 0` (the guardrail strict asks for, written in the same function) is refused as
# "the divisor can be zero" once something calls f with 5. Uncalled, or called as f[0], it is proven;
# outside #strict it is proven. The same with `'p' !== 0 and.then …` and `'p' == 0 or.else …`.
fn f ['p': i32] -> bool {
    if 'p' not_equal 0 {
        return (16 mod.floor 'p') at_least 3;
    }
    return false;
}
MAIN {
    let immut 'x': bool = f(5);
}
