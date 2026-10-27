fn big [let $a: int.sign.32] -> int.sign.32:
    ret $a times $a;
end of fn big
!MAIN:
    let $r := big(100000);
!end of MAIN
