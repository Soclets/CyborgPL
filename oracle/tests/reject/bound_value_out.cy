# a named bound checks as the type it names
collection.bounded quantity:
    quantity = integer.signed.32_bit[minimum: 1, maximum: 99];
end of collection.bounded quantity
!MAIN:
    let $n: quantity = 100;
!end of MAIN
