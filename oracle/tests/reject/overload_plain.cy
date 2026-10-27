fn f [$a: int.sign.64] -> int.sign.64:
    ret 1;
end of fn f
fn f [$b: float.bin.64] -> int.sign.64:
    ret 2;
end of fn f
!MAIN:
    std::print.shortest(f(1), newln);
!end of MAIN
