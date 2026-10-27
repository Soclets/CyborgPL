case.any [fn];
case.exact fn Area [] -> int.sign.32:
    ret 1;
end of fn Area
!MAIN:
    let $a := Area();
!end of MAIN
