# Inline ARM64 assembly beside the same work in CyborgPL (ffi.md, `inline.ASM`). Each `with [ … ]` binds
# variables to registers: `->` copies a value in before the block, `<-` stores a register into a new
# variable after it, and `<->` does both, keeping the variable's type. Assembly is always a
# `no_check` site: the prover cannot see inside it. Runs on aarch64-macos, built by kayo; the
# oracle reads it but cannot run assembly, so it stops at the first block (`.oracle`).

# a times b plus c, in one instruction
proc mul_add_asm [let $a: i64, let $b: i64, let $c: i64] -> i64:
    no_check inline.ASM with [$a -> x0, $b -> x1, $c -> x2, $r <- x0]:
        madd x0, x0, x1, x2
    ;inline.ASM
    ret $r;
;proc mul_add_asm

# how many zero bits before the first 1, counting from the top
proc leading_zeros_asm [let $n: u64] -> u64:
    var $z: u64 = $n;
    no_check inline.ASM with [$z <-> x0]:
        clz x0, x0
    ;inline.ASM
    ret $z;
;proc leading_zeros_asm

fn leading_zeros_zag [let $n: u64] -> u64:
    var $z: u64 = 0;
    var $mask: u64 = 9223372036854775808;   # the top bit
    while $mask != 0 and ($n and.bit $mask) == 0:
        change $mask to $mask shr 1;
        change $z to $z + 1;
    ;while
    ret $z;
;fn leading_zeros_zag

MAIN:
    std::print("6 times 7 plus 3: assembly ", mul_add_asm(6, 7, 3), ", CyborgPL ", 6 x 7 + 3, newln);
    let $xs: std::ls[u64] := [1, 255, 65536, 9223372036854775808, 0];
    for $n in lend $xs:
        std::print($n, " has ", leading_zeros_asm($n), " leading zeros (CyborgPL: ", leading_zeros_zag($n), ")", newln);
    ;for
;MAIN
