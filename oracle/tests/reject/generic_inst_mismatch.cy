record Box <T: any>:
    item: <T>;
end of record Box
!MAIN:
    let $b: Box[integer.signed.32_bit] = Box[item = "text"];
!end of MAIN
