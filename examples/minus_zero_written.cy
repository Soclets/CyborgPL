# A written -0.0 is 0: no bin is -0 except through the .ieee forms (owner, 2026-10-07: "a").
# check claude, 2026-10-19 (corpus 3878ea56): the oracle kept the sign of a written -0.0.
layout.C union Un { i: int32, u: uint32, f: bin32 }

MAIN {
    let 'a': bin32 = -0.0;
    let 'b' := -0.0: bin64;
    let mut 'un' := Un[i = 1];
    change 'un'.f = -0.0;
    no_check let 'u' := 'un'.u;
    std::print.stored['a', " ", 'b', " ", 'u', newln];
}
