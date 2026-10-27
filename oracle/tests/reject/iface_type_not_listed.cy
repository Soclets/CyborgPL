interface Number:
    int.sign.32;
    float.bin.64;
end of interface Number
fn f <T: Number> [$a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
    let $s := f("text");
!end of MAIN
