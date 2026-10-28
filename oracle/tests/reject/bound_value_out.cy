# a named bound checks as the type it names
coll.bounded quantity:
    quantity = i32[min: 1, max: 99];
;coll.bounded quantity
MAIN:
    let $n: quantity := 100;
;MAIN
