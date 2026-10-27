# Inline ARM64 assembly beside the same work in CyborgPL (ffi.md, `inline.ASM`). Each `with [ … ]` binds
# variables to registers: `->` copies a value in before the block, `<-` stores a register into a new
# variable after it, and `<->` does both, keeping the variable's type. Assembly is always a
# `no_check` site: the prover cannot see inside it. Runs on aarch64-macos, built by kayo; the
# oracle reads it but cannot run assembly, so it stops at the first block (`.oracle`).

# a times b plus c, in one instruction
procedure mul_add_asm [$a: integer.signed.64_bit, $b: integer.signed.64_bit, $c: integer.signed.64_bit] -> integer.signed.64_bit:
    !no_check inline.ASM with [$a -> x0, $b -> x1, $c -> x2, $r <- x0]:
        madd x0, x0, x1, x2
    end of inline.ASM
    return $r;
end of procedure mul_add_asm

# how many zero bits before the first 1, counting from the top
procedure leading_zeros_asm [$n: integer.unsigned.64_bit] -> integer.unsigned.64_bit:
    variable $z: integer.unsigned.64_bit = $n;
    !no_check inline.ASM with [$z <-> x0]:
        clz x0, x0
    end of inline.ASM
    return $z;
end of procedure leading_zeros_asm

function leading_zeros_zag [$n: integer.unsigned.64_bit] -> integer.unsigned.64_bit:
    variable $z: integer.unsigned.64_bit = 0;
    variable $mask: integer.unsigned.64_bit = 9223372036854775808;   # the top bit
    while $mask not_equal_to 0 and ($n and.bit $mask) equal_to 0:
        change $mask = $mask shift_right 1;
        change $z = $z plus 1;
    end of while
    return $z;
end of function leading_zeros_zag

!MAIN:
    std::print.shortest("6 times 7 plus 3: assembly ", mul_add_asm(6, 7, 3), ", CyborgPL ", 6 times 7 plus 3, newline);
    let $xs: std::list[integer.unsigned.64_bit] = [1, 255, 65536, 9223372036854775808, 0];
    for $n in lend $xs:
        std::print.shortest($n, " has ", leading_zeros_asm($n), " leading zeros (CyborgPL: ", leading_zeros_zag($n), ")", newline);
    end of for
!end of MAIN
