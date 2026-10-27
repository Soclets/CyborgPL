rec Box <T: any>:
    var item: <T>;
end of rec Box
!MAIN:
    let $b: Box[int.sign.32] = Box[item = "text"];
!end of MAIN
