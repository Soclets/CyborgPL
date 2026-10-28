%layout.C union Number:
    i: i32;
    f: f32;
;%layout.C union Number

MAIN:
    let $n := Number[i = 1, f = 2.0];
;MAIN
