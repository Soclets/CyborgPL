fn look [lent $a: str.utf8] -> int.sign.32:
    ret 1;
end of fn look
fn apply [$f: fn[str.utf8] -> int.sign.32] -> int.sign.32:
    ret $f("x");
end of fn apply
!MAIN:
    let $x := apply(look);
!end of MAIN
