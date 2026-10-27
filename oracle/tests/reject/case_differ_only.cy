case.any [fn];
fn area [] -> int.sign.32:
    ret 1;
end of fn area
rec Area:
    var n: int.sign.32;
end of rec Area
!MAIN:
    let $a := area();
!end of MAIN
