interface Number:
    integer.signed.32_bit;
end of interface Number
record Box <T: Number>:
    item: <T>;
end of record Box
!MAIN:
    let $b: Box[string.utf8] = Box[item = "x"];
!end of MAIN
