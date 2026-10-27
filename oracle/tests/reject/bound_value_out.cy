# a named bound checks as the type it names
coll.bounded quantity:
    quantity = int.sign.32[min: 1, max: 99];
end of coll.bounded quantity
!MAIN:
    let $n: quantity := 100;
!end of MAIN
