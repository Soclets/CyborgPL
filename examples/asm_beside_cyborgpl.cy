# Inline ARM64 assembly beside the same work in CyborgPL (ffi.md, `inline.ASM`). Each `with [ … ]` binds
# variables to registers: `->` copies a value in before the block, `<-` stores a register into a new
# variable after it, and `<->` does both, keeping the variable's type. Assembly is always a
# `no_check` site: the prover cannot see inside it. Runs on aarch64-macos, built by kayo; the
# oracle reads it but cannot run assembly, so it stops at the first block (`.oracle`).

# a times b plus c, in one instruction
fn mul_add_asm ['a': int64, 'b': int64, 'c': int64] -> int64 {
    no_check inline.ASM {
        madd x0, x0, x1, x2
    } with ['a' -> x0, 'b' -> x1, 'c' -> x2, 'r' <- x0];
    return 'r';
}

# how many zero bits before the first 1, counting from the top
fn leading_zeros_asm ['n': uint64] -> uint64 {
    let mut 'z': uint64 = 'n';
    no_check inline.ASM {
        clz x0, x0
    } with ['z' <-> x0];
    return 'z';
}

fn leading_zeros_zag ['n': uint64] -> uint64 {
    let mut 'z': uint64 = 0;
    let mut 'mask': uint64 = 9223372036854775808;   # the top bit
    while 'mask' not_equal 0 and ('n' and.bit 'mask') equal_to 0 {
        change 'mask' = 'mask' shr 1;
        change 'z' = 'z' + 1;
    }
    return 'z';
}

MAIN {
    std::print.useful["6 times 7 plus 3: assembly ", mul_add_asm[6, 7, 3], ", CyborgPL ", 6 x 7 + 3, newln];
    let 'xs': list[uint64] = [1, 255, 65536, 9223372036854775808, 0];
    for 'n' in lend 'xs' {
        std::print.useful['n', " has ", leading_zeros_asm['n'], " leading zeros (CyborgPL: ", leading_zeros_zag['n'], ")", newln];
    }
}
