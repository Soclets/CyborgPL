# Inline assembly (ffi.md), arm64: values into registers and out, a real
# syscall (getpid, 20) checked against C's, and x29/x30 clobbered in a
# function that then returns (spelled fp/lr for LLVM: the compiler rule).
%extern.C proc exit [let $code: i32];
%extern.C proc getpid [] -> i32;
proc trample [let $n: i64] -> i64:
    no_check inline.ASM with [$n -> x0, $m <- x0, clobber x30, clobber x29]:
        mov x30, #0
        mov x29, #0
        add x0, x0, #1
    ;inline.ASM
    ret $m;
;proc trample
MAIN:
    let $a: i64 := 20;
    let $b: i64 := 2;
    no_check inline.ASM with [$a -> x0, $b -> x1, $r <- x0, clobber x16]:
        add x0, x0, x1
        mul x0, x0, x1
        mov x16, #1
        add x0, x0, x16
    ;inline.ASM
    no_check inline.ASM with [$pid <- x0, clobber x16, clobber x1]:
        mov x16, #20
        svc #0x80
    ;inline.ASM
    var $got: i32 = 0;
    if $r == 45:
        change $got to $got + 1;
    ;if
    if trample(41) == 42:
        change $got to $got + 4;
    ;if
    no_check let $c := getpid();
    if to.i32($pid) == $c:
        change $got to $got + 2;
    ;if
    no_check my.exit($got);
;MAIN
