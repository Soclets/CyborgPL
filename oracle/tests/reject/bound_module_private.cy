# a module's bound is reached from outside it only when `pub`, as its other items are (owner, 2026-10-26)
module till:
    coll.bounded coin:
        coin = i32[min: 1, max: 200];
    ;coll.bounded coin
    pub fn show [let $c: coin] -> i32:
        ret $c;
    ;fn show
;module till
MAIN:
    let $c: till.coin := 50;
    std::print(till.show($c), newln);
    let $bad: till.coin := 500;
;MAIN
