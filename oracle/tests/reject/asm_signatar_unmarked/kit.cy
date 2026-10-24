# assembly is a `no_check` site, the base's since IEG was dissolved (owner, 2026-10-12): an unmarked `inline.ASM`
# block is refused.
MAIN {
    let mut $v: i64 = 1;
    inline.ASM { add x0, x0, #1 } with [$v <-> x0];
}
