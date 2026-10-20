# Finding 2026-10-19 (check run, seeds 10027, 10101, 10265, the C episode's union; cut down by hand): a
# written -0.0 stored in a bin keeps IEEE's sign in the oracle. DECISIONS 2026-10-07, "no -0 anywhere":
# no bin or brain16 result is -0 except through the .ieee forms, and a plain -'x' never gives one. The
# oracle's print.stored of `let 'a': f32 = -0.0` shows -0, and a union read of its bits gives
# 2147483648 (the sign bit); -'z' of a stored 0.0 is 0 as it should be. kayo gives 0, 0, 0, 0.
layout.C union Un { i: i32, u: u32, f: f32 }
MAIN {
    let 'a': f32 = -0.0;
    std::print.stored['a', newln];
    let 'z': f64 = 0.0;
    let 'b': f64 = -'z';
    std::print.stored['b', newln];
    let mut 'un' := Un[i = 5];
    change 'un'.f = -0.0;
    no_check let 'bits' := 'un'.u;
    std::print.useful['bits', newln];
    change 'un'.f = 'a';
    no_check let 'bits2' := 'un'.u;
    std::print.useful['bits2', newln];
}
