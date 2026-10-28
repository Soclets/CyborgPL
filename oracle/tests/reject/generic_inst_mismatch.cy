rec Box <T: any>:
    var item: <T>;
;rec Box
MAIN:
    let $b: Box[i32] := Box[item = "text"];
;MAIN
