set Number:
    i32;
;set Number
rec Box [<T>: Number]:
    var item: <T>;
;rec Box
MAIN:
    let $b: Box[str.utf8] := Box[item = "x"];
;MAIN
