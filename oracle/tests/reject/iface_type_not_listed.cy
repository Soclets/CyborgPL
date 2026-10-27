set Number:
    int.sign.32;
    float.bin.64;
end of set Number
fn f <T: Number> [let $a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
    let $s := f("text");
!end of MAIN
