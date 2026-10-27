sig Number:
    int.sign.32;
end of sig Number
rec Box <T: Number>:
    var item: <T>;
end of rec Box
!MAIN:
    let $b: Box[str.utf8] := Box[item = "x"];
!end of MAIN
