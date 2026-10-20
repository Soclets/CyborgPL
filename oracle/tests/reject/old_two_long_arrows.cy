# `<---->` is a spelling of the both-ways binding (owner, 2026-10-06); it checks, then the oracle
# refuses to run assembly.
MAIN {
    let mut 'v': i64 = 1;
    no_check inline.ASM { add x0, x0, #1 } with ['v' <----> x0];
}
