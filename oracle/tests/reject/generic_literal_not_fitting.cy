fn half_start [<T>: any] [let $x: <T>] -> <T>:
    let $h: <T> := 0.5;
    ret $h;
;fn half_start
MAIN:
    std::print(half_start(4: i32), newln);
;MAIN
